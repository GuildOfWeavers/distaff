use std::fmt;
use std::cmp;
use crate::math::field::{ add, sub, mul, ONE };
use crate::stark::hash_acc::STATE_WIDTH as ACC_STATE_WIDTH;
use super::{ NUM_OP_BITS, NUM_LD_OPS, MIN_STACK_DEPTH };

// CONSTANTS
// ================================================================================================
const OP_CODE_IDX   : usize = 0;
const PUSH_FLAG_IDX : usize = 1;
const OP_ACC_IDX    : usize = 2;
const OP_BITS_IDX   : usize = OP_ACC_IDX + ACC_STATE_WIDTH;
const STACK_IDX     : usize = OP_BITS_IDX + NUM_OP_BITS;

const STATIC_REGISTER_COUNT: usize = 2 + NUM_OP_BITS + ACC_STATE_WIDTH;

// TYPES AND INTERFACES
// ================================================================================================
#[derive(Debug, PartialEq)]
pub struct TraceState {
    registers       : Vec<u64>,
    state_width     : usize,
    op_flags        : [u64; NUM_LD_OPS],
    op_flags_set    : bool,
}

// TRACE STATE IMPLEMENTATION
// ================================================================================================
impl TraceState {

    pub fn new(stack_depth: usize) -> TraceState {
        let state_width = STATIC_REGISTER_COUNT + stack_depth;
        let num_registers = STATIC_REGISTER_COUNT + cmp::max(stack_depth, MIN_STACK_DEPTH);
        
        return TraceState {
            registers   : vec![0; num_registers],
            state_width : state_width,
            op_flags    : [0; NUM_LD_OPS],
            op_flags_set: false
        };
    }

    pub fn from_raw_state(mut state: Vec<u64>) -> TraceState {
        let state_width = state.len();
        let stack_depth = state_width - STATIC_REGISTER_COUNT;

        if stack_depth < MIN_STACK_DEPTH {
            state.resize(state.len() + (MIN_STACK_DEPTH - stack_depth), 0);
        }

        return TraceState {
            registers   : state,
            state_width : state_width,
            op_flags    : [0; NUM_LD_OPS],
            op_flags_set: false
        };
    }

    // OP_CODE
    // --------------------------------------------------------------------------------------------
    pub fn get_op_code(&self) -> u64 {
        return self.registers[OP_CODE_IDX];
    }

    pub fn set_op_code(&mut self, value: u64) {
        self.registers[OP_CODE_IDX] = value;
    }

    // PUSH_FLAG
    // --------------------------------------------------------------------------------------------
    pub fn get_push_flag(&self) -> u64 {
        return self.registers[PUSH_FLAG_IDX];
    }

    pub fn set_push_flag(&mut self, value: u64) {
        self.registers[PUSH_FLAG_IDX] = value;
    }

    // OP_ACC
    // --------------------------------------------------------------------------------------------
    pub fn get_op_acc(&self) -> &[u64] {
        return &self.registers[OP_ACC_IDX..(OP_ACC_IDX + ACC_STATE_WIDTH)];
    }

    pub fn set_op_acc(&mut self, value: [u64; ACC_STATE_WIDTH]) {
        self.registers[OP_ACC_IDX..(OP_ACC_IDX + ACC_STATE_WIDTH)].copy_from_slice(&value);
    }

    pub fn get_program_hash(&self) -> &[u64] {
        return &self.registers[OP_ACC_IDX..(OP_ACC_IDX + 4)];
    }

    // OP_BITS
    // --------------------------------------------------------------------------------------------
    pub fn get_op_bits(&self) -> &[u64] {
        return &self.registers[OP_BITS_IDX..(OP_BITS_IDX + NUM_OP_BITS)];
    }

    pub fn set_op_bits(&mut self, op_bits: [u64; NUM_OP_BITS]) {
        self.registers[OP_BITS_IDX..(OP_BITS_IDX + NUM_OP_BITS)].copy_from_slice(&op_bits);
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
            unsafe {
                let mutable_self = &mut *(self as *const _ as *mut TraceState);
                mutable_self.set_op_flags();
                mutable_self.op_flags_set = true;
            }
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

            for j in (segment_length..NUM_LD_OPS).step_by(segment_length) {
                op_flags.copy_within(0..segment_length, j);
            }
        }

        self.op_flags = op_flags;
    }

    // STACK
    // --------------------------------------------------------------------------------------------
    pub fn get_stack(&self) -> &[u64] {
        return &self.registers[STACK_IDX..];
    }

    pub fn set_stack_value(&mut self, index: usize, value: u64) {
        self.registers[STACK_IDX + index] = value;
    }

    pub fn compute_stack_depth(trace_register_count: usize) -> usize {
        return trace_register_count - STATIC_REGISTER_COUNT;
    }

    // RAW STATE
    // --------------------------------------------------------------------------------------------
    pub fn registers(&self) -> &[u64] {
        return &self.registers[..self.state_width];
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