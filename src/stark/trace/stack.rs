use crate::math::field;
use crate::trace::TraceState;
use crate::utils;

// CONSTANTS
// ================================================================================================
const MIN_STACK_DEPTH: usize = 8;
const MAX_STACK_DEPTH: usize = 64;

// TYPES AND INTERFACES
// ================================================================================================
pub struct Stack {
    trace       : Vec<Vec<u64>>,
    current_step: usize,
}

// STACK IMPLEMENTATION
// ================================================================================================
impl Stack {

    pub fn new(max_depth: usize, trace_length: usize) -> Stack {
        assert!(max_depth >= MIN_STACK_DEPTH, "max stack depth cannot be less than {}", MIN_STACK_DEPTH);
        assert!(max_depth <= MAX_STACK_DEPTH, "max stack depth cannot be greater than {}", MAX_STACK_DEPTH);
        assert!(trace_length.is_power_of_two(), "trace length must be a power of 2");

        let current_step: usize = 0;
        let mut trace: Vec<Vec<u64>> = Vec::with_capacity(max_depth);
        for _ in 0..max_depth {
            let mut slot = utils::uninit_vector(trace_length);
            slot[current_step] = 0;
            trace.push(slot);
        }
        return Stack { trace, current_step };
    }

    pub fn get_register_trace(&self, index: usize) -> &[u64] {
        return &self.trace[index];
    }

    pub fn fill_state(&self, state: &mut TraceState, step: usize) {
        for i in 0..self.trace.len() {
            state.stack[i] = self.trace[i][step];
        }
    }

    pub fn max_stack_depth(&self) -> usize {
        return self.trace.len();
    }

    pub fn trace_length(&self) -> usize {
        return self.trace[0].len();
    }

    pub fn current_step(&self) -> usize {
        return self.current_step;
    }

    // OPERATIONS
    // --------------------------------------------------------------------------------------------
    pub fn noop(&mut self) {
        for i in 0..self.trace.len() {
            let slot_value = self.trace[i][self.current_step];
            self.trace[i][self.current_step + 1] = slot_value;
        }
        self.current_step += 1;
    }

    pub fn pull1(&mut self) {
        self.trace[0][self.current_step + 1] = self.trace[1][self.current_step];
        self.trace[1][self.current_step + 1] = self.trace[0][self.current_step];
        for i in 2..self.trace.len() {
            let slot_value = self.trace[i][self.current_step];
            self.trace[i][self.current_step + 1] = slot_value;
        }
        self.current_step += 1;
    }

    pub fn pull2(&mut self) {
        self.trace[0][self.current_step + 1] = self.trace[2][self.current_step];
        self.trace[1][self.current_step + 1] = self.trace[0][self.current_step];
        self.trace[2][self.current_step + 1] = self.trace[1][self.current_step];
        for i in 3..self.trace.len() {
            let slot_value = self.trace[i][self.current_step];
            self.trace[i][self.current_step + 1] = slot_value;
        }
        self.current_step += 1;
    }

    pub fn push(&mut self, value: u64) {
        self.shift_right(0, 1);
        self.trace[0][self.current_step + 1] = value;
        self.current_step += 1;
    }

    pub fn drop(&mut self) {
        self.shift_left(1, 1);
        self.current_step += 1;
    }

    pub fn dup0(&mut self) {
        self.shift_right(0, 1);
        let value = self.trace[0][self.current_step];
        self.trace[0][self.current_step + 1] = value;
        self.current_step += 1;
    }

    pub fn dup1(&mut self) {
        self.shift_right(0, 1);
        let value = self.trace[1][self.current_step];
        self.trace[0][self.current_step + 1] = value;
        self.current_step += 1;
    }

    pub fn add(&mut self) {
        let x = self.trace[0][self.current_step];
        let y = self.trace[1][self.current_step];
        self.trace[0][self.current_step + 1] = field::add(x, y);
        self.shift_left(2, 1);
        self.current_step += 1;
    }

    pub fn sub(&mut self) {
        let x = self.trace[0][self.current_step];
        let y = self.trace[1][self.current_step];
        self.trace[0][self.current_step + 1] = field::sub(y, x);
        self.shift_left(2, 1);
        self.current_step += 1;
    }

    pub fn mul(&mut self) {
        let x = self.trace[0][self.current_step];
        let y = self.trace[1][self.current_step];
        self.trace[0][self.current_step + 1] = field::mul(x, y);
        self.shift_left(2, 1);
        self.current_step += 1;
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------

    fn shift_left(&mut self, start: usize, pos_count: usize) {
        debug_assert!(start >= pos_count, "start index cannot be smaller than than pos_count");

        // shift all values by pos_count to the left
        for i in start..self.trace.len() {
            let slot_value = self.trace[i][self.current_step];
            self.trace[i - pos_count][self.current_step + 1] = slot_value;
        }

        // set all "shifted-in" slots to 0
        for i in (self.trace.len() - pos_count)..self.trace.len() {
            self.trace[i][self.current_step + 1] = 0;
        }
    }

    fn shift_right(&mut self, start: usize, pos_count: usize) {
        debug_assert!(pos_count < self.trace.len(), "pos_count must be smaller than stack depth");
        for i in start..(self.trace.len() - pos_count) {
            let slot_value = self.trace[i][self.current_step];
            self.trace[i + pos_count][self.current_step + 1] = slot_value;
        }
    }
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    
    use crate::trace::TraceState;

    const STACK_DEPTH: usize = 8;
    const STATE_COUNT: usize = 16;

    #[test]
    fn new() {
        let stack = super::Stack::new(STACK_DEPTH, STATE_COUNT);
        assert_eq!(STACK_DEPTH, stack.max_stack_depth());
        assert_eq!(STATE_COUNT, stack.trace_length());

        let mut state = TraceState::new();
        stack.fill_state(&mut state, 0);
        assert_eq!([0u64; 32], state.stack);
    }

    #[test]
    fn noop() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(STACK_DEPTH, STATE_COUNT);
        let mut expected = [0u64; 32];

        stack.noop();
        stack.fill_state(&mut state, 1);
        assert_eq!(expected, state.stack);

        stack.push(1);
        stack.noop();
        stack.noop();
        stack.fill_state(&mut state, 4);
        expected[0] = 1;
        assert_eq!(expected, state.stack);
    }

    #[test]
    fn push() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(STACK_DEPTH, STATE_COUNT);
        let mut expected = [0u64; 32];

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
    }

    #[test]
    fn drop() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(STACK_DEPTH, STATE_COUNT);
        let mut expected = [0u64; 32];

        stack.push(1);
        stack.push(2);
        stack.drop();
        stack.fill_state(&mut state, 3);
        expected[0] = 1;
        assert_eq!(expected, state.stack);
    }

    
    #[test]
    fn dup0() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(STACK_DEPTH, STATE_COUNT);
        let mut expected = [0u64; 32];

        stack.push(1);
        stack.push(2);
        stack.dup0();
        stack.fill_state(&mut state, 3);
        expected[0..3].copy_from_slice(&[2, 2, 1]);
        assert_eq!(expected, state.stack);
    }

    #[test]
    fn dup1() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(STACK_DEPTH, STATE_COUNT);
        let mut expected = [0u64; 32];

        stack.push(1);
        stack.push(2);
        stack.dup1();
        stack.fill_state(&mut state, 3);
        expected[0..3].copy_from_slice(&[1, 2, 1]);
        assert_eq!(expected, state.stack);
    }

    #[test]
    fn add() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(STACK_DEPTH, STATE_COUNT);
        let mut expected = [0u64; 32];

        stack.push(1);
        stack.push(2);
        stack.add();
        stack.fill_state(&mut state, 3);
        expected[0] = 3;
        assert_eq!(expected, state.stack);
    }

    #[test]
    fn sub() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(STACK_DEPTH, STATE_COUNT);
        let mut expected = [0u64; 32];

        stack.push(5);
        stack.push(2);
        stack.sub();
        stack.fill_state(&mut state, 3);
        expected[0] = 3;
        assert_eq!(expected, state.stack);
    }

    #[test]
    fn mul() {
        let mut state = TraceState::new();
        let mut stack = super::Stack::new(STACK_DEPTH, STATE_COUNT);
        let mut expected = [0u64; 32];

        stack.push(2);
        stack.push(3);
        stack.mul();
        stack.fill_state(&mut state, 3);
        expected[0] = 6;
        assert_eq!(expected, state.stack);
    }
}