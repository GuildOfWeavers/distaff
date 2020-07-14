use crate::math::{ field };
use crate::utils::{ hasher };
use crate::{ HASH_STATE_WIDTH, MIN_STACK_DEPTH, MAX_STACK_DEPTH };
use super::{ ProgramInputs, super::opcodes2::{ UserOps as Opcode, OpHint } };

#[cfg(test)]
mod tests;

// TYPES AND INTERFACES
// ================================================================================================
pub struct Stack {
    registers   : Vec<Vec<u128>>,
    tape_a      : Vec<u128>,
    tape_b      : Vec<u128>,
    max_depth   : usize,
    depth       : usize,
    step        : usize,
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
            depth: public_inputs.len(),
            step: 0,
        };
    }

    /// Executes `opcode` against the current state of the stack.
    pub fn execute(&mut self, op_code: Opcode, op_hint: OpHint) {

        // increment step pointer and make sure there is enough memory allocated to hold the trace
        self.advance_step();

        // execute the appropriate action against the current state of the stack
        match op_code {

            Opcode::Noop        => self.op_noop(),
            Opcode::Assert      => self.op_assert(),
            Opcode::AssertEq    => self.op_asserteq(),

            Opcode::Push        => self.op_push(op_hint),

            Opcode::Read        => self.op_read(op_hint),
            Opcode::Read2       => self.op_read2(),

            Opcode::Dup         => self.op_dup(),
            Opcode::Dup2        => self.op_dup2(),
            Opcode::Dup4        => self.op_dup4(),
            Opcode::Pad2        => self.op_pad2(),

            Opcode::Drop        => self.op_drop(),
            Opcode::Drop4       => self.op_drop4(),

            Opcode::Swap        => self.op_swap(),
            Opcode::Swap2       => self.op_swap2(),
            Opcode::Swap4       => self.op_swap4(),

            Opcode::Roll4       => self.op_roll4(),
            Opcode::Roll8       => self.op_roll8(),

            Opcode::Choose      => self.op_choose(),
            Opcode::Choose2     => self.op_choose2(),

            Opcode::Add         => self.op_add(),
            Opcode::Mul         => self.op_mul(),
            Opcode::Inv         => self.op_inv(),
            Opcode::Neg         => self.op_neg(),
            Opcode::Not         => self.op_not(),
            Opcode::And         => self.op_and(),
            Opcode::Or          => self.op_or(),

            Opcode::Eq          => self.op_eq(),
            Opcode::Cmp         => self.op_cmp(op_hint),
            Opcode::BinAcc      => self.op_binacc(op_hint),

            Opcode::RescR       => self.op_rescr(),
        }
    }

    /// Returns trace length of register traces in the decoder.
    pub fn trace_length(&self) -> usize {
        return self.registers[0].len();
    }

    /// Returns value of the current step pointer.
    pub fn current_step(&self) -> usize {
        return self.step;
    }

    /// Returns the value at the top of the stack at the current step.
    pub fn get_stack_top(&self) -> u128 {
        return self.registers[0][self.step];
    }

    /// TODO: add description
    pub fn finalize_trace(&mut self) {
        let trace_length = self.trace_length();
        for register in self.registers.iter_mut() {
            register.resize(self.step + 1, field::ZERO);
            register.resize(trace_length, register[self.step]);
        }
    }

    /// Merges all register traces into a single vector of traces.
    pub fn into_register_traces(mut self) -> Vec<Vec<u128>> {
        self.registers.truncate(self.max_depth);
        return self.registers;
    }

    // FLOW CONTROL OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn op_noop(&mut self) {
        self.copy_state(0);
    }

    fn op_assert(&mut self) {
        assert!(self.depth >= 1, "stack underflow at step {}", self.step);
        let value = self.registers[0][self.step - 1];
        assert!(value == field::ONE, "ASSERT failed at step {}", self.step);
        self.shift_left(1, 1);
    }

    fn op_asserteq(&mut self) {
        assert!(self.depth >= 2, "stack underflow at step {}", self.step);
        let x = self.registers[0][self.step - 1];
        let y = self.registers[1][self.step - 1];
        assert!(x == y, "ASSERTEQ failed at step {}", self.step);
        self.shift_left(2, 2);
    }

    // INPUT OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn op_push(&mut self, hint: OpHint) {
        self.shift_right(0, 1);
        let op_value = match hint {
            OpHint::PushValue(value) => value,
            _ => panic!("invalid value for PUSH operation at step {}", self.step),
        };
        self.registers[0][self.step] = op_value;
    }

    fn op_read(&mut self, hint: OpHint) {
        // process execution hint
        match hint {
            OpHint::EqStart => {
                // if we are about to equality comparison sequence, push inverse of the difference
                // between top two stack values onto secret tape A, if they are equal; otherwise
                // push value 1
                assert!(self.depth >= 2, "stack underflow at step {}", self.step);
                let x = self.registers[0][self.step - 1];
                let y = self.registers[1][self.step - 1];
                if x == y {
                    self.tape_a.push(field::ONE);
                }
                else {
                    self.tape_a.push(field::inv(field::sub(x, y)));
                }
            },
            _ => {
                assert!(self.tape_a.len() > 0, "attempt to read from empty tape A at step {}", self.step);
            }
        }

        self.shift_right(0, 1);
        let value = self.tape_a.pop().unwrap();
        self.registers[0][self.step] = value;
    }

    fn op_read2(&mut self) {
        assert!(self.tape_a.len() > 0, "attempt to read from empty tape A at step {}", self.step);
        assert!(self.tape_b.len() > 0, "attempt to read from empty tape B at step {}", self.step);
        self.shift_right(0, 2);
        let value_a = self.tape_a.pop().unwrap();
        let value_b = self.tape_b.pop().unwrap();
        self.registers[0][self.step] = value_b;
        self.registers[1][self.step] = value_a;
    }

    // STACK MANIPULATION OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn op_dup(&mut self) {
        assert!(self.depth >= 1, "stack underflow at step {}", self.step);
        self.shift_right(0, 1);
        self.registers[0][self.step] = self.registers[0][self.step - 1];
    }

    fn op_dup2(&mut self) {
        assert!(self.depth >= 2, "stack underflow at step {}", self.step);
        self.shift_right(0, 2);
        self.registers[0][self.step] = self.registers[0][self.step - 1];
        self.registers[1][self.step] = self.registers[1][self.step - 1];
    }

    fn op_dup4(&mut self) {
        assert!(self.depth >= 4, "stack underflow at step {}", self.step);
        self.shift_right(0, 4);
        self.registers[0][self.step] = self.registers[0][self.step - 1];
        self.registers[1][self.step] = self.registers[1][self.step - 1];
        self.registers[2][self.step] = self.registers[2][self.step - 1];
        self.registers[3][self.step] = self.registers[3][self.step - 1];
    }

    fn op_pad2(&mut self) {
        self.shift_right(0, 2);
        self.registers[0][self.step] = field::ZERO;
        self.registers[1][self.step] = field::ZERO;
    }

    fn op_drop(&mut self) {
        assert!(self.depth >= 1, "stack underflow at step {}", self.step);
        self.shift_left(1, 1);
    }

    fn op_drop4(&mut self) {
        assert!(self.depth >= 4, "stack underflow at step {}", self.step);
        self.shift_left(4, 4);
    }

    fn op_swap(&mut self) {
        assert!(self.depth >= 2, "stack underflow at step {}", self.step);
        self.registers[0][self.step] = self.registers[1][self.step - 1];
        self.registers[1][self.step] = self.registers[0][self.step - 1];
        self.copy_state(2);
    }

    fn op_swap2(&mut self) {
        assert!(self.depth >= 4, "stack underflow at step {}", self.step);
        self.registers[0][self.step] = self.registers[2][self.step - 1];
        self.registers[1][self.step] = self.registers[3][self.step - 1];
        self.registers[2][self.step] = self.registers[0][self.step - 1];
        self.registers[3][self.step] = self.registers[1][self.step - 1];
        self.copy_state(4);
    }

    fn op_swap4(&mut self) {
        assert!(self.depth >= 8, "stack underflow at step {}", self.step);
        self.registers[0][self.step] = self.registers[4][self.step - 1];
        self.registers[1][self.step] = self.registers[5][self.step - 1];
        self.registers[2][self.step] = self.registers[6][self.step - 1];
        self.registers[3][self.step] = self.registers[7][self.step - 1];
        self.registers[4][self.step] = self.registers[0][self.step - 1];
        self.registers[5][self.step] = self.registers[1][self.step - 1];
        self.registers[6][self.step] = self.registers[2][self.step - 1];
        self.registers[7][self.step] = self.registers[3][self.step - 1];
        self.copy_state(8);
    }

    fn op_roll4(&mut self) {
        assert!(self.depth >= 4, "stack underflow at step {}", self.step);
        self.registers[0][self.step] = self.registers[3][self.step - 1];
        self.registers[1][self.step] = self.registers[0][self.step - 1];
        self.registers[2][self.step] = self.registers[1][self.step - 1];
        self.registers[3][self.step] = self.registers[2][self.step - 1];
        self.copy_state(4);
    }

    fn op_roll8(&mut self) {
        assert!(self.depth >= 8, "stack underflow at step {}", self.step);
        self.registers[0][self.step] = self.registers[7][self.step - 1];
        self.registers[1][self.step] = self.registers[0][self.step - 1];
        self.registers[2][self.step] = self.registers[1][self.step - 1];
        self.registers[3][self.step] = self.registers[2][self.step - 1];
        self.registers[4][self.step] = self.registers[3][self.step - 1];
        self.registers[5][self.step] = self.registers[4][self.step - 1];
        self.registers[6][self.step] = self.registers[5][self.step - 1];
        self.registers[7][self.step] = self.registers[6][self.step - 1];
        self.copy_state(8);
    }

    // SELECTION OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn op_choose(&mut self) {
        assert!(self.depth >= 3, "stack underflow at step {}", self.step);
        let condition = self.registers[2][self.step - 1];
        if condition == field::ONE {
            self.registers[0][self.step] = self.registers[0][self.step - 1];
        }
        else if condition == field::ZERO {
            self.registers[0][self.step] = self.registers[1][self.step - 1];
        }
        else {
            assert!(false, "CHOOSE on a non-binary condition at step {}", self.step);
        }
        self.shift_left(3, 2);
    }

    fn op_choose2(&mut self) {
        assert!(self.depth >= 6, "stack underflow at step {}", self.step);
        let condition = self.registers[4][self.step - 1];
        if condition == field::ONE {
            self.registers[0][self.step] = self.registers[0][self.step - 1];
            self.registers[1][self.step] = self.registers[1][self.step - 1];
        }
        else if condition == field::ZERO {
            self.registers[0][self.step] = self.registers[2][self.step - 1];
            self.registers[1][self.step] = self.registers[3][self.step - 1];
        }
        else {
            assert!(false, "CHOOSE2 on a non-binary condition at step {}", self.step);
        }
        self.shift_left(6, 4);
    }

    // ARITHMETIC AND BOOLEAN OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn op_add(&mut self) {
        assert!(self.depth >= 2, "stack underflow at step {}", self.step);
        let x = self.registers[0][self.step - 1];
        let y = self.registers[1][self.step - 1];
        self.registers[0][self.step] = field::add(x, y);
        self.shift_left(2, 1);
    }

    fn op_mul(&mut self) {
        assert!(self.depth >= 2, "stack underflow at step {}", self.step);
        let x = self.registers[0][self.step - 1];
        let y = self.registers[1][self.step - 1];
        self.registers[0][self.step] = field::mul(x, y);
        self.shift_left(2, 1);
    }

    fn op_inv(&mut self) {
        assert!(self.depth >= 1, "stack underflow at step {}", self.step);
        let x = self.registers[0][self.step - 1];
        assert!(x != field::ZERO, "cannot compute INV of {} at step {}", field::ZERO, self.step);
        self.registers[0][self.step] = field::inv(x);
        self.copy_state(1);
    }

    fn op_neg(&mut self) {
        assert!(self.depth >= 1, "stack underflow at step {}", self.step);
        let x = self.registers[0][self.step - 1];
        self.registers[0][self.step] = field::neg(x);
        self.copy_state(1);
    }

    fn op_not(&mut self) {
        assert!(self.depth >= 1, "stack underflow at step {}", self.step);
        let x = self.registers[0][self.step - 1];
        assert!(is_binary(x), "cannot compute NOT of a non-binary value at step {}", self.step);
        self.registers[0][self.step] = field::sub(field::ONE, x);
        self.copy_state(1);
    }

    fn op_and(&mut self) {
        assert!(self.depth >= 2, "stack underflow at step {}", self.step);
        let x = self.registers[0][self.step - 1];
        let y = self.registers[1][self.step - 1];
        assert!(is_binary(x), "cannot compute AND for a non-binary value at step {}", self.step);
        assert!(is_binary(y), "cannot compute AND for a non-binary value at step {}", self.step);

        self.registers[0][self.step] = if x == field::ONE && y == field::ONE { field::ONE } else { field::ZERO };
        self.shift_left(2, 1);
    }

    fn op_or(&mut self) {
        assert!(self.depth >= 2, "stack underflow at step {}", self.step);
        let x = self.registers[0][self.step - 1];
        let y = self.registers[1][self.step - 1];
        assert!(is_binary(x), "cannot compute OR for a non-binary value at step {}", self.step);
        assert!(is_binary(y), "cannot compute OR for a non-binary value at step {}", self.step);

        self.registers[0][self.step] = if x == field::ONE || y == field::ONE { field::ONE } else { field::ZERO };
        self.shift_left(2, 1);
    }


    // COMPARISON OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn op_eq(&mut self) {
        assert!(self.depth >= 3, "stack underflow at step {}", self.step);
        let aux = self.registers[0][self.step - 1];
        let x = self.registers[1][self.step - 1];
        let y = self.registers[2][self.step - 1];
        if x == y {
            self.registers[0][self.step] = field::ONE;
        } else {
            let diff = field::sub(x, y);
            assert!(aux == field::inv(diff), "invalid AUX value for EQ operation at step {}", self.step);
            self.registers[0][self.step] = field::ZERO;
        }
        self.shift_left(3, 2);
    }

    fn op_cmp(&mut self, hint: OpHint) {
        // process execution hint
        match hint {
            OpHint::CmpStart(n) => {
                // if we are about to start comparison sequence, push binary decompositions
                // of a and b values onto the tapes
                assert!(self.depth >= 10, "stack underflow at step {}", self.step);
                let a_val = self.registers[8][self.step - 1];
                let b_val = self.registers[9][self.step - 1];
                for i in 0..n {
                    self.tape_a.push((a_val >> i) & 1);
                    self.tape_b.push((b_val >> i) & 1);
                }
            },
            _ => {
                assert!(self.depth >= 8, "stack underflow at step {}", self.step);
                assert!(self.tape_a.len() > 0, "attempt to read from empty tape A at step {}", self.step);
                assert!(self.tape_b.len() > 0, "attempt to read from empty tape B at step {}", self.step);
            }
        }

        // get next bits of a and b values from the tapes
        let a_bit = self.tape_a.pop().unwrap();
        assert!(a_bit == field::ZERO || a_bit == field::ONE,
            "expected binary input at step {} but received: {}", self.step, a_bit);
        let b_bit = self.tape_b.pop().unwrap();
        assert!(b_bit == field::ZERO || b_bit == field::ONE,
            "expected binary input at step {} but received: {}", self.step, b_bit);

        // determine which bit is greater
        let bit_gt = field::mul(a_bit, field::sub(field::ONE, b_bit));
        let bit_lt = field::mul(b_bit, field::sub(field::ONE, a_bit));

        // compute current power of 2 for binary decomposition
        let power_of_two = self.registers[0][self.step - 1];
        assert!(power_of_two.is_power_of_two(),
            "expected top of the stack at step {} to be a power of 2, but received {}", self.step, power_of_two);
        let next_power_of_two = if power_of_two == 1 {
            field::div(power_of_two, 2)
        }
        else {
            power_of_two >> 1
        };

        // determine if the result of comparison is already known
        let gt = self.registers[4][self.step - 1];
        let lt = self.registers[5][self.step - 1];
        let not_set = field::mul(field::sub(field::ONE, gt), field::sub(field::ONE, lt));

        // update the next state of the computation
        self.registers[0][self.step] = next_power_of_two;
        self.registers[1][self.step] = a_bit;
        self.registers[2][self.step] = b_bit;
        self.registers[3][self.step] = not_set;
        self.registers[4][self.step] = field::add(gt, field::mul(bit_gt, not_set));
        self.registers[5][self.step] = field::add(lt, field::mul(bit_lt, not_set));
        self.registers[6][self.step] = field::add(self.registers[6][self.step - 1], field::mul(b_bit, power_of_two));
        self.registers[7][self.step] = field::add(self.registers[7][self.step - 1], field::mul(a_bit, power_of_two));

        self.copy_state(8);
    }

    fn op_binacc(&mut self, hint: OpHint) {
        // process execution hint
        match hint {
            OpHint::RcStart(n) => {
                // if we are about to start range check sequence, push binary decompositions
                // of the value onto tape A
                assert!(self.depth >= 4, "stack underflow at step {}", self.step);
                let val = self.registers[3][self.step - 1];
                for i in 0..n {
                    self.tape_a.push((val >> i) & 1);
                }
            },
            _ => {
                assert!(self.depth >= 3, "stack underflow at step {}", self.step);
                assert!(self.tape_a.len() > 0, "attempt to read from empty tape A at step {}", self.step);
            }
        }

        // get the next bit of the value from tape A
        let bit = self.tape_a.pop().unwrap();
        assert!(bit == field::ZERO || bit == field::ONE,
            "expected binary input at step {} but received: {}", self.step, bit);

        // compute current power of 2 for binary decomposition
        let power_of_two = self.registers[0][self.step - 1];
        assert!(power_of_two.is_power_of_two(),
            "expected top of the stack at step {} to be a power of 2, but received {}", self.step, power_of_two);
        let next_power_of_two = if power_of_two == 1 {
                field::div(power_of_two, 2)
            }
            else {
                power_of_two >> 1
            };

        let acc = self.registers[2][self.step - 1];

        // update the next state of the computation
        self.registers[0][self.step] = next_power_of_two;
        self.registers[1][self.step] = bit;
        self.registers[2][self.step] = field::add(acc, field::mul(bit, power_of_two));

        self.copy_state(3);
    }

    // CRYPTOGRAPHIC OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn op_rescr(&mut self) {
        assert!(self.depth >= HASH_STATE_WIDTH, "stack underflow at step {}", self.step);
        let mut state = [
            self.registers[0][self.step - 1],
            self.registers[1][self.step - 1],
            self.registers[2][self.step - 1],
            self.registers[3][self.step - 1],
            self.registers[4][self.step - 1],
            self.registers[5][self.step - 1],
        ];

        hasher::apply_round(&mut state, self.step - 1);

        self.registers[0][self.step] = state[0];
        self.registers[1][self.step] = state[1];
        self.registers[2][self.step] = state[2];
        self.registers[3][self.step] = state[3];
        self.registers[4][self.step] = state[4];
        self.registers[5][self.step] = state[5];

        self.copy_state(HASH_STATE_WIDTH);
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------

    fn copy_state(&mut self, start: usize) {
        for i in start..self.depth {
            self.registers[i][self.step] = self.registers[i][self.step - 1];
        }
    }

    fn shift_left(&mut self, start: usize, pos_count: usize) {
        assert!(self.depth >= pos_count, "stack underflow at step {}", self.step);
        
        // shift all values by pos_count to the left
        for i in start..self.depth {
            self.registers[i - pos_count][self.step] = self.registers[i][self.step - 1];
        }

        // set all "shifted-in" slots to 0
        for i in (self.depth - pos_count)..self.depth {
            self.registers[i][self.step] = field::ZERO;
        }

        // stack depth has been reduced by pos_count
        self.depth -= pos_count;
    }

    fn shift_right(&mut self, start: usize, pos_count: usize) {
        
        self.depth += pos_count;
        assert!(self.depth <= MAX_STACK_DEPTH, "stack overflow at step {}", self.step);

        if self.depth > self.max_depth {
            self.max_depth += pos_count;
            if self.max_depth > self.registers.len() {
                self.add_registers(self.max_depth - self.registers.len());
            }
        }

        for i in start..(self.depth - pos_count) {
            self.registers[i + pos_count][self.step] = self.registers[i][self.step - 1];
        }
    }

    /// Extends the stack by the specified number of registers.
    fn add_registers(&mut self, num_registers: usize) {
        for _ in 0..num_registers {
            self.registers.push(vec![field::ZERO; self.trace_length()]);
        }
    }

    fn advance_step(&mut self) {
        // increment step by 1
        self.step += 1;

        // make sure there is enough memory allocated for register traces
        if self.step >= self.trace_length() {
            let new_length = self.trace_length() * 2;
            for register in self.registers.iter_mut() { register.resize(new_length, field::ZERO); }
        }
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn is_binary(value: u128) -> bool {
    return value == field::ZERO || value == field::ONE;
}