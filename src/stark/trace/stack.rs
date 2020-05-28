use std::cmp;
use crate::math::{ FiniteField };
use crate::processor::opcodes;
use crate::utils::{ filled_vector };
use super::{ MAX_INPUTS, MIN_STACK_DEPTH, MAX_STACK_DEPTH };

// TRACE BUILDER
// ================================================================================================
pub fn execute<T>(program: &[T], inputs: &[T], extension_factor: usize) -> Vec<Vec<T>>
    where T: FiniteField
{
    let trace_length = program.len();
    let domain_size = trace_length * extension_factor;

    assert!(program.len() > 1, "program length must be greater than 1");
    assert!(program.len().is_power_of_two(), "program length must be a power of 2");
    assert!(program[0] == T::from(opcodes::BEGIN), "first operation of a program must be BEGIN");
    assert!(program[program.len() - 1] == T::from(opcodes::NOOP), "last operation of a program must be NOOP");
    assert!(extension_factor.is_power_of_two(), "trace extension factor must be a power of 2");
    assert!(inputs.len() <= MAX_INPUTS, "expected {} or fewer inputs, received {}", MAX_INPUTS, inputs.len());

    // allocate space for stack registers and populate the first state with input values
    let init_stack_depth = cmp::max(inputs.len(), MIN_STACK_DEPTH);
    let mut registers: Vec<Vec<T>> = Vec::with_capacity(init_stack_depth);
    for i in 0..init_stack_depth {
        let mut register = filled_vector(trace_length, domain_size, T::ZERO);
        if i < inputs.len() { 
            register[0] = inputs[i];
        }
        registers.push(register);
    }

    let mut stack = StackTrace { registers, max_depth: inputs.len(), depth: inputs.len() };

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

            opcodes::DROP    => stack.drop(i),

            opcodes::DUP     => stack.dup(i),
            opcodes::DUP2    => stack.dup2(i),
            opcodes::DUP4    => stack.dup4(i),

            opcodes::SWAP    => stack.swap(i),
            opcodes::SWAP2   => stack.swap2(i),
            opcodes::SWAP4   => stack.swap4(i),

            opcodes::ROLL4   => stack.roll4(i),
            opcodes::ROLL8   => stack.roll8(i),

            opcodes::CHOOSE  => stack.choose(i),
            opcodes::CHOOSE2 => stack.choose2(i),

            opcodes::ADD     => stack.add(i),
            opcodes::SUB     => stack.sub(i),
            opcodes::MUL     => stack.mul(i),

            _ => panic!("operation {} is not supported", program[i])
        }
        i += 1;
    }

    // keep only the registers used during program execution
    stack.registers.truncate(stack.max_depth);

    return stack.registers;
}

// TYPES AND INTERFACES
// ================================================================================================
struct StackTrace<T> 
    where T: FiniteField
{
    registers   : Vec<Vec<T>>,
    max_depth   : usize,
    depth       : usize,
}

// STACK IMPLEMENTATION
// ================================================================================================
impl <T> StackTrace<T>
    where T: FiniteField
{
    // OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn noop(&mut self, step: usize) {
        self.copy_state(step, 0);
    }

    fn swap(&mut self, step: usize) {
        self.registers[0][step + 1] = self.registers[1][step];
        self.registers[1][step + 1] = self.registers[0][step];
        self.copy_state(step, 2);
    }

    fn swap2(&mut self, step: usize) {
        // TODO: update depth?
        self.registers[0][step + 1] = self.registers[2][step];
        self.registers[1][step + 1] = self.registers[3][step];
        self.registers[2][step + 1] = self.registers[0][step];
        self.registers[3][step + 1] = self.registers[1][step];
        self.copy_state(step, 4);
    }

    fn swap4(&mut self, step: usize) {
        // TODO: update depth?
        self.registers[0][step + 1] = self.registers[4][step];
        self.registers[1][step + 1] = self.registers[5][step];
        self.registers[2][step + 1] = self.registers[6][step];
        self.registers[3][step + 1] = self.registers[7][step];
        self.registers[4][step + 1] = self.registers[0][step];
        self.registers[5][step + 1] = self.registers[1][step];
        self.registers[6][step + 1] = self.registers[2][step];
        self.registers[7][step + 1] = self.registers[3][step];
        self.copy_state(step, 8);
    }

    fn roll4(&mut self, step: usize) {
        // TODO: update depth?
        self.registers[0][step + 1] = self.registers[3][step];
        self.registers[1][step + 1] = self.registers[0][step];
        self.registers[2][step + 1] = self.registers[1][step];
        self.registers[3][step + 1] = self.registers[2][step];
        self.copy_state(step, 4);
    }

    fn roll8(&mut self, step: usize) {
        // TODO: update depth?
        self.registers[0][step + 1] = self.registers[7][step];
        self.registers[1][step + 1] = self.registers[0][step];
        self.registers[2][step + 1] = self.registers[1][step];
        self.registers[3][step + 1] = self.registers[2][step];
        self.registers[4][step + 1] = self.registers[3][step];
        self.registers[5][step + 1] = self.registers[4][step];
        self.registers[6][step + 1] = self.registers[5][step];
        self.registers[7][step + 1] = self.registers[6][step];
        self.copy_state(step, 8);
    }

    fn choose(&mut self, step: usize) {
        // TODO: check depth?
        let condition = self.registers[2][step];
        if condition == T::ONE {
            self.registers[0][step + 1] = self.registers[0][step];
        }
        else if condition == T::ZERO {
            self.registers[0][step + 1] = self.registers[1][step];
        }
        else {
            assert!(false, "cannot CHOOSE on a non-binary condition");
        }
        self.shift_left(step, 3, 2);
    }

    fn choose2(&mut self, step: usize) {
        // TODO: check depth?
        let condition = self.registers[4][step];
        if condition == T::ONE {
            self.registers[0][step + 1] = self.registers[0][step];
            self.registers[1][step + 1] = self.registers[1][step];
        }
        else if condition == T::ZERO {
            self.registers[0][step + 1] = self.registers[2][step];
            self.registers[1][step + 1] = self.registers[3][step];
        }
        else {
            assert!(false, "cannot CHOOSE on a non-binary condition");
        }
        self.shift_left(step, 6, 4);
    }

    fn push(&mut self, step: usize, value: T) {
        self.shift_right(step, 0, 1);
        self.registers[0][step + 1] = value;
    }

    fn dup(&mut self, step: usize) {
        self.shift_right(step, 0, 1);
        self.registers[0][step + 1] = self.registers[0][step];
    }

    fn dup2(&mut self, step: usize) {
        self.shift_right(step, 0, 2);
        self.registers[0][step + 1] = self.registers[0][step];
        self.registers[1][step + 1] = self.registers[1][step];
    }

    fn dup4(&mut self, step: usize) {
        self.shift_right(step, 0, 4);
        self.registers[0][step + 1] = self.registers[0][step];
        self.registers[1][step + 1] = self.registers[1][step];
        self.registers[2][step + 1] = self.registers[2][step];
        self.registers[3][step + 1] = self.registers[3][step];
    }

    fn drop(&mut self, step: usize) {
        self.shift_left(step, 1, 1);
    }

    fn add(&mut self, step: usize) {
        let x = self.registers[0][step];
        let y = self.registers[1][step];
        self.registers[0][step + 1] = T::add(x, y);
        self.shift_left(step, 2, 1);
    }

    fn sub(&mut self, step: usize) {
        let x = self.registers[0][step];
        let y = self.registers[1][step];
        self.registers[0][step + 1] = T::sub(y, x);
        self.shift_left(step, 2, 1);
    }

    fn mul(&mut self, step: usize) {
        let x = self.registers[0][step];
        let y = self.registers[1][step];
        self.registers[0][step + 1] = T::mul(x, y);
        self.shift_left(step, 2, 1);
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------

    fn copy_state(&mut self, step: usize, start: usize,) {
        for i in start..self.depth {
            let slot_value = self.registers[i][step];
            self.registers[i][step + 1] = slot_value;
        }
    }

    fn shift_left(&mut self, step: usize, start: usize, pos_count: usize) {
        assert!(self.depth >= pos_count, "stack underflow at step {}", step);
        
        // shift all values by pos_count to the left
        for i in start..self.depth {
            let slot_value = self.registers[i][step];
            self.registers[i - pos_count][step + 1] = slot_value;
        }

        // set all "shifted-in" slots to 0
        for i in (self.depth - pos_count)..self.depth {
            self.registers[i][step + 1] = T::ZERO;
        }

        // stack depth has been reduced by pos_count
        self.depth -= pos_count;
    }

    fn shift_right(&mut self, step: usize, start: usize, pos_count: usize) {
        
        self.depth += pos_count;
        assert!(self.depth <= MAX_STACK_DEPTH, "stack overflow at step {}", step);

        if self.depth > self.max_depth {
            self.max_depth += pos_count;
            if self.max_depth > self.registers.len() {
                self.add_registers(self.max_depth - self.registers.len());
            }
        }

        for i in start..(self.depth - pos_count) {
            let slot_value = self.registers[i][step];
            self.registers[i + pos_count][step + 1] = slot_value;
        }
    }

    /// Extends the stack by the specified number of registers
    fn add_registers(&mut self, num_registers: usize) {
        let trace_length = self.registers[0].len();
        let trace_capacity = self.registers[0].capacity();
        for _ in 0..num_registers {
            let register = filled_vector(trace_length, trace_capacity, T::ZERO);
            self.registers.push(register);
        }
    }
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    
    use crate::math::{ F64, FiniteField };
    use crate::utils::{ filled_vector };

    const TRACE_LENGTH: usize = 16;
    const EXTENSION_FACTOR: usize = 16;

    #[test]
    fn noop() {
        let mut stack = init_stack(&[1, 2, 3, 4]);
        stack.noop(0);
        assert_eq!(vec![1, 2, 3, 4, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(4, stack.depth);
        assert_eq!(4, stack.max_depth);
    }

    #[test]
    fn swap() {
        let mut stack = init_stack(&[1, 2, 3, 4]);
        stack.swap(0);
        assert_eq!(vec![2, 1, 3, 4, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(4, stack.depth);
        assert_eq!(4, stack.max_depth);
    }

    #[test]
    fn swap2() {
        let mut stack = init_stack(&[1, 2, 3, 4]);
        stack.swap2(0);
        assert_eq!(vec![3, 4, 1, 2, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(4, stack.depth);
        assert_eq!(4, stack.max_depth);
    }

    #[test]
    fn swap4() {
        let mut stack = init_stack(&[1, 2, 3, 4, 5, 6, 7, 8]);
        stack.swap4(0);
        assert_eq!(vec![5, 6, 7, 8, 1, 2, 3, 4], get_stack_state(&stack, 1));

        assert_eq!(8, stack.depth);
        assert_eq!(8, stack.max_depth);
    }

    #[test]
    fn roll4() {
        let mut stack = init_stack(&[1, 2, 3, 4]);
        stack.roll4(0);
        assert_eq!(vec![4, 1, 2, 3, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(4, stack.depth);
        assert_eq!(4, stack.max_depth);
    }

    #[test]
    fn roll8() {
        let mut stack = init_stack(&[1, 2, 3, 4, 5, 6, 7, 8]);
        stack.roll8(0);
        assert_eq!(vec![8, 1, 2, 3, 4, 5, 6, 7], get_stack_state(&stack, 1));

        assert_eq!(8, stack.depth);
        assert_eq!(8, stack.max_depth);
    }

    #[test]
    fn choose() {
        // choose on true
        let mut stack = init_stack(&[2, 3, 0]);
        stack.choose(0);
        assert_eq!(vec![3, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(1, stack.depth);
        assert_eq!(3, stack.max_depth);

        let mut stack = init_stack(&[2, 3, 0, 4]);
        stack.choose(0);
        assert_eq!(vec![3, 4, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(2, stack.depth);
        assert_eq!(4, stack.max_depth);

        // choose on false
        let mut stack = init_stack(&[2, 3, 1, 4]);
        stack.choose(0);
        assert_eq!(vec![2, 4, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(2, stack.depth);
        assert_eq!(4, stack.max_depth);
    }

    #[test]
    fn choose2() {
        // choose on true
        let mut stack = init_stack(&[2, 3, 4, 5, 0, 6, 7]);
        stack.choose2(0);
        assert_eq!(vec![4, 5, 7, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(3, stack.depth);
        assert_eq!(7, stack.max_depth);

        // choose on false
        let mut stack = init_stack(&[2, 3, 4, 5, 1, 6, 7]);
        stack.choose2(0);
        assert_eq!(vec![2, 3, 7, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(3, stack.depth);
        assert_eq!(7, stack.max_depth);
    }

    #[test]
    fn push() {
        let mut stack = init_stack(&[]);
        stack.push(0, 3);
        assert_eq!(vec![3, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(1, stack.depth);
        assert_eq!(1, stack.max_depth);
    }
    
    #[test]
    fn dup() {
        let mut stack = init_stack(&[1, 2]);
        stack.dup(0);
        assert_eq!(vec![1, 1, 2, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(3, stack.depth);
        assert_eq!(3, stack.max_depth);
    }

    #[test]
    fn dup2() {
        let mut stack = init_stack(&[1, 2, 3, 4]);
        stack.dup2(0);
        assert_eq!(vec![1, 2, 1, 2, 3, 4, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(6, stack.depth);
        assert_eq!(6, stack.max_depth);
    }

    #[test]
    fn dup4() {
        let mut stack = init_stack(&[1, 2, 3, 4]);
        stack.dup4(0);
        assert_eq!(vec![1, 2, 3, 4, 1, 2, 3, 4], get_stack_state(&stack, 1));

        assert_eq!(8, stack.depth);
        assert_eq!(8, stack.max_depth);
    }

    #[test]
    fn drop() {
        let mut stack = init_stack(&[1, 2]);
        stack.drop(0);
        assert_eq!(vec![2, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(1, stack.depth);
        assert_eq!(2, stack.max_depth);
    }

    #[test]
    fn add() {
        let mut stack = init_stack(&[1, 2]);
        stack.add(0);
        assert_eq!(vec![3, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(1, stack.depth);
        assert_eq!(2, stack.max_depth);
    }

    #[test]
    fn sub() {
        let mut stack = init_stack(&[2, 5]);
        stack.sub(0);
        assert_eq!(vec![3, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(1, stack.depth);
        assert_eq!(2, stack.max_depth);
    }

    #[test]
    fn mul() {
        let mut stack = init_stack(&[2, 3]);
        stack.mul(0);
        assert_eq!(vec![6, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(1, stack.depth);
        assert_eq!(2, stack.max_depth);
    }

    fn init_stack(inputs: &[u64]) -> super::StackTrace<u64> {
        let mut registers: Vec<Vec<u64>> = Vec::with_capacity(super::MIN_STACK_DEPTH);
        for i in 0..super::MIN_STACK_DEPTH {
            let mut register = filled_vector(TRACE_LENGTH, TRACE_LENGTH * EXTENSION_FACTOR, F64::ZERO);
            if i < inputs.len() { 
                register[0] = inputs[i];
            }
            registers.push(register);
        }
    
        return super::StackTrace { registers, max_depth: inputs.len(), depth: inputs.len() };
    }

    fn get_stack_state(stack: &super::StackTrace<u64>, step: usize) -> Vec<u64> {
        let mut state = Vec::with_capacity(stack.registers.len());
        for i in 0..stack.registers.len() {
            state.push(stack.registers[i][step]);
        }
        return state;
    }
}