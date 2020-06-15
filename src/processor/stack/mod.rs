use crate::math::{ FiniteField, F128 };
use crate::processor::{ ProgramInputs, opcodes };
use crate::stark::{ Hasher };
use crate::utils::{ filled_vector };

// TODO: get these constants global declarations
const HASH_STATE_WIDTH: usize = 6;
const MAX_USER_STACK_DEPTH: usize = 31;
const MIN_USER_STACK_DEPTH: usize = 8;

// TYPES AND INTERFACES
// ================================================================================================
pub struct Stack {
    pub aux_register    : Vec<u128>,
    pub user_registers  : Vec<Vec<u128>>,
    pub secret_inputs_a : Vec<u128>,
    pub secret_inputs_b : Vec<u128>,
    pub max_depth       : usize,
    pub depth           : usize,
}

// STACK IMPLEMENTATION
// ================================================================================================
impl Stack
{
    pub fn new(inputs: &ProgramInputs<u128>, init_trace_length: usize) -> Stack {

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

        let aux_register = vec![F128::ZERO; init_trace_length];

        // reverse secret inputs so that they are consumed in FIFO order
        let [secret_inputs_a, secret_inputs_b] = inputs.get_secret_inputs();
        let mut secret_inputs_a = secret_inputs_a.clone();
        secret_inputs_a.reverse();
        let mut secret_inputs_b = secret_inputs_b.clone();
        secret_inputs_b.reverse();

        return Stack {
            aux_register,
            user_registers,
            secret_inputs_a,
            secret_inputs_b,
            max_depth: public_inputs.len(),
            depth: public_inputs.len()
        };
    }

    pub fn execute(&mut self, current_op: F128, next_op: F128, step: usize) {
        match current_op.as_u8() {

            opcodes::BEGIN   => self.noop(step),
            opcodes::NOOP    => self.noop(step),
            opcodes::ASSERT  => self.assert(step),

            opcodes::PUSH    => self.push(step, next_op),

            opcodes::READ    => self.read(step),
            opcodes::READ2   => self.read2(step),

            opcodes::DUP     => self.dup(step),
            opcodes::DUP2    => self.dup2(step),
            opcodes::DUP4    => self.dup4(step),
            opcodes::PAD2    => self.pad2(step),

            opcodes::DROP    => self.drop(step),
            opcodes::DROP4   => self.drop4(step),

            opcodes::SWAP    => self.swap(step),
            opcodes::SWAP2   => self.swap2(step),
            opcodes::SWAP4   => self.swap4(step),

            opcodes::ROLL4   => self.roll4(step),
            opcodes::ROLL8   => self.roll8(step),

            opcodes::CHOOSE  => self.choose(step),
            opcodes::CHOOSE2 => self.choose2(step),

            opcodes::ADD     => self.add(step),
            opcodes::MUL     => self.mul(step),
            opcodes::INV     => self.inv(step),
            opcodes::NEG     => self.neg(step),
            opcodes::NOT     => self.not(step),

            opcodes::EQ      => self.eq(step),
            opcodes::CMP     => self.cmp(step),
            opcodes::BINACC  => self.binacc(step),

            opcodes::HASHR   => self.hashr(step),

            _ => panic!("operation {} is not supported", current_op)
        }
    }

    pub fn into_register_traces(mut self) -> Vec<Vec<u128>> {
        self.user_registers.truncate(self.max_depth);
        let mut registers = Vec::with_capacity(1 + self.user_registers.len());
        registers.push(self.aux_register);
        registers.append(&mut self.user_registers);
        return registers;
    }

    // OPERATIONS
    // --------------------------------------------------------------------------------------------
    pub fn noop(&mut self, step: usize) {
        self.copy_state(step, 0);
    }

    pub fn assert(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        let value = self.user_registers[0][step];
        assert!(value == F128::ONE, "ASSERT failed at step {}", step);
        self.shift_left(step, 1, 1);
    }

    pub fn push(&mut self, step: usize, value: F128) {
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
        self.user_registers[0][step + 1] = F128::ZERO;
        self.user_registers[1][step + 1] = F128::ZERO;
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

    pub fn choose2(&mut self, step: usize) {
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

    pub fn add(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        let y = self.user_registers[1][step];
        self.user_registers[0][step + 1] = F128::add(x, y);
        self.shift_left(step, 2, 1);
    }

    pub fn mul(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        let y = self.user_registers[1][step];
        self.user_registers[0][step + 1] = F128::mul(x, y);
        self.shift_left(step, 2, 1);
    }

    pub fn inv(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        assert!(x != F128::ZERO, "cannot compute INV of {} at step {}", F128::ZERO, step);
        self.user_registers[0][step + 1] = F128::inv(x);
        self.copy_state(step, 1);
    }

    pub fn neg(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        self.user_registers[0][step + 1] = F128::neg(x);
        self.copy_state(step, 1);
    }

    pub fn not(&mut self, step: usize) {
        assert!(self.depth >= 1, "stack underflow at step {}", step);
        let x = self.user_registers[0][step];
        assert!(x == F128::ZERO || x == F128::ONE, "cannot compute NOT of a non-binary value at step {}", step);
        self.user_registers[0][step + 1] = F128::sub(F128::ONE, x);
        self.copy_state(step, 1);
    }

    pub fn eq(&mut self, step: usize) {
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

    pub fn cmp(&mut self, step: usize) {
        assert!(self.depth >= 7, "stack underflow at step {}", step);
        assert!(self.secret_inputs_a.len() > 0, "ran out of secret inputs at step {}", step);
        assert!(self.secret_inputs_b.len() > 0, "ran out of secret inputs at step {}", step);
        let a_bit = self.secret_inputs_a.pop().unwrap();
        assert!(a_bit == F128::ZERO || a_bit == F128::ONE,
            "expected binary input at step {} but received: {}", step, a_bit);
        let b_bit = self.secret_inputs_b.pop().unwrap();
        assert!(b_bit == F128::ZERO || b_bit == F128::ONE,
            "expected binary input at step {} but received: {}", step, b_bit);

        let bit_gt = F128::mul(a_bit, F128::sub(F128::ONE, b_bit));
        let bit_lt = F128::mul(b_bit, F128::sub(F128::ONE, a_bit));

        let power_of_two = self.user_registers[0][step];    // TODO: make sure it is power of 2
        let gt = self.user_registers[3][step];
        let lt = self.user_registers[4][step];
        let not_set = F128::mul(F128::sub(F128::ONE, gt), F128::sub(F128::ONE, lt));

        self.aux_register[step] = not_set;
        self.user_registers[0][step + 1] = F128::div(power_of_two, F128::from_usize(2)); // TODO: replace with shift
        self.user_registers[1][step + 1] = a_bit;
        self.user_registers[2][step + 1] = b_bit;
        self.user_registers[3][step + 1] = F128::add(gt, F128::mul(bit_gt, not_set));
        self.user_registers[4][step + 1] = F128::add(lt, F128::mul(bit_lt, not_set));
        self.user_registers[5][step + 1] = F128::add(self.user_registers[5][step], F128::mul(b_bit, power_of_two));
        self.user_registers[6][step + 1] = F128::add(self.user_registers[6][step], F128::mul(a_bit, power_of_two));

        self.copy_state(step, 7);
    }

    pub fn binacc(&mut self, step: usize) {
        assert!(self.depth >= 2, "stack underflow at step {}", step);
        assert!(self.secret_inputs_a.len() > 0, "ran out of secret inputs at step {}", step);

        let bit = self.secret_inputs_a.pop().unwrap();
        let power_of_two = self.user_registers[0][step];    // TODO: make sure it is power of 2
        let acc = self.user_registers[1][step];

        self.aux_register[step] = bit;
        self.user_registers[0][step + 1] = F128::div(power_of_two, F128::from_usize(2)); // TODO: replace with shift
        self.user_registers[1][step + 1] = F128::add(acc, F128::mul(bit, power_of_two));

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

        F128::apply_round(&mut state, step);

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
}