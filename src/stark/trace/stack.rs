use crate::math::{ field, polys };
use crate::trace::TraceState;
use crate::utils;

// CONSTANTS
// ================================================================================================
const MIN_STACK_DEPTH: usize = 8;
pub const MAX_STACK_DEPTH: usize = 32;

// TYPES AND INTERFACES
// ================================================================================================
pub struct Stack {
    registers   : Vec<Vec<u64>>,
    current_step: usize,
    max_depth   : usize,
    depth       : usize,
}

// STACK IMPLEMENTATION
// ================================================================================================
impl Stack {

    pub fn new(trace_length: usize, extension_factor: usize) -> Stack {
        assert!(trace_length.is_power_of_two(), "trace length must be a power of 2");
        let trace_capacity = trace_length * extension_factor;

        let current_step: usize = 0;
        let mut registers: Vec<Vec<u64>> = Vec::with_capacity(MIN_STACK_DEPTH);
        for _ in 0..MIN_STACK_DEPTH {
            let mut register = utils::zero_filled_vector(trace_length, trace_capacity);
            register[current_step] = 0;
            registers.push(register);
        }
        return Stack { registers, current_step, max_depth: 0, depth: 0 };
    }

    pub fn get_register_trace(&self, index: usize) -> &[u64] {
        return &self.registers[index];
    }

    pub fn fill_state(&self, state: &mut TraceState, step: usize) {
        for i in 0..self.max_depth {
            state.stack[i] = self.registers[i][step];
        }
    }

    pub fn depth(&self) -> usize {
        return self.depth;
    }

    pub fn max_depth(&self) -> usize {
        return self.max_depth;
    }

    pub fn trace_length(&self) -> usize {
        return self.registers[0].len();
    }

    // INTERPOLATION AND EXTENSION
    // --------------------------------------------------------------------------------------------
    pub fn interpolate_registers(&mut self, inv_twiddles: &[u64]) {
        for i in 0..self.max_depth() {
            polys::interpolate_fft_twiddles(&mut self.registers[i], &inv_twiddles, true);
        }
    }

    pub fn extend_registers(&mut self, twiddles: &[u64]) {
        let domain_length = self.registers[0].capacity();
        for i in 0..self.max_depth() {
            debug_assert!(self.registers[i].capacity() == domain_length, "invalid register capacity");
            unsafe { self.registers[i].set_len(domain_length); }
            polys::eval_fft_twiddles(&mut self.registers[i], &twiddles, true);
        }

        for i in self.max_depth()..self.registers.len() {
            debug_assert!(self.registers[i].capacity() == domain_length, "invalid register capacity");
            unsafe { self.registers[i].set_len(domain_length); }
        }
    }

    // OPERATIONS
    // --------------------------------------------------------------------------------------------
    pub fn noop(&mut self) {
        self.copy_state(0);
        self.current_step += 1;
    }

    pub fn pull1(&mut self) {
        self.registers[0][self.current_step + 1] = self.registers[1][self.current_step];
        self.registers[1][self.current_step + 1] = self.registers[0][self.current_step];
        self.copy_state(2);
        self.current_step += 1;
    }

    pub fn pull2(&mut self) {
        self.registers[0][self.current_step + 1] = self.registers[2][self.current_step];
        self.registers[1][self.current_step + 1] = self.registers[0][self.current_step];
        self.registers[2][self.current_step + 1] = self.registers[1][self.current_step];
        self.copy_state(3);
        self.current_step += 1;
    }

    pub fn push(&mut self, value: u64) {
        self.shift_right(0, 1);
        self.registers[0][self.current_step + 1] = value;
        self.current_step += 1;
    }

    pub fn dup0(&mut self) {
        self.shift_right(0, 1);
        let value = self.registers[0][self.current_step];
        self.registers[0][self.current_step + 1] = value;
        self.current_step += 1;
    }

    pub fn dup1(&mut self) {
        self.shift_right(0, 1);
        let value = self.registers[1][self.current_step];
        self.registers[0][self.current_step + 1] = value;
        self.current_step += 1;
    }

    pub fn drop(&mut self) {
        self.shift_left(1, 1);
        self.current_step += 1;
    }

    pub fn add(&mut self) {
        let x = self.registers[0][self.current_step];
        let y = self.registers[1][self.current_step];
        self.registers[0][self.current_step + 1] = field::add(x, y);
        self.shift_left(2, 1);
        self.current_step += 1;
    }

    pub fn sub(&mut self) {
        let x = self.registers[0][self.current_step];
        let y = self.registers[1][self.current_step];
        self.registers[0][self.current_step + 1] = field::sub(y, x);
        self.shift_left(2, 1);
        self.current_step += 1;
    }

    pub fn mul(&mut self) {
        let x = self.registers[0][self.current_step];
        let y = self.registers[1][self.current_step];
        self.registers[0][self.current_step + 1] = field::mul(x, y);
        self.shift_left(2, 1);
        self.current_step += 1;
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------

    fn copy_state(&mut self, start: usize) {
        for i in start..self.registers.len() {
            let slot_value = self.registers[i][self.current_step];
            self.registers[i][self.current_step + 1] = slot_value;
        }
    }

    fn shift_left(&mut self, start: usize, pos_count: usize) {
        assert!(self.depth >= pos_count, "stack underflow at step {}", self.current_step);
        
        // shift all values by pos_count to the left
        for i in start..self.depth {
            let slot_value = self.registers[i][self.current_step];
            self.registers[i - pos_count][self.current_step + 1] = slot_value;
        }

        // set all "shifted-in" slots to 0
        for i in (self.depth - pos_count)..self.depth {
            self.registers[i][self.current_step + 1] = 0;
        }

        // stack depth has been reduced by pos_count
        self.depth -= pos_count;
    }

    fn shift_right(&mut self, start: usize, pos_count: usize) {
        
        self.depth += pos_count;
        assert!(self.depth <= MAX_STACK_DEPTH, "stack overflow at step {}", self.current_step);

        if self.depth > self.max_depth {
            self.max_depth += pos_count;
            if self.max_depth > self.registers.len() {
                self.add_registers(self.max_depth - self.registers.len());
            }
        }

        for i in start..(self.depth - pos_count) {
            let slot_value = self.registers[i][self.current_step];
            self.registers[i + pos_count][self.current_step + 1] = slot_value;
        }
    }

    /// Extends the stack by the specified number of registers
    fn add_registers(&mut self, num_registers: usize) {
        let trace_length = self.registers[0].len();
        let trace_capacity = self.registers[0].capacity();
        for _ in 0..num_registers {
            let register = utils::zero_filled_vector(trace_length, trace_capacity);
            self.registers.push(register);
        }
    }
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    
    use crate::trace::TraceState;

    const TRACE_LENGTH: usize = 16;
    const EXTENSION_FACTOR: usize = 16;

    #[test]
    fn new() {
        let mut state = TraceState::new();
        let stack = super::Stack::new(TRACE_LENGTH, EXTENSION_FACTOR);
        let expected = vec![0u64; super::MAX_STACK_DEPTH];

        assert_eq!(0, stack.depth());
        assert_eq!(0, stack.max_depth());
        assert_eq!(TRACE_LENGTH, stack.trace_length());

        stack.fill_state(&mut state, 0);
        assert_eq!(expected, state.stack.to_vec());
    }

    #[test]
    fn growth() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(TRACE_LENGTH, EXTENSION_FACTOR);
        let mut expected = vec![0u64; super::MAX_STACK_DEPTH];

        // adding to stack should grow it
        stack.push(1);
        assert_eq!(1, stack.depth());
        assert_eq!(1, stack.max_depth());
        
        stack.push(2);
        assert_eq!(2, stack.depth());
        assert_eq!(2, stack.max_depth());

        // grow the stack beyond MIN_STACK_DEPTH to make sure everything works
        stack.push(3);
        stack.push(4);
        stack.push(5);
        stack.push(6);
        stack.push(7);
        stack.push(8);
        stack.push(9);
        stack.push(10);
        assert_eq!(10, stack.depth());
        assert_eq!(10, stack.max_depth());

        expected[0..10].copy_from_slice(&[10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
        stack.fill_state(&mut state, 10);
        assert_eq!(expected, state.stack.to_vec());

        // removing from the stack should reduce the depth but not max_depth
        stack.drop();
        stack.drop();
        stack.drop();
        assert_eq!(7, stack.depth());
        assert_eq!(10, stack.max_depth());

        expected[0..10].copy_from_slice(&[7, 6, 5, 4, 3, 2, 1, 0, 0, 0]);
        stack.fill_state(&mut state, 13);
        assert_eq!(expected, state.stack.to_vec());

        // adding to stack again should increase depth but not max_depth
        stack.push(11);
        stack.push(12);
        assert_eq!(9, stack.depth());
        assert_eq!(10, stack.max_depth());

        expected[0..10].copy_from_slice(&[12, 11, 7, 6, 5, 4, 3, 2, 1, 0]);
        stack.fill_state(&mut state, 15);
        assert_eq!(expected, state.stack.to_vec());
    }

    #[test]
    fn noop() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(TRACE_LENGTH, EXTENSION_FACTOR);
        let mut expected = vec![0u64; super::MAX_STACK_DEPTH];

        stack.noop();
        stack.fill_state(&mut state, 1);
        assert_eq!(expected, state.stack);

        stack.push(1);
        stack.noop();
        stack.noop();
        stack.fill_state(&mut state, 4);
        expected[0] = 1;
        assert_eq!(expected, state.stack.to_vec());

        assert_eq!(1, stack.depth());
        assert_eq!(1, stack.max_depth());
    }

    #[test]
    fn pull1() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(TRACE_LENGTH, EXTENSION_FACTOR);
        let mut expected = vec![0u64; super::MAX_STACK_DEPTH];

        stack.push(1);
        stack.push(2);
        stack.push(3);
        stack.pull1();

        stack.fill_state(&mut state, 4);
        expected[0..3].copy_from_slice(&[2, 3, 1]);
        assert_eq!(expected, state.stack);

        assert_eq!(3, stack.depth());
        assert_eq!(3, stack.max_depth());
    }

    #[test]
    fn pull2() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(TRACE_LENGTH, EXTENSION_FACTOR);
        let mut expected = vec![0u64; super::MAX_STACK_DEPTH];

        stack.push(1);
        stack.push(2);
        stack.push(3);
        stack.push(4);
        stack.pull2();

        stack.fill_state(&mut state, 5);
        expected[0..4].copy_from_slice(&[2, 4, 3, 1]);
        assert_eq!(expected, state.stack);

        assert_eq!(4, stack.depth());
        assert_eq!(4, stack.max_depth());
    }

    #[test]
    fn push() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(TRACE_LENGTH, EXTENSION_FACTOR);
        let mut expected = vec![0u64; super::MAX_STACK_DEPTH];

        stack.push(1);
        stack.fill_state(&mut state, 1);
        expected[0] = 1;
        assert_eq!(expected, state.stack);

        stack.push(2);
        stack.fill_state(&mut state, 2);
        expected[0..2].copy_from_slice(&[2, 1]);
        assert_eq!(expected, state.stack);

        stack.push(3);
        stack.fill_state(&mut state, 3);
        expected[0..3].copy_from_slice(&[3, 2, 1]);
        assert_eq!(expected, state.stack);

        assert_eq!(3, stack.depth());
        assert_eq!(3, stack.max_depth());
    }
    
    #[test]
    fn dup0() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(TRACE_LENGTH, EXTENSION_FACTOR);
        let mut expected = vec![0u64; super::MAX_STACK_DEPTH];

        stack.push(1);
        stack.push(2);
        stack.dup0();
        stack.fill_state(&mut state, 3);
        expected[0..3].copy_from_slice(&[2, 2, 1]);
        assert_eq!(expected, state.stack);

        assert_eq!(3, stack.depth());
        assert_eq!(3, stack.max_depth());
    }

    #[test]
    fn dup1() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(TRACE_LENGTH, EXTENSION_FACTOR);
        let mut expected = vec![0u64; super::MAX_STACK_DEPTH];

        stack.push(1);
        stack.push(2);
        stack.dup1();
        stack.fill_state(&mut state, 3);
        expected[0..3].copy_from_slice(&[1, 2, 1]);
        assert_eq!(expected, state.stack);

        assert_eq!(3, stack.depth());
        assert_eq!(3, stack.max_depth());
    }

    #[test]
    fn drop() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(TRACE_LENGTH, EXTENSION_FACTOR);
        let mut expected = vec![0u64; super::MAX_STACK_DEPTH];

        stack.push(1);
        stack.push(2);
        assert_eq!(2, stack.depth());
        assert_eq!(2, stack.max_depth());

        stack.drop();
        stack.fill_state(&mut state, 3);
        expected[0] = 1;
        assert_eq!(expected, state.stack);

        assert_eq!(1, stack.depth());
        assert_eq!(2, stack.max_depth());
    }

    #[test]
    fn add() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(TRACE_LENGTH, EXTENSION_FACTOR);
        let mut expected = vec![0u64; super::MAX_STACK_DEPTH];

        stack.push(1);
        stack.push(2);
        stack.add();
        stack.fill_state(&mut state, 3);
        expected[0] = 3;
        assert_eq!(expected, state.stack);

        assert_eq!(1, stack.depth());
        assert_eq!(2, stack.max_depth());
    }

    #[test]
    fn sub() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(TRACE_LENGTH, EXTENSION_FACTOR);
        let mut expected = vec![0u64; super::MAX_STACK_DEPTH];

        stack.push(5);
        stack.push(2);
        stack.sub();
        stack.fill_state(&mut state, 3);
        expected[0] = 3;
        assert_eq!(expected, state.stack);

        assert_eq!(1, stack.depth());
        assert_eq!(2, stack.max_depth());
    }

    #[test]
    fn mul() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(TRACE_LENGTH, EXTENSION_FACTOR);
        let mut expected = vec![0u64; super::MAX_STACK_DEPTH];

        stack.push(2);
        stack.push(3);
        stack.mul();
        stack.fill_state(&mut state, 3);
        expected[0] = 6;
        assert_eq!(expected, state.stack);

        assert_eq!(1, stack.depth());
        assert_eq!(2, stack.max_depth());
    }
}