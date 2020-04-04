use crate::math::field;
use crate::utils;

// CONSTANTS
// ================================================================================================
const MIN_STACK_DEPTH: usize = 8;
const MAX_STACK_DEPTH: usize = 64;

// TYPES AND INTERFACES
// ================================================================================================
pub struct Stack {
    trace   : Vec<Vec<u64>>,
    current : usize
}

// STACK IMPLEMENTATION
// ================================================================================================
impl Stack {

    pub fn new(depth: usize, state_count: usize) -> Stack {
        assert!(depth >= MIN_STACK_DEPTH, "stack depth cannot be less than {}", MIN_STACK_DEPTH);
        assert!(depth <= MAX_STACK_DEPTH, "stack depth cannot be greater than {}", MAX_STACK_DEPTH);
        assert!(state_count.is_power_of_two(), "state count must be a power of 2");

        let current: usize = 0;
        let mut trace: Vec<Vec<u64>> = Vec::with_capacity(depth);
        for _ in 0..depth {
            let mut slot = utils::uninit_vector(state_count);
            slot[current] = 0;
            trace.push(slot);
        }
        return Stack { trace, current };
    }

    pub fn fill_row(&self, row: &mut [u64], step: usize,) {
        debug_assert!(row.len() == self.trace.len(), "row length must be equal to {}", self.trace.len());
        for i in 0..self.trace.len() {
            row[i] = self.trace[i][step];
        }
    }

    pub fn depth(&self) -> usize {
        return self.trace.len();
    }

    pub fn states(&self) -> usize {
        return self.trace[0].len();
    }

    // INSTRUCTIONS
    // --------------------------------------------------------------------------------------------
    pub fn noop(&mut self) {
        for i in 0..self.trace.len() {
            let slot_value = self.trace[i][self.current];
            self.trace[i][self.current + 1] = slot_value;
        }
        self.current += 1;
    }

    pub fn push(&mut self, value: u64) {
        self.shift_right(0, 1);
        self.trace[0][self.current + 1] = value;
        self.current += 1;
    }

    pub fn add(&mut self) {
        let x = self.trace[0][self.current];
        let y = self.trace[1][self.current];
        self.trace[0][self.current + 1] = field::add(x, y);
        self.shift_left(2, 1);
        self.current += 1;
    }

    pub fn mul(&mut self) {
        let x = self.trace[0][self.current];
        let y = self.trace[1][self.current];
        self.trace[0][self.current + 1] = field::mul(x, y);
        self.shift_left(2, 1);
        self.current += 1;
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------

    fn shift_left(&mut self, start: usize, pos_count: usize) {
        debug_assert!(start >= pos_count, "start index cannot be smaller than than pos_count");

        // shift all values by pos_count to the left
        for i in start..self.trace.len() {
            let slot_value = self.trace[i][self.current];
            self.trace[i - pos_count][self.current + 1] = slot_value;
        }

        // set all "shifted-in" slots to 0
        for i in (self.trace.len() - pos_count)..self.trace.len() {
            self.trace[i][self.current + 1] = 0;
        }
    }

    fn shift_right(&mut self, start: usize, pos_count: usize) {
        debug_assert!(pos_count < self.trace.len(), "pos_count must be smaller than stack depth");
        for i in start..(self.trace.len() - pos_count) {
            let slot_value = self.trace[i][self.current];
            self.trace[i + pos_count][self.current + 1] = slot_value;
        }
    }
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    
    const STACK_DEPTH: usize = 8;
    const STATE_COUNT: usize = 16;

    #[test]
    fn new() {
        let stack = super::Stack::new(STACK_DEPTH, STATE_COUNT);
        assert_eq!(STACK_DEPTH, stack.depth());
        assert_eq!(STATE_COUNT, stack.states());

        let mut row = [0u64; 8];
        stack.fill_row(&mut row, 0);
        assert_eq!([0, 0, 0, 0, 0, 0, 0, 0], row);
    }

    #[test]
    fn noop() {
        let mut row = [0u64; 8];
        let mut stack = super::Stack::new(STACK_DEPTH, STATE_COUNT);

        stack.noop();
        stack.fill_row(&mut row, 1);
        assert_eq!([0, 0, 0, 0, 0, 0, 0, 0], row);

        stack.push(1);
        stack.noop();
        stack.noop();
        stack.fill_row(&mut row, 4);
        assert_eq!([1, 0, 0, 0, 0, 0, 0, 0], row);
    }

    #[test]
    fn push() {
        let mut row = [0u64; 8];
        let mut stack = super::Stack::new(STACK_DEPTH, STATE_COUNT);

        stack.push(1);
        stack.fill_row(&mut row, 1);
        assert_eq!([1, 0, 0, 0, 0, 0, 0, 0], row);

        stack.push(2);
        stack.fill_row(&mut row, 2);
        assert_eq!([2, 1, 0, 0, 0, 0, 0, 0], row);

        stack.push(3);
        stack.fill_row(&mut row, 3);
        assert_eq!([3, 2, 1, 0, 0, 0, 0, 0], row);
    }

    #[test]
    fn add() {
        let mut row = [0u64; 8];
        let mut stack = super::Stack::new(STACK_DEPTH, STATE_COUNT);

        stack.push(1);
        stack.push(2);
        stack.add();
        stack.fill_row(&mut row, 3);
        assert_eq!([3, 0, 0, 0, 0, 0, 0, 0], row);
    }

    #[test]
    fn mul() {
        let mut row = [0u64; 8];
        let mut stack = super::Stack::new(STACK_DEPTH, STATE_COUNT);

        stack.push(2);
        stack.push(3);
        stack.mul();
        stack.fill_row(&mut row, 3);
        assert_eq!([6, 0, 0, 0, 0, 0, 0, 0], row);
    }
}