//use crate::{ ACC_STATE_WIDTH, NUM_OP_BITS};
use crate::{ Accumulator };
use super::{ opcodes::f128 as opcodes };

// TODO: get these constants from global declarations
const ACC_STATE_WIDTH: usize = 4;
const NUM_OP_BITS: usize = 5;

pub struct Decoder {
    op_code     : Vec<u128>,
    op_bits     : Vec<Vec<u128>>,
    op_acc      : Vec<Vec<u128>>,
    acc_state   : [u128; ACC_STATE_WIDTH],
}

impl Decoder {

    pub fn new(init_trace_length: usize) -> Decoder {

        // initialize op_bits registers
        let op_bits = vec![
            vec![0; init_trace_length], vec![0; init_trace_length], vec![0; init_trace_length],
            vec![0; init_trace_length], vec![0; init_trace_length]
        ];

        // initialize op_acc registers
        let op_acc = vec![
            vec![0; init_trace_length], vec![0; init_trace_length],
            vec![0; init_trace_length], vec![0; init_trace_length],
        ];

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

    pub fn decode(&mut self, op: u128, is_push: bool, step: usize) {

        self.op_code[step] = op;
        if is_push {
            self.set_op_bits(opcodes::NOOP, step);
        }
        else {
            self.set_op_bits(op, step);
        }
        
        // add last operation to program hash
        u128::apply_round(&mut self.acc_state, op, step);
        for i in 0..ACC_STATE_WIDTH {
            self.op_acc[i][step + 1] = self.acc_state[i];
        }
    }

    pub fn into_register_trace(mut self) -> Vec<Vec<u128>> {
        let mut registers = Vec::with_capacity(1 + NUM_OP_BITS + ACC_STATE_WIDTH);
        registers.push(self.op_code);
        registers.append(&mut self.op_bits);
        registers.append(&mut self.op_acc);
        return registers;
    }

    fn set_op_bits(&mut self, op_code: u128, step: usize) {
        for i in 0..NUM_OP_BITS {
            self.op_bits[i][step] = (op_code >> i) & 1;
        }
    }
}