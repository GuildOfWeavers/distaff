use std::cmp;
use crate::math::{ Field, FiniteField };
use crate::processor::opcodes;
use crate::utils::{ zero_filled_vector };
use super::{ MAX_INPUTS, MIN_STACK_DEPTH, MAX_STACK_DEPTH };

// TRACE BUILDER
// ================================================================================================
pub fn execute(program: &[u64], inputs: &[u64], extension_factor: usize) -> Vec<Vec<u64>> {

    let trace_length = program.len();
    let domain_size = trace_length * extension_factor;

    assert!(inputs.len() <= MAX_INPUTS, "expected {} or fewer inputs, received {}", MAX_INPUTS, inputs.len());
    assert!(trace_length.is_power_of_two(), "trace length must be a power of 2");
    assert!(extension_factor.is_power_of_two(), "trace extension factor must be a power of 2");
    assert!(program[trace_length - 1] == opcodes::NOOP, "last operation of a program must be NOOP");

    // allocate space for stack registers and populate the first state with input values
    let init_stack_depth = cmp::max(inputs.len(), MIN_STACK_DEPTH);
    let mut registers: Vec<Vec<u64>> = Vec::with_capacity(init_stack_depth);
    for i in 0..init_stack_depth {
        let mut register = zero_filled_vector(trace_length, domain_size);
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
        match program[i] {
            opcodes::NOOP  => stack.noop(i),

            opcodes::PUSH  => {
                // push the value of the next instruction onto the stack and skip a step
                // since next instruction is not an operation
                stack.push(i, program[i + 1]);
                i += 1;
                stack.noop(i);
            },
            opcodes::DUP0  => stack.dup0(i),
            opcodes::DUP1  => stack.dup1(i),

            opcodes::PULL1 => stack.pull1(i),
            opcodes::PULL2 => stack.pull2(i),

            opcodes::DROP  => stack.drop(i),
            opcodes::ADD   => stack.add(i),
            opcodes::SUB   => stack.sub(i),
            opcodes::MUL   => stack.mul(i),

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
struct StackTrace {
    registers   : Vec<Vec<u64>>,
    max_depth   : usize,
    depth       : usize,
}

// STACK IMPLEMENTATION
// ================================================================================================
impl StackTrace {

    // OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn noop(&mut self, step: usize) {
        self.copy_state(step, 0);
    }

    fn pull1(&mut self, step: usize) {
        self.registers[0][step + 1] = self.registers[1][step];
        self.registers[1][step + 1] = self.registers[0][step];
        self.copy_state(step, 2);
    }

    fn pull2(&mut self, step: usize) {
        self.registers[0][step + 1] = self.registers[2][step];
        self.registers[1][step + 1] = self.registers[0][step];
        self.registers[2][step + 1] = self.registers[1][step];
        self.copy_state(step, 3);
    }

    fn push(&mut self, step: usize, value: u64) {
        self.shift_right(step, 0, 1);
        self.registers[0][step + 1] = value;
    }

    fn dup0(&mut self, step: usize) {
        self.shift_right(step, 0, 1);
        let value = self.registers[0][step];
        self.registers[0][step + 1] = value;
    }

    fn dup1(&mut self, step: usize) {
        self.shift_right(step, 0, 1);
        let value = self.registers[1][step];
        self.registers[0][step + 1] = value;
    }

    fn drop(&mut self, step: usize) {
        self.shift_left(step, 1, 1);
    }

    fn add(&mut self, step: usize) {
        let x = self.registers[0][step];
        let y = self.registers[1][step];
        self.registers[0][step + 1] = Field::add(x, y);
        self.shift_left(step, 2, 1);
    }

    fn sub(&mut self, step: usize) {
        let x = self.registers[0][step];
        let y = self.registers[1][step];
        self.registers[0][step + 1] = Field::sub(y, x);
        self.shift_left(step, 2, 1);
    }

    fn mul(&mut self, step: usize) {
        let x = self.registers[0][step];
        let y = self.registers[1][step];
        self.registers[0][step + 1] = Field::mul(x, y);
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
            self.registers[i][step + 1] = 0;
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
            let register = zero_filled_vector(trace_length, trace_capacity);
            self.registers.push(register);
        }
    }
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    
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
    fn pull1() {
        let mut stack = init_stack(&[1, 2, 3, 4]);
        stack.pull1(0);
        assert_eq!(vec![2, 1, 3, 4, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(4, stack.depth);
        assert_eq!(4, stack.max_depth);
    }

    #[test]
    fn pull2() {
        let mut stack = init_stack(&[1, 2, 3, 4]);
        stack.pull2(0);
        assert_eq!(vec![3, 1, 2, 4, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(4, stack.depth);
        assert_eq!(4, stack.max_depth);
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
    fn dup0() {
        let mut stack = init_stack(&[1, 2]);
        stack.dup0(0);
        assert_eq!(vec![1, 1, 2, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(3, stack.depth);
        assert_eq!(3, stack.max_depth);
    }

    #[test]
    fn dup1() {
        let mut stack = init_stack(&[1, 2]);
        stack.dup1(0);
        assert_eq!(vec![2, 1, 2, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

        assert_eq!(3, stack.depth);
        assert_eq!(3, stack.max_depth);
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

    fn init_stack(inputs: &[u64]) -> super::StackTrace {
        let mut registers: Vec<Vec<u64>> = Vec::with_capacity(super::MIN_STACK_DEPTH);
        for i in 0..super::MIN_STACK_DEPTH {
            let mut register = super::zero_filled_vector(TRACE_LENGTH, TRACE_LENGTH * EXTENSION_FACTOR);
            if i < inputs.len() { 
                register[0] = inputs[i];
            }
            registers.push(register);
        }
    
        return super::StackTrace { registers, max_depth: inputs.len(), depth: inputs.len() };
    }

    fn get_stack_state(stack: &super::StackTrace, step: usize) -> Vec<u64> {
        let mut state = Vec::with_capacity(stack.registers.len());
        for i in 0..stack.registers.len() {
            state.push(stack.registers[i][step]);
        }
        return state;
    }
}