use crate::math::{ field };
use crate::utils::{ filled_vector, hasher };
use crate::{ HASH_STATE_WIDTH, MIN_STACK_DEPTH, MAX_STACK_DEPTH };
use super::{ ProgramInputs, opcodes, ExecutionHint };

#[cfg(test)]
mod tests;

#[cfg(test)]
pub mod stack2;

// TYPES AND INTERFACES
// ================================================================================================
pub struct Stack {
    registers   : Vec<Vec<u128>>,
    tape_a      : Vec<u128>,
    tape_b      : Vec<u128>,
    max_depth   : usize,
    depth       : usize,
}

// STACK IMPLEMENTATION
// ================================================================================================
impl Stack {

    /// Returns a new Stack with enough memory allocated for each register to hold trace lengths
    /// of `init_trace_length` steps. Register traces will be expanded dynamically if the number
    /// of actual steps exceeds this initial setting.
    pub fn new(inputs: &ProgramInputs, init_trace_length: usize) -> Stack {

        // allocate space for register traces and initialize the first state with public inputs
        let public_inputs = inputs.get_public_inputs();
        let init_stack_depth = std::cmp::max(public_inputs.len(), MIN_STACK_DEPTH);
        let mut registers: Vec<Vec<u128>> = Vec::with_capacity(init_stack_depth);
        for i in 0..init_stack_depth {
            let mut register = vec![field::ZERO; init_trace_length];
            if i < public_inputs.len() { 
                register[0] = public_inputs[i];
            }
            registers.push(register);
        }

        // reverse secret inputs so that they are consumed in FIFO order
        let [secret_inputs_a, secret_inputs_b] = inputs.get_secret_inputs();
        let mut tape_a = secret_inputs_a.clone();
        tape_a.reverse();
        let mut tape_b = secret_inputs_b.clone();
        tape_b.reverse();

        return Stack {
            registers,
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

            opcodes::BEGIN      => self.op_noop(step),
            opcodes::NOOP       => self.op_noop(step),
            opcodes::ASSERT     => self.op_assert(step),
            opcodes::ASSERTEQ   => self.op_asserteq(step),

            opcodes::PUSH       => self.op_push(step, next_op),

            opcodes::READ       => self.op_read(step, hint),
            opcodes::READ2      => self.op_read2(step),

            opcodes::DUP        => self.op_dup(step),
            opcodes::DUP2       => self.op_dup2(step),
            opcodes::DUP4       => self.op_dup4(step),
            opcodes::PAD2       => self.op_pad2(step),

            opcodes::DROP       => self.op_drop(step),
            opcodes::DROP4      => self.op_drop4(step),

            opcodes::SWAP       => self.op_swap(step),
            opcodes::SWAP2      => self.op_swap2(step),
            opcodes::SWAP4      => self.op_swap4(step),

            opcodes::ROLL4      => self.op_roll4(step),
            opcodes::ROLL8      => self.op_roll8(step),

            opcodes::CHOOSE     => self.op_choose(step),
            opcodes::CHOOSE2    => self.op_choose2(step),

            opcodes::ADD        => self.op_add(step),
            opcodes::MUL        => self.op_mul(step),
            opcodes::INV        => self.op_inv(step),
            opcodes::NEG        => self.op_neg(step),
            opcodes::NOT        => self.op_not(step),
            opcodes::AND        => self.op_and(step),
            opcodes::OR         => self.op_or(step),

            opcodes::EQ         => self.op_eq(step),
            opcodes::CMP        => self.op_cmp(step, hint),
            opcodes::BINACC     => self.op_binacc(step, hint),

            opcodes::RESCR      => self.op_rescr(step),

            _ => panic!("operation {} is not supported", current_op)
        }
    }

    /// Returns the value at the top of the stack at specified `step`.
    pub fn get_stack_top(&self, step: usize) -> u128 {
        return self.registers[0][step];
    }

    /// Merges all register traces into a single vector of traces.
    pub fn into_register_traces(mut self) -> Vec<Vec<u128>> {
        self.registers.truncate(self.max_depth);
        return self.registers;
    }

    // FLOW CONTROL OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn op_noop(&mut self, step: usize) {
        self.copy_state(step, 0);
    }

    fn op_assert(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        let value = self.registers[0][step];
        assert!(value == field::ONE, "ASSERT failed at step {}", step);
        self.shift_left(step, 1, 1);
    }

    fn op_asserteq(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        let x = self.registers[0][step];
        let y = self.registers[1][step];
        assert!(x == y, "ASSERTEQ failed at step {}", step);
        self.shift_left(step, 2, 2);
    }

    // INPUT OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn op_push(&mut self, step: usize, value: u128) {
        self.shift_right(step, 0, 1);
        self.registers[0][step + 1] = value;
    }

    fn op_read(&mut self, step: usize, hint: ExecutionHint) {
        // process execution hint
        match hint {
            ExecutionHint::EqStart => {
                // if we are about to equality comparison sequence, push inverse of the difference
                // between top two stack values onto secret tape A, if they are equal; otherwise
                // push value 1
                assert!(self.depth >= 2, "stack underflow at step {}", step);
                let x = self.registers[0][step];
                let y = self.registers[1][step];
                if x == y {
                    self.tape_a.push(field::ONE);
                }
                else {
                    self.tape_a.push(field::inv(field::sub(x, y)));
                }
            },
            _ => {
                assert!(self.tape_a.len() > 0, "attempt to read from empty tape A at step {}", step);
            }
        }

        self.shift_right(step, 0, 1);
        let value = self.tape_a.pop().unwrap();
        self.registers[0][step + 1] = value;
    }

    fn op_read2(&mut self, step: usize) {
        assert!(self.tape_a.len() > 0, "attempt to read from empty tape A at step {}", step);
        assert!(self.tape_b.len() > 0, "attempt to read from empty tape B at step {}", step);
        self.shift_right(step, 0, 2);
        let value_a = self.tape_a.pop().unwrap();
        let value_b = self.tape_b.pop().unwrap();
        self.registers[0][step + 1] = value_b;
        self.registers[1][step + 1] = value_a;
    }

    // STACK MANIPULATION OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn op_dup(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        self.shift_right(step, 0, 1);
        self.registers[0][step + 1] = self.registers[0][step];
    }

    fn op_dup2(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        self.shift_right(step, 0, 2);
        self.registers[0][step + 1] = self.registers[0][step];
        self.registers[1][step + 1] = self.registers[1][step];
    }

    fn op_dup4(&mut self, step: usize) {
        assert!(self.depth >= 4, "stack underflow at step {}", step);
        self.shift_right(step, 0, 4);
        self.registers[0][step + 1] = self.registers[0][step];
        self.registers[1][step + 1] = self.registers[1][step];
        self.registers[2][step + 1] = self.registers[2][step];
        self.registers[3][step + 1] = self.registers[3][step];
    }

    fn op_pad2(&mut self, step: usize) {
        self.shift_right(step, 0, 2);
        self.registers[0][step + 1] = field::ZERO;
        self.registers[1][step + 1] = field::ZERO;
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
        self.registers[0][step + 1] = self.registers[1][step];
        self.registers[1][step + 1] = self.registers[0][step];
        self.copy_state(step, 2);
    }

    fn op_swap2(&mut self, step: usize) {
        assert!(self.depth >= 4, "stack underflow at step {}", step);
        self.registers[0][step + 1] = self.registers[2][step];
        self.registers[1][step + 1] = self.registers[3][step];
        self.registers[2][step + 1] = self.registers[0][step];
        self.registers[3][step + 1] = self.registers[1][step];
        self.copy_state(step, 4);
    }

    fn op_swap4(&mut self, step: usize) {
        assert!(self.depth >= 8, "stack underflow at step {}", step);
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

    fn op_roll4(&mut self, step: usize) {
        assert!(self.depth >= 4, "stack underflow at step {}", step);
        self.registers[0][step + 1] = self.registers[3][step];
        self.registers[1][step + 1] = self.registers[0][step];
        self.registers[2][step + 1] = self.registers[1][step];
        self.registers[3][step + 1] = self.registers[2][step];
        self.copy_state(step, 4);
    }

    fn op_roll8(&mut self, step: usize) {
        assert!(self.depth >= 8, "stack underflow at step {}", step);
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

    // SELECTION OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn op_choose(&mut self, step: usize) {
        assert!(self.depth >= 3, "stack underflow at step {}", step);
        let condition = self.registers[2][step];
        if condition == field::ONE {
            self.registers[0][step + 1] = self.registers[0][step];
        }
        else if condition == field::ZERO {
            self.registers[0][step + 1] = self.registers[1][step];
        }
        else {
            assert!(false, "CHOOSE on a non-binary condition at step {}", step);
        }
        self.shift_left(step, 3, 2);
    }

    fn op_choose2(&mut self, step: usize) {
        assert!(self.depth >= 6, "stack underflow at step {}", step);
        let condition = self.registers[4][step];
        if condition == field::ONE {
            self.registers[0][step + 1] = self.registers[0][step];
            self.registers[1][step + 1] = self.registers[1][step];
        }
        else if condition == field::ZERO {
            self.registers[0][step + 1] = self.registers[2][step];
            self.registers[1][step + 1] = self.registers[3][step];
        }
        else {
            assert!(false, "CHOOSE2 on a non-binary condition at step {}", step);
        }
        self.shift_left(step, 6, 4);
    }

    // ARITHMETIC AND BOOLEAN OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn op_add(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        let x = self.registers[0][step];
        let y = self.registers[1][step];
        self.registers[0][step + 1] = field::add(x, y);
        self.shift_left(step, 2, 1);
    }

    fn op_mul(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        let x = self.registers[0][step];
        let y = self.registers[1][step];
        self.registers[0][step + 1] = field::mul(x, y);
        self.shift_left(step, 2, 1);
    }

    fn op_inv(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        let x = self.registers[0][step];
        assert!(x != field::ZERO, "cannot compute INV of {} at step {}", field::ZERO, step);
        self.registers[0][step + 1] = field::inv(x);
        self.copy_state(step, 1);
    }

    fn op_neg(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        let x = self.registers[0][step];
        self.registers[0][step + 1] = field::neg(x);
        self.copy_state(step, 1);
    }

    fn op_not(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        let x = self.registers[0][step];
        assert!(is_binary(x), "cannot compute NOT of a non-binary value at step {}", step);
        self.registers[0][step + 1] = field::sub(field::ONE, x);
        self.copy_state(step, 1);
    }

    fn op_and(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        let x = self.registers[0][step];
        let y = self.registers[1][step];
        assert!(is_binary(x), "cannot compute AND for a non-binary value at step {}", step);
        assert!(is_binary(y), "cannot compute AND for a non-binary value at step {}", step);

        self.registers[0][step + 1] = if x == field::ONE && y == field::ONE { 1 } else { 0 };
        self.shift_left(step, 2, 1);
    }

    fn op_or(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        let x = self.registers[0][step];
        let y = self.registers[1][step];
        assert!(is_binary(x), "cannot compute OR for a non-binary value at step {}", step);
        assert!(is_binary(y), "cannot compute OR for a non-binary value at step {}", step);

        self.registers[0][step + 1] = if x == field::ONE || y == field::ONE { 1 } else { 0 };
        self.shift_left(step, 2, 1);
    }


    // COMPARISON OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn op_eq(&mut self, step: usize) {
        assert!(self.depth >= 3, "stack underflow at step {}", step);
        let aux = self.registers[0][step];
        let x = self.registers[1][step];
        let y = self.registers[2][step];
        if x == y {
            self.registers[0][step + 1] = field::ONE;
        } else {
            let diff = field::sub(x, y);
            assert!(aux == field::inv(diff), "invalid AUX value for EQ operation at step {}", step);
            self.registers[0][step + 1] = field::ZERO;
        }
        self.shift_left(step, 3, 2);
    }

    fn op_cmp(&mut self, step: usize, hint: ExecutionHint) {
        // process execution hint
        match hint {
            ExecutionHint::CmpStart(n) => {
                // if we are about to start comparison sequence, push binary decompositions
                // of a and b values onto the tapes
                assert!(self.depth >= 10, "stack underflow at step {}", step);
                let a_val = self.registers[8][step];
                let b_val = self.registers[9][step];
                for i in 0..n {
                    self.tape_a.push((a_val >> i) & 1);
                    self.tape_b.push((b_val >> i) & 1);
                }
            },
            _ => {
                assert!(self.depth >= 8, "stack underflow at step {}", step);
                assert!(self.tape_a.len() > 0, "attempt to read from empty tape A at step {}", step);
                assert!(self.tape_b.len() > 0, "attempt to read from empty tape B at step {}", step);
            }
        }

        // get next bits of a and b values from the tapes
        let a_bit = self.tape_a.pop().unwrap();
        assert!(a_bit == field::ZERO || a_bit == field::ONE,
            "expected binary input at step {} but received: {}", step, a_bit);
        let b_bit = self.tape_b.pop().unwrap();
        assert!(b_bit == field::ZERO || b_bit == field::ONE,
            "expected binary input at step {} but received: {}", step, b_bit);

        // determine which bit is greater
        let bit_gt = field::mul(a_bit, field::sub(field::ONE, b_bit));
        let bit_lt = field::mul(b_bit, field::sub(field::ONE, a_bit));

        // compute current power of 2 for binary decomposition
        let power_of_two = self.registers[0][step];
        assert!(power_of_two.is_power_of_two(),
            "expected top of the stack at step {} to be a power of 2, but received {}", step, power_of_two);
        let next_power_of_two = if power_of_two == 1 {
            field::div(power_of_two, 2)
        }
        else {
            power_of_two >> 1
        };

        // determine if the result of comparison is already known
        let gt = self.registers[4][step];
        let lt = self.registers[5][step];
        let not_set = field::mul(field::sub(field::ONE, gt), field::sub(field::ONE, lt));

        // update the next state of the computation
        self.registers[0][step + 1] = next_power_of_two;
        self.registers[1][step + 1] = a_bit;
        self.registers[2][step + 1] = b_bit;
        self.registers[3][step + 1] = not_set;
        self.registers[4][step + 1] = field::add(gt, field::mul(bit_gt, not_set));
        self.registers[5][step + 1] = field::add(lt, field::mul(bit_lt, not_set));
        self.registers[6][step + 1] = field::add(self.registers[6][step], field::mul(b_bit, power_of_two));
        self.registers[7][step + 1] = field::add(self.registers[7][step], field::mul(a_bit, power_of_two));

        self.copy_state(step, 8);
    }

    fn op_binacc(&mut self, step: usize, hint: ExecutionHint) {
        // process execution hint
        match hint {
            ExecutionHint::RcStart(n) => {
                // if we are about to start range check sequence, push binary decompositions
                // of the value onto tape A
                assert!(self.depth >= 4, "stack underflow at step {}", step);
                let val = self.registers[3][step];
                for i in 0..n {
                    self.tape_a.push((val >> i) & 1);
                }
            },
            _ => {
                assert!(self.depth >= 3, "stack underflow at step {}", step);
                assert!(self.tape_a.len() > 0, "attempt to read from empty tape A at step {}", step);
            }
        }

        // get the next bit of the value from tape A
        let bit = self.tape_a.pop().unwrap();
        assert!(bit == field::ZERO || bit == field::ONE,
            "expected binary input at step {} but received: {}", step, bit);

        // compute current power of 2 for binary decomposition
        let power_of_two = self.registers[0][step];
        assert!(power_of_two.is_power_of_two(),
            "expected top of the stack at step {} to be a power of 2, but received {}", step, power_of_two);
        let next_power_of_two = if power_of_two == 1 {
                field::div(power_of_two, 2)
            }
            else {
                power_of_two >> 1
            };

        let acc = self.registers[2][step];

        // update the next state of the computation
        self.registers[0][step + 1] = next_power_of_two;
        self.registers[1][step + 1] = bit;
        self.registers[2][step + 1] = field::add(acc, field::mul(bit, power_of_two));

        self.copy_state(step, 3);
    }

    // CRYPTOGRAPHIC OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn op_rescr(&mut self, step: usize) {
        assert!(self.depth >= HASH_STATE_WIDTH, "stack underflow at step {}", step);
        let mut state = [
            self.registers[0][step],
            self.registers[1][step],
            self.registers[2][step],
            self.registers[3][step],
            self.registers[4][step],
            self.registers[5][step],
        ];

        hasher::apply_round(&mut state, step);

        self.registers[0][step + 1] = state[0];
        self.registers[1][step + 1] = state[1];
        self.registers[2][step + 1] = state[2];
        self.registers[3][step + 1] = state[3];
        self.registers[4][step + 1] = state[4];
        self.registers[5][step + 1] = state[5];

        self.copy_state(step, HASH_STATE_WIDTH);
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
            self.registers[i][step + 1] = field::ZERO;
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
            let register = filled_vector(trace_length, trace_capacity, field::ZERO);
            self.registers.push(register);
        }
    }

    fn ensure_trace_capacity(&mut self, step: usize) {
        let current_length = self.registers[0].len();
        if step >= current_length - 1 {
            let new_length = current_length * 2;
            for i in 0..self.registers.len() {
                self.registers[i].resize(new_length, 0);
            }
        }
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn is_binary(value: u128) -> bool {
    return value == field::ZERO || value == field::ONE;
}