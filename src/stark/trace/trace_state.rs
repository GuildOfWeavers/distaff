use std::fmt;
use std::cmp;
use crate::math::field::{ add, sub, mul, ONE };
use crate::stark::{ utils::hash_acc::STATE_WIDTH as ACC_WIDTH };
use super::{ NUM_OP_BITS, NUM_LD_OPS, MIN_STACK_DEPTH };

// CONSTANTS
// ================================================================================================
const OP_CODE_IDX   : usize = 0;
const PUSH_FLAG_IDX : usize = 1;
const OP_ACC_IDX    : usize = 2;
const OP_BITS_IDX   : usize = OP_ACC_IDX + ACC_WIDTH;
const STACK_IDX     : usize = OP_BITS_IDX + NUM_OP_BITS;

// TYPES AND INTERFACES
// ================================================================================================
#[derive(Debug, PartialEq)]
pub struct TraceState {
    pub state   : Vec<u64>,
    op_flags    : [u64; NUM_LD_OPS],
    op_flags_set: bool,
}

// TRACE STATE IMPLEMENTATION
// ================================================================================================
impl TraceState {

    pub fn new(stack_depth: usize) -> TraceState {
        let stack_depth = cmp::max(stack_depth, MIN_STACK_DEPTH);
        let state_width = 2 + NUM_OP_BITS + ACC_WIDTH + stack_depth;
        
        return TraceState {
            state       : vec![0; state_width],
            op_flags    : [0; NUM_LD_OPS],
            op_flags_set: false
        };
    }

    pub fn from_raw_state(state: Vec<u64>) -> TraceState {
        return TraceState {
            state       : state,
            op_flags    : [0; NUM_LD_OPS],
            op_flags_set: false
        };
    }

    // OP_CODE
    // --------------------------------------------------------------------------------------------
    pub fn get_op_code(&self) -> u64 {
        return self.state[OP_CODE_IDX];
    }

    pub fn set_op_code(&mut self, value: u64) {
        self.state[OP_CODE_IDX] = value;
    }

    // PUSH_FLAG
    // --------------------------------------------------------------------------------------------
    pub fn get_push_flag(&self) -> u64 {
        return self.state[PUSH_FLAG_IDX];
    }

    pub fn set_push_flag(&mut self, value: u64) {
        self.state[PUSH_FLAG_IDX] = value;
    }

    // OP_ACC
    // --------------------------------------------------------------------------------------------
    pub fn get_op_acc(&self) -> &[u64] {
        return &self.state[OP_ACC_IDX..(OP_ACC_IDX + ACC_WIDTH)];
    }

    pub fn set_op_acc(&mut self, value: [u64; ACC_WIDTH]) {
        self.state[OP_ACC_IDX..(OP_ACC_IDX + ACC_WIDTH)].copy_from_slice(&value);
    }

    // OP_BITS
    // --------------------------------------------------------------------------------------------
    pub fn get_op_bits(&self) -> &[u64] {
        return &self.state[OP_BITS_IDX..(OP_BITS_IDX + NUM_OP_BITS)];
    }

    pub fn set_op_bits(&mut self, op_bits: [u64; NUM_OP_BITS]) {
        self.state[OP_BITS_IDX..(OP_BITS_IDX + NUM_OP_BITS)].copy_from_slice(&op_bits);
        self.op_flags_set = false;
    }

    pub fn get_op_bits_value(&self) -> u64 {
        let op_bits = self.get_op_bits();
        let mut value = op_bits[0];
        value = add(value, mul(op_bits[1],  2));
        value = add(value, mul(op_bits[2],  4));
        value = add(value, mul(op_bits[3],  8));
        value = add(value, mul(op_bits[4], 16));
        return value;
    }

    // OP_FLAGS
    // --------------------------------------------------------------------------------------------
    pub fn get_op_flags(&self) -> [u64; NUM_LD_OPS] {
        if !self.op_flags_set {
            let mutable_self = unsafe { &mut *(self as *const _ as *mut TraceState) };
            mutable_self.set_op_flags();
            mutable_self.op_flags_set = true;
        }
        return self.op_flags;
    }

    fn set_op_flags(&mut self) {
        // TODO: needs to be optimized

        // initialize op_flags to 1
        let mut op_flags = [1; NUM_LD_OPS];

        // expand the bits
        let op_bits = self.get_op_bits();
        for i in 0..5 {
            
            let segment_length = usize::pow(2, (i + 1) as u32);

            let inv_bit = sub(ONE, op_bits[i]);
            for j in 0..(segment_length / 2) {
                op_flags[j] = mul(op_flags[j], inv_bit);
            }

            for j in (segment_length / 2)..segment_length {
                op_flags[j] = mul(op_flags[j], op_bits[i]);
            }

            let segment_slice = unsafe { &*(&op_flags[0..segment_length] as *const [u64]) };
            for j in (segment_length..NUM_LD_OPS).step_by(segment_length) {
                op_flags[j..(j + segment_length)].copy_from_slice(segment_slice);
            }
        }

        self.op_flags = op_flags;
    }

    // STACK
    // --------------------------------------------------------------------------------------------
    pub fn get_stack(&self) -> &[u64] {
        return &self.state[STACK_IDX..];
    }

    pub fn set_stack_value(&mut self, index: usize, value: u64) {
        self.state[STACK_IDX + index] = value;
    }
}

impl fmt::Display for TraceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]\t[{}]\t{:?}\t{:?}",
            self.get_op_code(), 
            self.get_push_flag(),
            self.get_op_bits(),
            self.get_stack())
    }
}