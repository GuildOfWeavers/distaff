use std::cmp;
use crate::math::{ FiniteField };
use crate::processor::opcodes;
use crate::stark::{ ProgramInputs, utils::Hasher };
use crate::stark::{ MIN_STACK_DEPTH, MAX_STACK_DEPTH, AUX_WIDTH, HASH_STATE_WIDTH };
use crate::utils::{ filled_vector };

// CONSTANTS
// ================================================================================================
const MIN_USER_STACK_DEPTH: usize = MIN_STACK_DEPTH - AUX_WIDTH;
const MAX_USER_STACK_DEPTH: usize = MAX_STACK_DEPTH - AUX_WIDTH;

// TRACE BUILDER
// ================================================================================================
pub fn execute<T>(program: &[T], inputs: &ProgramInputs<T>, extension_factor: usize) -> Vec<Vec<T>>
    where T: FiniteField + Hasher
{
    let trace_length = program.len();
    let domain_size = trace_length * extension_factor;

    assert!(program.len() > 1, "program length must be greater than 1");
    assert!(program.len().is_power_of_two(), "program length must be a power of 2");
    assert!(program[0] == T::from(opcodes::BEGIN), "first operation of a program must be BEGIN");
    assert!(program[program.len() - 1] == T::from(opcodes::NOOP), "last operation of a program must be NOOP");
    assert!(extension_factor.is_power_of_two(), "trace extension factor must be a power of 2");

    // allocate space for stack registers and populate the first state with public inputs
    let public_inputs = inputs.get_public_inputs();
    let init_stack_depth = cmp::max(public_inputs.len(), MIN_USER_STACK_DEPTH);
    let mut user_registers: Vec<Vec<T>> = Vec::with_capacity(init_stack_depth);
    for i in 0..init_stack_depth {
        let mut register = filled_vector(trace_length, domain_size, T::ZERO);
        if i < public_inputs.len() { 
            register[0] = public_inputs[i];
        }
        user_registers.push(register);
    }

    let mut aux_registers = Vec::with_capacity(AUX_WIDTH);
    for _ in 0..AUX_WIDTH {
        aux_registers.push(filled_vector(trace_length, domain_size, T::ZERO));
    }

    // reverse secret inputs so that they are consumed in FIFO order
    let [secret_inputs_a, secret_inputs_b] = inputs.get_secret_inputs();
    let mut secret_inputs_a = secret_inputs_a.clone();
    secret_inputs_a.reverse();
    let mut secret_inputs_b = secret_inputs_b.clone();
    secret_inputs_b.reverse();

    let mut stack = StackTrace {
        aux_registers,
        user_registers,
        secret_inputs_a,
        secret_inputs_b,
        max_depth: public_inputs.len(),
        depth: public_inputs.len()
    };

    // execute the program capturing each successive stack state in the trace
    let mut i = 0; 
    while i < trace_length - 1 {
        // update stack state based on the current operation
        // TODO: make sure operation can be safely cast to u8
        match program[i].as_u8() {
            opcodes::BEGIN => stack.noop(i),
            opcodes::NOOP  => stack.noop(i),

            opcodes::PUSH  => {
                // push the value of the next instruction onto the stack and skip a step
                // since next instruction is not an operation
                stack.push(i, program[i + 1]);
                i += 1;
                stack.noop(i);
            },

            opcodes::READ    => stack.read(i),
            opcodes::READ2   => stack.read2(i),

            opcodes::PAD2    => stack.pad2(i),
            opcodes::DUP     => stack.dup(i),
            opcodes::DUP2    => stack.dup2(i),
            opcodes::DUP4    => stack.dup4(i),

            opcodes::DROP    => stack.drop(i),
            opcodes::DROP4   => stack.drop4(i),

            opcodes::SWAP    => stack.swap(i),
            opcodes::SWAP2   => stack.swap2(i),
            opcodes::SWAP4   => stack.swap4(i),

            opcodes::ROLL4   => stack.roll4(i),
            opcodes::ROLL8   => stack.roll8(i),

            opcodes::CHOOSE  => stack.choose(i),
            opcodes::CHOOSE2 => stack.choose2(i),

            opcodes::ADD     => stack.add(i),
            opcodes::MUL     => stack.mul(i),
            opcodes::INV     => stack.inv(i),
            opcodes::NEG     => stack.neg(i),

            opcodes::HASH    => stack.hash(i),

            _ => panic!("operation {} is not supported", program[i])
        }
        i += 1;
    }

    // make sure all secret inputs have been consumed
    assert!(stack.secret_inputs_a.len() == 0 && stack.secret_inputs_b.len() == 0,
        "not all secret inputs have been consumed");

    // keep only the registers used during program execution
    stack.user_registers.truncate(stack.max_depth);
    let mut registers = Vec::with_capacity(AUX_WIDTH + stack.user_registers.len());
    registers.append(&mut stack.aux_registers);
    registers.append(&mut stack.user_registers);

    return registers;
}

// TYPES AND INTERFACES
// ================================================================================================
struct StackTrace<T: FiniteField + Hasher> {
    aux_registers   : Vec<Vec<T>>,
    user_registers  : Vec<Vec<T>>,
    secret_inputs_a : Vec<T>,
    secret_inputs_b : Vec<T>,
    max_depth       : usize,
    depth           : usize,
}

// STACK IMPLEMENTATION
// ================================================================================================
impl <T> StackTrace<T>
    where T: FiniteField + Hasher
{
    // OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn noop(&mut self, step: usize) {
        self.copy_state(step, 0);
    }

    fn push(&mut self, step: usize, value: T) {
        self.shift_right(step, 0, 1);
        self.user_registers[0][step + 1] = value;
    }

    fn read(&mut self, step: usize) {
        assert!(self.secret_inputs_a.len() > 0, "ran out of secret inputs at step {}", step);
        self.shift_right(step, 0, 1);
        let value = self.secret_inputs_a.pop().unwrap();
        self.user_registers[0][step + 1] = value;
    }

    fn read2(&mut self, step: usize) {
        assert!(self.secret_inputs_a.len() > 0, "ran out of secret inputs at step {}", step);
        assert!(self.secret_inputs_b.len() > 0, "ran out of secret inputs at step {}", step);
        self.shift_right(step, 0, 2);
        let value_a = self.secret_inputs_a.pop().unwrap();
        let value_b = self.secret_inputs_b.pop().unwrap();
        self.user_registers[0][step + 1] = value_b;
        self.user_registers[1][step + 1] = value_a;
    }

    fn drop(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        self.shift_left(step, 1, 1);
    }

    fn drop4(&mut self, step: usize) {
        assert!(self.depth >= 4, "stack underflow at step {}", step);
        self.shift_left(step, 4, 4);
    }

    fn swap(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        self.user_registers[0][step + 1] = self.user_registers[1][step];
        self.user_registers[1][step + 1] = self.user_registers[0][step];
        self.copy_state(step, 2);
    }

    fn swap2(&mut self, step: usize) {
        assert!(self.depth >= 4, "stack underflow at step {}", step);
        self.user_registers[0][step + 1] = self.user_registers[2][step];
        self.user_registers[1][step + 1] = self.user_registers[3][step];
        self.user_registers[2][step + 1] = self.user_registers[0][step];
        self.user_registers[3][step + 1] = self.user_registers[1][step];
        self.copy_state(step, 4);
    }

    fn swap4(&mut self, step: usize) {
        assert!(self.depth >= 8, "stack underflow at step {}", step);
        self.user_registers[0][step + 1] = self.user_registers[4][step];
        self.user_registers[1][step + 1] = self.user_registers[5][step];
        self.user_registers[2][step + 1] = self.user_registers[6][step];
        self.user_registers[3][step + 1] = self.user_registers[7][step];
        self.user_registers[4][step + 1] = self.user_registers[0][step];
        self.user_registers[5][step + 1] = self.user_registers[1][step];
        self.user_registers[6][step + 1] = self.user_registers[2][step];
        self.user_registers[7][step + 1] = self.user_registers[3][step];
        self.copy_state(step, 8);
    }

    fn roll4(&mut self, step: usize) {
        assert!(self.depth >= 4, "stack underflow at step {}", step);
        self.user_registers[0][step + 1] = self.user_registers[3][step];
        self.user_registers[1][step + 1] = self.user_registers[0][step];
        self.user_registers[2][step + 1] = self.user_registers[1][step];
        self.user_registers[3][step + 1] = self.user_registers[2][step];
        self.copy_state(step, 4);
    }

    fn roll8(&mut self, step: usize) {
        assert!(self.depth >= 8, "stack underflow at step {}", step);
        self.user_registers[0][step + 1] = self.user_registers[7][step];
        self.user_registers[1][step + 1] = self.user_registers[0][step];
        self.user_registers[2][step + 1] = self.user_registers[1][step];
        self.user_registers[3][step + 1] = self.user_registers[2][step];
        self.user_registers[4][step + 1] = self.user_registers[3][step];
        self.user_registers[5][step + 1] = self.user_registers[4][step];
        self.user_registers[6][step + 1] = self.user_registers[5][step];
        self.user_registers[7][step + 1] = self.user_registers[6][step];
        self.copy_state(step, 8);
    }

    fn choose(&mut self, step: usize) {
        assert!(self.depth >= 3, "stack underflow at step {}", step);
        let condition = self.user_registers[2][step];
        if condition == T::ONE {
            self.user_registers[0][step + 1] = self.user_registers[0][step];
        }
        else if condition == T::ZERO {
            self.user_registers[0][step + 1] = self.user_registers[1][step];
        }
        else {
            assert!(false, "cannot CHOOSE on a non-binary condition");
        }
        self.shift_left(step, 3, 2);
    }

    fn choose2(&mut self, step: usize) {
        assert!(self.depth >= 6, "stack underflow at step {}", step);
        let condition = self.user_registers[4][step];
        if condition == T::ONE {
            self.user_registers[0][step + 1] = self.user_registers[0][step];
            self.user_registers[1][step + 1] = self.user_registers[1][step];
        }
        else if condition == T::ZERO {
            self.user_registers[0][step + 1] = self.user_registers[2][step];
            self.user_registers[1][step + 1] = self.user_registers[3][step];
        }
        else {
            assert!(false, "cannot CHOOSE on a non-binary condition");
        }
        self.shift_left(step, 6, 4);
    }

    fn pad2(&mut self, step: usize) {
        self.shift_right(step, 0, 2);
        self.user_registers[0][step + 1] = T::ZERO;
        self.user_registers[1][step + 1] = T::ZERO;
    }

    fn dup(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        self.shift_right(step, 0, 1);
        self.user_registers[0][step + 1] = self.user_registers[0][step];
    }

    fn dup2(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        self.shift_right(step, 0, 2);
        self.user_registers[0][step + 1] = self.user_registers[0][step];
        self.user_registers[1][step + 1] = self.user_registers[1][step];
    }

    fn dup4(&mut self, step: usize) {
        assert!(self.depth >= 4, "stack underflow at step {}", step);
        self.shift_right(step, 0, 4);
        self.user_registers[0][step + 1] = self.user_registers[0][step];
        self.user_registers[1][step + 1] = self.user_registers[1][step];
        self.user_registers[2][step + 1] = self.user_registers[2][step];
        self.user_registers[3][step + 1] = self.user_registers[3][step];
    }

    fn add(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        let y = self.user_registers[1][step];
        self.user_registers[0][step + 1] = T::add(x, y);
        self.shift_left(step, 2, 1);
    }

    fn mul(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        let y = self.user_registers[1][step];
        self.user_registers[0][step + 1] = T::mul(x, y);
        self.shift_left(step, 2, 1);
    }

    fn inv(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        self.user_registers[0][step + 1] = T::inv(x);
        self.copy_state(step, 1);
    }

    fn neg(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        self.user_registers[0][step + 1] = T::neg(x);
        self.copy_state(step, 1);
    }

    fn hash(&mut self, step: usize) {
        assert!(self.depth >= HASH_STATE_WIDTH, "stack underflow at step {}", step);
        let mut state = [
            self.user_registers[0][step],
            self.user_registers[1][step],
            self.user_registers[2][step],
            self.user_registers[3][step],
            self.user_registers[4][step],
            self.user_registers[5][step],
        ];

        T::apply_round(&mut state, step);

        self.user_registers[0][step + 1] = state[0];
        self.user_registers[1][step + 1] = state[1];
        self.user_registers[2][step + 1] = state[2];
        self.user_registers[3][step + 1] = state[3];
        self.user_registers[4][step + 1] = state[4];
        self.user_registers[5][step + 1] = state[5];

        self.copy_state(step, HASH_STATE_WIDTH);
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------

    fn copy_state(&mut self, step: usize, start: usize,) {
        for i in start..self.depth {
            let slot_value = self.user_registers[i][step];
            self.user_registers[i][step + 1] = slot_value;
        }
    }

    fn shift_left(&mut self, step: usize, start: usize, pos_count: usize) {
        assert!(self.depth >= pos_count, "stack underflow at step {}", step);
        
        // shift all values by pos_count to the left
        for i in start..self.depth {
            let slot_value = self.user_registers[i][step];
            self.user_registers[i - pos_count][step + 1] = slot_value;
        }

        // set all "shifted-in" slots to 0
        for i in (self.depth - pos_count)..self.depth {
            self.user_registers[i][step + 1] = T::ZERO;
        }

        // stack depth has been reduced by pos_count
        self.depth -= pos_count;
    }

    fn shift_right(&mut self, step: usize, start: usize, pos_count: usize) {
        
        self.depth += pos_count;
        assert!(self.depth <= MAX_USER_STACK_DEPTH, "stack overflow at step {}", step);

        if self.depth > self.max_depth {
            self.max_depth += pos_count;
            if self.max_depth > self.user_registers.len() {
                self.add_registers(self.max_depth - self.user_registers.len());
            }
        }

        for i in start..(self.depth - pos_count) {
            let slot_value = self.user_registers[i][step];
            self.user_registers[i + pos_count][step + 1] = slot_value;
        }
    }

    /// Extends the stack by the specified number of registers
    fn add_registers(&mut self, num_registers: usize) {
        let trace_length = self.user_registers[0].len();
        let trace_capacity = self.user_registers[0].capacity();
        for _ in 0..num_registers {
            let register = filled_vector(trace_length, trace_capacity, T::ZERO);
            self.user_registers.push(register);
        }
    }
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    
    use crate::math::{ F128, FiniteField };
    use crate::stark::{ Hasher };
    use crate::utils::{ filled_vector };
    use super::{ AUX_WIDTH };

    const TRACE_LENGTH: usize = 16;
    const EXTENSION_FACTOR: usize = 16;

    #[test]
    fn noop() {
        let mut stack = init_stack(&[1, 2, 3, 4], &[], &[]);
        stack.noop(0);
        assert_eq!(vec![1, 2, 3, 4, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(4, stack.depth);
        assert_eq!(4, stack.max_depth);
    }

    #[test]
    fn swap() {
        let mut stack = init_stack(&[1, 2, 3, 4], &[], &[]);
        stack.swap(0);
        assert_eq!(vec![2, 1, 3, 4, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(4, stack.depth);
        assert_eq!(4, stack.max_depth);
    }

    #[test]
    fn swap2() {
        let mut stack = init_stack(&[1, 2, 3, 4], &[], &[]);
        stack.swap2(0);
        assert_eq!(vec![3, 4, 1, 2, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(4, stack.depth);
        assert_eq!(4, stack.max_depth);
    }

    #[test]
    fn swap4() {
        let mut stack = init_stack(&[1, 2, 3, 4, 5, 6, 7, 8], &[], &[]);
        stack.swap4(0);
        assert_eq!(vec![5, 6, 7, 8, 1, 2, 3, 4], get_stack_state(&stack, 1));

        assert_eq!(8, stack.depth);
        assert_eq!(8, stack.max_depth);
    }

    #[test]
    fn roll4() {
        let mut stack = init_stack(&[1, 2, 3, 4], &[], &[]);
        stack.roll4(0);
        assert_eq!(vec![4, 1, 2, 3, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(4, stack.depth);
        assert_eq!(4, stack.max_depth);
    }

    #[test]
    fn roll8() {
        let mut stack = init_stack(&[1, 2, 3, 4, 5, 6, 7, 8], &[], &[]);
        stack.roll8(0);
        assert_eq!(vec![8, 1, 2, 3, 4, 5, 6, 7], get_stack_state(&stack, 1));

        assert_eq!(8, stack.depth);
        assert_eq!(8, stack.max_depth);
    }

    #[test]
    fn choose() {
        // choose on true
        let mut stack = init_stack(&[2, 3, 0], &[], &[]);
        stack.choose(0);
        assert_eq!(vec![3, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(1, stack.depth);
        assert_eq!(3, stack.max_depth);

        let mut stack = init_stack(&[2, 3, 0, 4], &[], &[]);
        stack.choose(0);
        assert_eq!(vec![3, 4, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(2, stack.depth);
        assert_eq!(4, stack.max_depth);

        // choose on false
        let mut stack = init_stack(&[2, 3, 1, 4], &[], &[]);
        stack.choose(0);
        assert_eq!(vec![2, 4, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(2, stack.depth);
        assert_eq!(4, stack.max_depth);
    }

    #[test]
    fn choose2() {
        // choose on true
        let mut stack = init_stack(&[2, 3, 4, 5, 0, 6, 7], &[], &[]);
        stack.choose2(0);
        assert_eq!(vec![4, 5, 7, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(3, stack.depth);
        assert_eq!(7, stack.max_depth);

        // choose on false
        let mut stack = init_stack(&[2, 3, 4, 5, 1, 6, 7], &[], &[]);
        stack.choose2(0);
        assert_eq!(vec![2, 3, 7, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(3, stack.depth);
        assert_eq!(7, stack.max_depth);
    }

    #[test]
    fn push() {
        let mut stack = init_stack(&[], &[], &[]);
        stack.push(0, 3);
        assert_eq!(vec![3, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(1, stack.depth);
        assert_eq!(1, stack.max_depth);
    }
    
    #[test]
    fn pad2() {
        let mut stack = init_stack(&[1, 2], &[], &[]);
        stack.pad2(0);
        assert_eq!(vec![0, 0, 1, 2, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(4, stack.depth);
        assert_eq!(4, stack.max_depth);
    }

    #[test]
    fn dup() {
        let mut stack = init_stack(&[1, 2], &[], &[]);
        stack.dup(0);
        assert_eq!(vec![1, 1, 2, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(3, stack.depth);
        assert_eq!(3, stack.max_depth);
    }

    #[test]
    fn dup2() {
        let mut stack = init_stack(&[1, 2, 3, 4], &[], &[]);
        stack.dup2(0);
        assert_eq!(vec![1, 2, 1, 2, 3, 4, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(6, stack.depth);
        assert_eq!(6, stack.max_depth);
    }

    #[test]
    fn dup4() {
        let mut stack = init_stack(&[1, 2, 3, 4], &[], &[]);
        stack.dup4(0);
        assert_eq!(vec![1, 2, 3, 4, 1, 2, 3, 4], get_stack_state(&stack, 1));

        assert_eq!(8, stack.depth);
        assert_eq!(8, stack.max_depth);
    }

    #[test]
    fn drop() {
        let mut stack = init_stack(&[1, 2], &[], &[]);
        stack.drop(0);
        assert_eq!(vec![2, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(1, stack.depth);
        assert_eq!(2, stack.max_depth);
    }

    #[test]
    fn drop4() {
        let mut stack = init_stack(&[1, 2, 3, 4, 5], &[], &[]);
        stack.drop4(0);
        assert_eq!(vec![5, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(1, stack.depth);
        assert_eq!(5, stack.max_depth);
    }

    #[test]
    fn add() {
        let mut stack = init_stack(&[1, 2], &[], &[]);
        stack.add(0);
        assert_eq!(vec![3, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(1, stack.depth);
        assert_eq!(2, stack.max_depth);
    }

    #[test]
    fn mul() {
        let mut stack = init_stack(&[2, 3], &[], &[]);
        stack.mul(0);
        assert_eq!(vec![6, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(1, stack.depth);
        assert_eq!(2, stack.max_depth);
    }

    #[test]
    fn inv() {
        let mut stack = init_stack(&[2, 3], &[], &[]);
        stack.inv(0);
        assert_eq!(vec![F128::inv(2), 3, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(2, stack.depth);
        assert_eq!(2, stack.max_depth);
    }

    #[test]
    fn neg() {
        let mut stack = init_stack(&[2, 3], &[], &[]);
        stack.neg(0);
        assert_eq!(vec![F128::neg(2), 3, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(2, stack.depth);
        assert_eq!(2, stack.max_depth);
    }

    #[test]
    fn hash() {
        let mut stack = init_stack(&[0, 0, 1, 2, 3, 4], &[], &[]);
        let mut expected = vec![0, 0, 1, 2, 3, 4, 0, 0];

        stack.hash(0);
        <F128 as Hasher>::apply_round(&mut expected[..F128::STATE_WIDTH], 0);
        assert_eq!(expected, get_stack_state(&stack, 1));

        stack.hash(1);
        <F128 as Hasher>::apply_round(&mut expected[..F128::STATE_WIDTH], 1);
        assert_eq!(expected, get_stack_state(&stack, 2));

        assert_eq!(6, stack.depth);
        assert_eq!(6, stack.max_depth);
    }

    #[test]
    fn read() {
        let mut stack = init_stack(&[1], &[2, 3], &[]);

        stack.read(0);
        assert_eq!(vec![2, 1, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(2, stack.depth);
        assert_eq!(2, stack.max_depth);

        stack.read(1);
        assert_eq!(vec![3, 2, 1, 0, 0, 0, 0, 0], get_stack_state(&stack, 2));

        assert_eq!(3, stack.depth);
        assert_eq!(3, stack.max_depth);
    }

    #[test]
    fn read2() {
        let mut stack = init_stack(&[1], &[2, 4], &[3, 5]);

        stack.read2(0);
        assert_eq!(vec![3, 2, 1, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(3, stack.depth);
        assert_eq!(3, stack.max_depth);

        stack.read2(1);
        assert_eq!(vec![5, 4, 3, 2, 1, 0, 0, 0], get_stack_state(&stack, 2));

        assert_eq!(5, stack.depth);
        assert_eq!(5, stack.max_depth);
    }

    fn init_stack(public_inputs: &[F128], secret_inputs_a: &[F128], secret_inputs_b: &[F128]) -> super::StackTrace<F128> {
        let mut user_registers: Vec<Vec<F128>> = Vec::with_capacity(super::MIN_USER_STACK_DEPTH);
        for i in 0..super::MIN_USER_STACK_DEPTH {
            let mut register = filled_vector(TRACE_LENGTH, TRACE_LENGTH * EXTENSION_FACTOR, F128::ZERO);
            if i < public_inputs.len() { 
                register[0] = public_inputs[i];
            }
            user_registers.push(register);
        }
    
        let mut aux_registers = Vec::with_capacity(AUX_WIDTH);
        for _ in 0..AUX_WIDTH {
            aux_registers.push(filled_vector(TRACE_LENGTH, TRACE_LENGTH * EXTENSION_FACTOR, F128::ZERO));
        }

        let mut secret_inputs_a = secret_inputs_a.to_vec();
        secret_inputs_a.reverse();
        let mut secret_inputs_b = secret_inputs_b.to_vec();
        secret_inputs_b.reverse();

        return super::StackTrace {
            aux_registers,
            user_registers,
            secret_inputs_a,
            secret_inputs_b,
            max_depth: public_inputs.len(),
            depth    : public_inputs.len()
        };
    }

    fn get_stack_state(stack: &super::StackTrace<F128>, step: usize) -> Vec<F128> {
        let mut state = Vec::with_capacity(stack.user_registers.len());
        for i in 0..stack.user_registers.len() {
            state.push(stack.user_registers[i][step]);
        }
        return state;
    }
}