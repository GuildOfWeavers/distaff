use crate::{ ACC_STATE_WIDTH, NUM_OP_BITS};
use crate::utils::{ accumulator };
use super::{ opcodes::f128 as opcodes };

// TYPES AND INTERFACES
// ================================================================================================
pub struct Decoder {
    op_code     : Vec<u128>,
    op_bits     : Vec<Vec<u128>>,
    op_acc      : Vec<Vec<u128>>,
    acc_state   : [u128; ACC_STATE_WIDTH],
}

// DECODER IMPLEMENTATION
// ================================================================================================
impl Decoder {

    /// Returns a new Decoder with enough memory allocated for each register to hold trace lengths
    /// of `init_trace_length` steps. Register traces will be expanded dynamically if the number
    /// of actual steps exceeds this initial setting.
    pub fn new(init_trace_length: usize) -> Decoder {

        // initialize op_bits registers
        let mut op_bits = Vec::with_capacity(NUM_OP_BITS);
        for _ in 0..NUM_OP_BITS {
            op_bits.push(vec![0; init_trace_length]);
        }

        // initialize op_acc registers
        let mut op_acc = Vec::with_capacity(ACC_STATE_WIDTH);
        for _ in 0..ACC_STATE_WIDTH {
            op_acc.push(vec![0; init_trace_length]);
        }

        let mut decoder = Decoder {
            op_code     : vec![0; init_trace_length],
            acc_state   : [0; ACC_STATE_WIDTH],
            op_bits,
            op_acc,
        };

        // populate the first state with BEGIN operation
        decoder.op_code[0] = opcodes::BEGIN;
        decoder.set_op_bits(opcodes::BEGIN, 0);

        return decoder;
    }

    /// Decodes the operation and appends the resulting decoder state to all register traces; if
    /// the `is_push` is set to true, op is interpreted as a value to be pushed onto the stack 
    /// rather than an actual op code.
    pub fn decode(&mut self, op: u128, is_push: bool, step: usize) {

        // make sure there is enough space to update current step
        self.ensure_trace_capacity(step);

        // set op_code and update op_bits baded on the op_code (assuming the operation isn't a PUSH
        self.op_code[step] = op;
        if is_push {
            self.set_op_bits(opcodes::NOOP, step);
        }
        else {
            self.set_op_bits(op, step);
        }
        
        // add the operation operation to program hash
        accumulator::apply_round(&mut self.acc_state, op, step);
        for i in 0..ACC_STATE_WIDTH {
            self.op_acc[i][step + 1] = self.acc_state[i];
        }
    }

    /// Merges all register traces into a single vector of traces.
    pub fn into_register_trace(mut self) -> Vec<Vec<u128>> {
        let mut registers = Vec::with_capacity(1 + NUM_OP_BITS + ACC_STATE_WIDTH);
        registers.push(self.op_code);
        registers.append(&mut self.op_bits);
        registers.append(&mut self.op_acc);
        return registers;
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------

    fn set_op_bits(&mut self, op_code: u128, step: usize) {
        for i in 0..NUM_OP_BITS {
            self.op_bits[i][step] = (op_code >> i) & 1;
        }
    }

    fn ensure_trace_capacity(&mut self, step: usize) {
        if step >= self.op_code.len() - 1 {
            let new_length = self.op_code.len() * 2;
            self.op_code.resize(new_length, 0);
            for i in 0..NUM_OP_BITS {
                self.op_bits[i].resize(new_length, 0);
            }
            for i in 0..ACC_STATE_WIDTH {
                self.op_acc[i].resize(new_length, 0);
            }
        }
    }
}