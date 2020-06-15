use crate::math::{ FiniteField };
use crate::stark::{ HASH_STATE_WIDTH };
use crate::utils::{ filled_vector, Hasher };
use super::{ MAX_USER_STACK_DEPTH };

// TYPES AND INTERFACES
// ================================================================================================
pub struct StackTrace<T: FiniteField + Hasher> {
    pub aux_register    : Vec<T>,
    pub user_registers  : Vec<Vec<T>>,
    pub secret_inputs_a : Vec<T>,
    pub secret_inputs_b : Vec<T>,
    pub max_depth       : usize,
    pub depth           : usize,
}

// STACK IMPLEMENTATION
// ================================================================================================
impl <T> StackTrace<T>
    where T: FiniteField + Hasher
{
    // OPERATIONS
    // --------------------------------------------------------------------------------------------
    pub fn noop(&mut self, step: usize) {
        self.copy_state(step, 0);
    }

    pub fn assert(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        let value = self.user_registers[0][step];
        assert!(value == T::ONE, "ASSERT failed at step {}", step);
        self.shift_left(step, 1, 1);
    }

    pub fn push(&mut self, step: usize, value: T) {
        self.shift_right(step, 0, 1);
        self.user_registers[0][step + 1] = value;
    }

    pub fn read(&mut self, step: usize) {
        assert!(self.secret_inputs_a.len() > 0, "ran out of secret inputs at step {}", step);
        self.shift_right(step, 0, 1);
        let value = self.secret_inputs_a.pop().unwrap();
        self.user_registers[0][step + 1] = value;
    }

    pub fn read2(&mut self, step: usize) {
        assert!(self.secret_inputs_a.len() > 0, "ran out of secret inputs at step {}", step);
        assert!(self.secret_inputs_b.len() > 0, "ran out of secret inputs at step {}", step);
        self.shift_right(step, 0, 2);
        let value_a = self.secret_inputs_a.pop().unwrap();
        let value_b = self.secret_inputs_b.pop().unwrap();
        self.user_registers[0][step + 1] = value_b;
        self.user_registers[1][step + 1] = value_a;
    }

    pub fn dup(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        self.shift_right(step, 0, 1);
        self.user_registers[0][step + 1] = self.user_registers[0][step];
    }

    pub fn dup2(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        self.shift_right(step, 0, 2);
        self.user_registers[0][step + 1] = self.user_registers[0][step];
        self.user_registers[1][step + 1] = self.user_registers[1][step];
    }

    pub fn dup4(&mut self, step: usize) {
        assert!(self.depth >= 4, "stack underflow at step {}", step);
        self.shift_right(step, 0, 4);
        self.user_registers[0][step + 1] = self.user_registers[0][step];
        self.user_registers[1][step + 1] = self.user_registers[1][step];
        self.user_registers[2][step + 1] = self.user_registers[2][step];
        self.user_registers[3][step + 1] = self.user_registers[3][step];
    }

    pub fn pad2(&mut self, step: usize) {
        self.shift_right(step, 0, 2);
        self.user_registers[0][step + 1] = T::ZERO;
        self.user_registers[1][step + 1] = T::ZERO;
    }

    pub fn drop(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        self.shift_left(step, 1, 1);
    }

    pub fn drop4(&mut self, step: usize) {
        assert!(self.depth >= 4, "stack underflow at step {}", step);
        self.shift_left(step, 4, 4);
    }

    pub fn swap(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        self.user_registers[0][step + 1] = self.user_registers[1][step];
        self.user_registers[1][step + 1] = self.user_registers[0][step];
        self.copy_state(step, 2);
    }

    pub fn swap2(&mut self, step: usize) {
        assert!(self.depth >= 4, "stack underflow at step {}", step);
        self.user_registers[0][step + 1] = self.user_registers[2][step];
        self.user_registers[1][step + 1] = self.user_registers[3][step];
        self.user_registers[2][step + 1] = self.user_registers[0][step];
        self.user_registers[3][step + 1] = self.user_registers[1][step];
        self.copy_state(step, 4);
    }

    pub fn swap4(&mut self, step: usize) {
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

    pub fn roll4(&mut self, step: usize) {
        assert!(self.depth >= 4, "stack underflow at step {}", step);
        self.user_registers[0][step + 1] = self.user_registers[3][step];
        self.user_registers[1][step + 1] = self.user_registers[0][step];
        self.user_registers[2][step + 1] = self.user_registers[1][step];
        self.user_registers[3][step + 1] = self.user_registers[2][step];
        self.copy_state(step, 4);
    }

    pub fn roll8(&mut self, step: usize) {
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

    pub fn choose(&mut self, step: usize) {
        assert!(self.depth >= 3, "stack underflow at step {}", step);
        let condition = self.user_registers[2][step];
        if condition == T::ONE {
            self.user_registers[0][step + 1] = self.user_registers[0][step];
        }
        else if condition == T::ZERO {
            self.user_registers[0][step + 1] = self.user_registers[1][step];
        }
        else {
            assert!(false, "CHOOSE on a non-binary condition at step {}", step);
        }
        self.shift_left(step, 3, 2);
    }

    pub fn choose2(&mut self, step: usize) {
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
            assert!(false, "CHOOSE2 on a non-binary condition at step {}", step);
        }
        self.shift_left(step, 6, 4);
    }

    pub fn add(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        let y = self.user_registers[1][step];
        self.user_registers[0][step + 1] = T::add(x, y);
        self.shift_left(step, 2, 1);
    }

    pub fn mul(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        let y = self.user_registers[1][step];
        self.user_registers[0][step + 1] = T::mul(x, y);
        self.shift_left(step, 2, 1);
    }

    pub fn inv(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        assert!(x != T::ZERO, "cannot compute INV of {} at step {}", T::ZERO, step);
        self.user_registers[0][step + 1] = T::inv(x);
        self.copy_state(step, 1);
    }

    pub fn neg(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        self.user_registers[0][step + 1] = T::neg(x);
        self.copy_state(step, 1);
    }

    pub fn not(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        assert!(x == T::ZERO || x == T::ONE, "cannot compute NOT of a non-binary value at step {}", step);
        self.user_registers[0][step + 1] = T::sub(T::ONE, x);
        self.copy_state(step, 1);
    }

    pub fn eq(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        let y = self.user_registers[1][step];
        if x == y {
            self.aux_register[step] = T::ONE;
            self.user_registers[0][step + 1] = T::ONE;
        } else {
            let diff = T::sub(x, y);
            self.aux_register[step] = T::inv(diff);
            self.user_registers[0][step + 1] = T::ZERO;
        }
        self.shift_left(step, 2, 1);
    }

    pub fn cmp(&mut self, step: usize) {
        assert!(self.depth >= 7, "stack underflow at step {}", step);
        assert!(self.secret_inputs_a.len() > 0, "ran out of secret inputs at step {}", step);
        assert!(self.secret_inputs_b.len() > 0, "ran out of secret inputs at step {}", step);
        let a_bit = self.secret_inputs_a.pop().unwrap();
        assert!(a_bit == T::ZERO || a_bit == T::ONE,
            "expected binary input at step {} but received: {}", step, a_bit);
        let b_bit = self.secret_inputs_b.pop().unwrap();
        assert!(b_bit == T::ZERO || b_bit == T::ONE,
            "expected binary input at step {} but received: {}", step, b_bit);

        let bit_gt = T::mul(a_bit, T::sub(T::ONE, b_bit));
        let bit_lt = T::mul(b_bit, T::sub(T::ONE, a_bit));

        let power_of_two = self.user_registers[0][step];    // TODO: make sure it is power of 2
        let gt = self.user_registers[3][step];
        let lt = self.user_registers[4][step];
        let not_set = T::mul(T::sub(T::ONE, gt), T::sub(T::ONE, lt));

        self.aux_register[step] = not_set;
        self.user_registers[0][step + 1] = T::div(power_of_two, T::from_usize(2)); // TODO: replace with shift
        self.user_registers[1][step + 1] = a_bit;
        self.user_registers[2][step + 1] = b_bit;
        self.user_registers[3][step + 1] = T::add(gt, T::mul(bit_gt, not_set));
        self.user_registers[4][step + 1] = T::add(lt, T::mul(bit_lt, not_set));
        self.user_registers[5][step + 1] = T::add(self.user_registers[5][step], T::mul(b_bit, power_of_two));
        self.user_registers[6][step + 1] = T::add(self.user_registers[6][step], T::mul(a_bit, power_of_two));

        self.copy_state(step, 7);
    }

    pub fn binacc(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        assert!(self.secret_inputs_a.len() > 0, "ran out of secret inputs at step {}", step);

        let bit = self.secret_inputs_a.pop().unwrap();
        let power_of_two = self.user_registers[0][step];    // TODO: make sure it is power of 2
        let acc = self.user_registers[1][step];

        self.aux_register[step] = bit;
        self.user_registers[0][step + 1] = T::div(power_of_two, T::from_usize(2)); // TODO: replace with shift
        self.user_registers[1][step + 1] = T::add(acc, T::mul(bit, power_of_two));

        self.copy_state(step, 2);
    }

    pub fn hashr(&mut self, step: usize) {
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