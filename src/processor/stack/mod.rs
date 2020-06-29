use crate::math::{ FiniteField, F128 };
use crate::utils::{ filled_vector, hasher };
use crate::{ HASH_STATE_WIDTH, MIN_STACK_DEPTH, MAX_STACK_DEPTH };
use super::{ ProgramInputs, opcodes, ExecutionHint };

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================
const MIN_USER_STACK_DEPTH: usize = MIN_STACK_DEPTH - 1;
const MAX_USER_STACK_DEPTH: usize = MAX_STACK_DEPTH - 1;

// TYPES AND INTERFACES
// ================================================================================================
pub struct Stack {
    aux_register    : Vec<u128>,
    user_registers  : Vec<Vec<u128>>,
    tape_a          : Vec<u128>,
    tape_b          : Vec<u128>,
    max_depth       : usize,
    depth           : usize,
}

// STACK IMPLEMENTATION
// ================================================================================================
impl Stack {

    /// Returns a new Stack with enough memory allocated for each register to hold trace lengths
    /// of `init_trace_length` steps. Register traces will be expanded dynamically if the number
    /// of actual steps exceeds this initial setting.
    pub fn new(inputs: &ProgramInputs<u128>, init_trace_length: usize) -> Stack {

        // allocate space for user register traces and initialize the first state with
        // public inputs
        let public_inputs = inputs.get_public_inputs();
        let init_stack_depth = std::cmp::max(public_inputs.len(), MIN_USER_STACK_DEPTH);
        let mut user_registers: Vec<Vec<u128>> = Vec::with_capacity(init_stack_depth);
        for i in 0..init_stack_depth {
            let mut register = vec![F128::ZERO; init_trace_length];
            if i < public_inputs.len() { 
                register[0] = public_inputs[i];
            }
            user_registers.push(register);
        }

        // allocate space for aux stack register
        let aux_register = vec![F128::ZERO; init_trace_length];

        // reverse secret inputs so that they are consumed in FIFO order
        let [secret_inputs_a, secret_inputs_b] = inputs.get_secret_inputs();
        let mut tape_a = secret_inputs_a.clone();
        tape_a.reverse();
        let mut tape_b = secret_inputs_b.clone();
        tape_b.reverse();

        return Stack {
            aux_register,
            user_registers,
            tape_a,
            tape_b,
            max_depth: public_inputs.len(),
            depth: public_inputs.len()
        };
    }

    /// Executes `current_op` against the state of the stack specified by `step` parameter;
    /// if `current_op` is a PUSH, `next_op` will be pushed onto the stack.
    pub fn execute(&mut self, current_op: u128, next_op: u128, hint: ExecutionHint, step: usize) {

        // make sure current_op contains a valid opcode
        let op_code = current_op as u8;
        assert!((op_code as u128) == current_op, "opcode {} is invalid", current_op);

        // make sure there is enough space to update current step
        self.ensure_trace_capacity(step);

        // execute the appropriate action against the current state of the stack
        match op_code {

            opcodes::BEGIN   => self.op_noop(step),
            opcodes::NOOP    => self.op_noop(step),
            opcodes::ASSERT  => self.op_assert(step),

            opcodes::PUSH    => self.op_push(step, next_op),

            opcodes::READ    => self.op_read(step),
            opcodes::READ2   => self.op_read2(step),

            opcodes::DUP     => self.op_dup(step),
            opcodes::DUP2    => self.op_dup2(step),
            opcodes::DUP4    => self.op_dup4(step),
            opcodes::PAD2    => self.op_pad2(step),

            opcodes::DROP    => self.op_drop(step),
            opcodes::DROP4   => self.op_drop4(step),

            opcodes::SWAP    => self.op_swap(step),
            opcodes::SWAP2   => self.op_swap2(step),
            opcodes::SWAP4   => self.op_swap4(step),

            opcodes::ROLL4   => self.op_roll4(step),
            opcodes::ROLL8   => self.op_roll8(step),

            opcodes::CHOOSE  => self.op_choose(step),
            opcodes::CHOOSE2 => self.op_choose2(step),

            opcodes::ADD     => self.op_add(step),
            opcodes::MUL     => self.op_mul(step),
            opcodes::INV     => self.op_inv(step),
            opcodes::NEG     => self.op_neg(step),
            opcodes::NOT     => self.op_not(step),

            opcodes::EQ      => self.op_eq(step),
            opcodes::CMP     => self.op_cmp(step, hint),
            opcodes::BINACC  => self.op_binacc(step, hint),

            opcodes::RESCR   => self.op_rescr(step),

            _ => panic!("operation {} is not supported", current_op)
        }
    }

    /// Returns the value at the top of the stack at specified `step`.
    pub fn get_stack_top(&self, step: usize) -> u128 {
        return self.user_registers[0][step];
    }

    /// Merges all register traces into a single vector of traces.
    pub fn into_register_traces(mut self) -> Vec<Vec<u128>> {
        self.user_registers.truncate(self.max_depth);
        let mut registers = Vec::with_capacity(1 + self.user_registers.len());
        registers.push(self.aux_register);
        registers.append(&mut self.user_registers);
        return registers;
    }

    // OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn op_noop(&mut self, step: usize) {
        self.copy_state(step, 0);
    }

    fn op_assert(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        let value = self.user_registers[0][step];
        assert!(value == F128::ONE, "ASSERT failed at step {}", step);
        self.shift_left(step, 1, 1);
    }

    fn op_push(&mut self, step: usize, value: F128) {
        self.shift_right(step, 0, 1);
        self.user_registers[0][step + 1] = value;
    }

    fn op_read(&mut self, step: usize) {
        assert!(self.tape_a.len() > 0, "attempt to read from empty tape A at step {}", step);
        self.shift_right(step, 0, 1);
        let value = self.tape_a.pop().unwrap();
        self.user_registers[0][step + 1] = value;
    }

    fn op_read2(&mut self, step: usize) {
        assert!(self.tape_a.len() > 0, "attempt to read from empty tape A at step {}", step);
        assert!(self.tape_b.len() > 0, "attempt to read from empty tape B at step {}", step);
        self.shift_right(step, 0, 2);
        let value_a = self.tape_a.pop().unwrap();
        let value_b = self.tape_b.pop().unwrap();
        self.user_registers[0][step + 1] = value_b;
        self.user_registers[1][step + 1] = value_a;
    }

    fn op_dup(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        self.shift_right(step, 0, 1);
        self.user_registers[0][step + 1] = self.user_registers[0][step];
    }

    fn op_dup2(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        self.shift_right(step, 0, 2);
        self.user_registers[0][step + 1] = self.user_registers[0][step];
        self.user_registers[1][step + 1] = self.user_registers[1][step];
    }

    fn op_dup4(&mut self, step: usize) {
        assert!(self.depth >= 4, "stack underflow at step {}", step);
        self.shift_right(step, 0, 4);
        self.user_registers[0][step + 1] = self.user_registers[0][step];
        self.user_registers[1][step + 1] = self.user_registers[1][step];
        self.user_registers[2][step + 1] = self.user_registers[2][step];
        self.user_registers[3][step + 1] = self.user_registers[3][step];
    }

    fn op_pad2(&mut self, step: usize) {
        self.shift_right(step, 0, 2);
        self.user_registers[0][step + 1] = F128::ZERO;
        self.user_registers[1][step + 1] = F128::ZERO;
    }

    fn op_drop(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        self.shift_left(step, 1, 1);
    }

    fn op_drop4(&mut self, step: usize) {
        assert!(self.depth >= 4, "stack underflow at step {}", step);
        self.shift_left(step, 4, 4);
    }

    fn op_swap(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        self.user_registers[0][step + 1] = self.user_registers[1][step];
        self.user_registers[1][step + 1] = self.user_registers[0][step];
        self.copy_state(step, 2);
    }

    fn op_swap2(&mut self, step: usize) {
        assert!(self.depth >= 4, "stack underflow at step {}", step);
        self.user_registers[0][step + 1] = self.user_registers[2][step];
        self.user_registers[1][step + 1] = self.user_registers[3][step];
        self.user_registers[2][step + 1] = self.user_registers[0][step];
        self.user_registers[3][step + 1] = self.user_registers[1][step];
        self.copy_state(step, 4);
    }

    fn op_swap4(&mut self, step: usize) {
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

    fn op_roll4(&mut self, step: usize) {
        assert!(self.depth >= 4, "stack underflow at step {}", step);
        self.user_registers[0][step + 1] = self.user_registers[3][step];
        self.user_registers[1][step + 1] = self.user_registers[0][step];
        self.user_registers[2][step + 1] = self.user_registers[1][step];
        self.user_registers[3][step + 1] = self.user_registers[2][step];
        self.copy_state(step, 4);
    }

    fn op_roll8(&mut self, step: usize) {
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

    fn op_choose(&mut self, step: usize) {
        assert!(self.depth >= 3, "stack underflow at step {}", step);
        let condition = self.user_registers[2][step];
        if condition == F128::ONE {
            self.user_registers[0][step + 1] = self.user_registers[0][step];
        }
        else if condition == F128::ZERO {
            self.user_registers[0][step + 1] = self.user_registers[1][step];
        }
        else {
            assert!(false, "CHOOSE on a non-binary condition at step {}", step);
        }
        self.shift_left(step, 3, 2);
    }

    fn op_choose2(&mut self, step: usize) {
        assert!(self.depth >= 6, "stack underflow at step {}", step);
        let condition = self.user_registers[4][step];
        if condition == F128::ONE {
            self.user_registers[0][step + 1] = self.user_registers[0][step];
            self.user_registers[1][step + 1] = self.user_registers[1][step];
        }
        else if condition == F128::ZERO {
            self.user_registers[0][step + 1] = self.user_registers[2][step];
            self.user_registers[1][step + 1] = self.user_registers[3][step];
        }
        else {
            assert!(false, "CHOOSE2 on a non-binary condition at step {}", step);
        }
        self.shift_left(step, 6, 4);
    }

    fn op_add(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        let y = self.user_registers[1][step];
        self.user_registers[0][step + 1] = F128::add(x, y);
        self.shift_left(step, 2, 1);
    }

    fn op_mul(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        let y = self.user_registers[1][step];
        self.user_registers[0][step + 1] = F128::mul(x, y);
        self.shift_left(step, 2, 1);
    }

    fn op_inv(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        assert!(x != F128::ZERO, "cannot compute INV of {} at step {}", F128::ZERO, step);
        self.user_registers[0][step + 1] = F128::inv(x);
        self.copy_state(step, 1);
    }

    fn op_neg(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        self.user_registers[0][step + 1] = F128::neg(x);
        self.copy_state(step, 1);
    }

    fn op_not(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        assert!(x == F128::ZERO || x == F128::ONE, "cannot compute NOT of a non-binary value at step {}", step);
        self.user_registers[0][step + 1] = F128::sub(F128::ONE, x);
        self.copy_state(step, 1);
    }

    fn op_eq(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        let y = self.user_registers[1][step];
        if x == y {
            self.aux_register[step] = F128::ONE;
            self.user_registers[0][step + 1] = F128::ONE;
        } else {
            let diff = F128::sub(x, y);
            self.aux_register[step] = F128::inv(diff);
            self.user_registers[0][step + 1] = F128::ZERO;
        }
        self.shift_left(step, 2, 1);
    }

    fn op_cmp(&mut self, step: usize, hint: ExecutionHint) {
        // process execution hint
        match hint {
            ExecutionHint::CmpStart(n) => {
                // if we are about to start comparison sequence, push binary decompositions
                // of a and b values onto the tapes
                assert!(self.depth >= 9, "stack underflow at step {}", step);
                let a_val = self.user_registers[7][step];
                let b_val = self.user_registers[8][step];
                for i in 0..n {
                    self.tape_a.push((a_val >> i) & 1);
                    self.tape_b.push((b_val >> i) & 1);
                }
            },
            _ => {
                assert!(self.depth >= 7, "stack underflow at step {}", step);
                assert!(self.tape_a.len() > 0, "attempt to read from empty tape A at step {}", step);
                assert!(self.tape_b.len() > 0, "attempt to read from empty tape B at step {}", step);
            }
        }

        // get next bits of a and b values from the tapes
        let a_bit = self.tape_a.pop().unwrap();
        assert!(a_bit == F128::ZERO || a_bit == F128::ONE,
            "expected binary input at step {} but received: {}", step, a_bit);
        let b_bit = self.tape_b.pop().unwrap();
        assert!(b_bit == F128::ZERO || b_bit == F128::ONE,
            "expected binary input at step {} but received: {}", step, b_bit);

        // determine which bit is greater
        let bit_gt = F128::mul(a_bit, F128::sub(F128::ONE, b_bit));
        let bit_lt = F128::mul(b_bit, F128::sub(F128::ONE, a_bit));

        // compute current power of 2 for binary decomposition
        let power_of_two = self.user_registers[0][step];
        assert!(power_of_two.is_power_of_two(),
            "expected top of the stack at step {} to be a power of 2, but received {}", step, power_of_two);
        let next_power_of_two = if power_of_two == 1 {
            F128::div(power_of_two, F128::from_usize(2))
        }
        else {
            power_of_two >> 1
        };

        // determine if the result of comparison is already known
        let gt = self.user_registers[3][step];
        let lt = self.user_registers[4][step];
        let not_set = F128::mul(F128::sub(F128::ONE, gt), F128::sub(F128::ONE, lt));

        // update the next state of the computation
        self.aux_register[step] = not_set;
        self.user_registers[0][step + 1] = next_power_of_two;
        self.user_registers[1][step + 1] = a_bit;
        self.user_registers[2][step + 1] = b_bit;
        self.user_registers[3][step + 1] = F128::add(gt, F128::mul(bit_gt, not_set));
        self.user_registers[4][step + 1] = F128::add(lt, F128::mul(bit_lt, not_set));
        self.user_registers[5][step + 1] = F128::add(self.user_registers[5][step], F128::mul(b_bit, power_of_two));
        self.user_registers[6][step + 1] = F128::add(self.user_registers[6][step], F128::mul(a_bit, power_of_two));

        self.copy_state(step, 7);
    }

    fn op_binacc(&mut self, step: usize, hint: ExecutionHint) {
        // process execution hint
        match hint {
            ExecutionHint::RcStart(n) => {
                // if we are about to start range check sequence, push binary decompositions
                // of the value onto tape A
                assert!(self.depth >= 3, "stack underflow at step {}", step);
                let val = self.user_registers[2][step];
                for i in 0..n {
                    self.tape_a.push((val >> i) & 1);
                }
            },
            _ => {
                assert!(self.depth >= 2, "stack underflow at step {}", step);
                assert!(self.tape_a.len() > 0, "attempt to read from empty tape A at step {}", step);
            }
        }

        // get the next bit of the value from tape A
        let bit = self.tape_a.pop().unwrap();
        assert!(bit == F128::ZERO || bit == F128::ONE,
            "expected binary input at step {} but received: {}", step, bit);

        // compute current power of 2 for binary decomposition
        let power_of_two = self.user_registers[0][step];
        assert!(power_of_two.is_power_of_two(),
            "expected top of the stack at step {} to be a power of 2, but received {}", step, power_of_two);
        let next_power_of_two = if power_of_two == 1 {
                F128::div(power_of_two, F128::from_usize(2))
            }
            else {
                power_of_two >> 1
            };

        let acc = self.user_registers[1][step];

        // update the next state of the computation
        self.aux_register[step] = bit;
        self.user_registers[0][step + 1] = next_power_of_two;
        self.user_registers[1][step + 1] = F128::add(acc, F128::mul(bit, power_of_two));

        self.copy_state(step, 2);
    }

    fn op_rescr(&mut self, step: usize) {
        assert!(self.depth >= HASH_STATE_WIDTH, "stack underflow at step {}", step);
        let mut state = [
            self.user_registers[0][step],
            self.user_registers[1][step],
            self.user_registers[2][step],
            self.user_registers[3][step],
            self.user_registers[4][step],
            self.user_registers[5][step],
        ];

        hasher::apply_round(&mut state, step);

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
            self.user_registers[i][step + 1] = F128::ZERO;
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
            let register = filled_vector(trace_length, trace_capacity, F128::ZERO);
            self.user_registers.push(register);
        }
    }

    fn ensure_trace_capacity(&mut self, step: usize) {
        if step >= self.aux_register.len() - 1 {
            let new_length = self.aux_register.len() * 2;
            self.aux_register.resize(new_length, 0);
            for i in 0..self.user_registers.len() {
                self.user_registers[i].resize(new_length, 0);
            }
        }
    }
}