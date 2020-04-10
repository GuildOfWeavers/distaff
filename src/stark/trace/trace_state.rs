use std::fmt;
use std::cmp;
use crate::math::field::{ add, sub, mul, ONE };
use super::{ acc_hash::STATE_WIDTH as ACC_WIDTH, MIN_STACK_DEPTH };

// CONSTANTS
// ================================================================================================
const NUM_OP_BITS: usize = 5;
const NUM_LD_OPS: usize = 32;

// TYPES AND INTERFACES
// ================================================================================================
#[derive(Debug, PartialEq)]
pub struct TraceState {
    op_code     : u64,
    push_flag   : u64,
    op_bits     : [u64; NUM_OP_BITS],
    op_acc      : [u64; ACC_WIDTH],
    stack       : Vec<u64>,
}

// TRACE STATE IMPLEMENTATION
// ================================================================================================
impl TraceState {

    pub fn new(stack_depth: usize) -> TraceState {
        let stack_depth = cmp::max(stack_depth, MIN_STACK_DEPTH);
        return TraceState {
            op_code     : 0,
            push_flag   : 0,
            op_bits     : [0; NUM_OP_BITS],
            op_acc      : [0; ACC_WIDTH],
            stack       : vec![0; stack_depth],
        };
    }

    // OP_CODE
    // --------------------------------------------------------------------------------------------
    pub fn get_op_code(&self) -> u64 {
        return self.op_code;
    }

    pub fn set_op_code(&mut self, value: u64) {
        self.op_code = value;
    }

    // PUSH_FLAG
    // --------------------------------------------------------------------------------------------
    pub fn get_push_flag(&self) -> u64 {
        return self.push_flag;
    }

    pub fn set_push_flag(&mut self, value: u64) {
        self.push_flag = value;
    }

    // OP_ACC
    // --------------------------------------------------------------------------------------------
    pub fn get_op_acc(&self) -> &[u64; ACC_WIDTH] {
        return &self.op_acc;
    }

    pub fn set_op_acc(&mut self, value: [u64; ACC_WIDTH]) {
        self.op_acc = value;
    }

    // OP_BITS
    // --------------------------------------------------------------------------------------------
    pub fn get_op_bits(&self) -> &[u64; NUM_OP_BITS] {
        return &self.op_bits;
    }

    pub fn set_op_bits(&mut self, op_bits: [u64; NUM_OP_BITS]) {
        self.op_bits = op_bits;
    }

    pub fn get_op_bits_value(&self) -> u64 {
        let mut value = self.op_bits[0];
        value = add(value, mul(self.op_bits[1],  2));
        value = add(value, mul(self.op_bits[2],  4));
        value = add(value, mul(self.op_bits[3],  8));
        value = add(value, mul(self.op_bits[4], 16));
        return value;
    }

    // OP_FLAGS
    // --------------------------------------------------------------------------------------------
    pub fn get_op_flags(&self) -> [u64; NUM_LD_OPS] {

        // TODO: needs to be optimized - takes 30% of constraint evaluation time

        // initialize op_flags to 1
        let mut op_flags = [1; NUM_LD_OPS];

        // expand the bits
        for i in 0..5 {
            
            let segment_length = usize::pow(2, (i + 1) as u32);

            let inv_bit = sub(ONE, self.op_bits[i]);
            for j in 0..(segment_length / 2) {
                op_flags[j] = mul(op_flags[j], inv_bit);
            }

            for j in (segment_length / 2)..segment_length {
                op_flags[j] = mul(op_flags[j], self.op_bits[i]);
            }

            let segment_slice = unsafe { &*(&op_flags[0..segment_length] as *const [u64]) };
            for j in (segment_length..NUM_LD_OPS).step_by(segment_length) {
                op_flags[j..(j + segment_length)].copy_from_slice(segment_slice);
            }
        }

        return op_flags;
    }

    // STACK
    // --------------------------------------------------------------------------------------------
    pub fn get_stack(&self) -> &[u64] {
        return &self.stack;
    }

    pub fn set_stack_value(&mut self, index: usize, value: u64) {
        self.stack[index] = value;
    }
}

impl fmt::Display for TraceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]\t[{}]\t{:?}\t{:?}", self.op_code, self.push_flag, self.op_bits, self.stack)
    }
}