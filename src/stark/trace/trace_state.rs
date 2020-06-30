use std::fmt;
use std::cmp;
use crate::math::field;
use crate::{
    MIN_STACK_DEPTH,
    DECODER_WIDTH,
    OP_CODE_INDEX,
    OP_BITS_RANGE,
    OP_ACC_RANGE,
    PROG_HASH_RANGE,
    NUM_LD_OPS
};

// TYPES AND INTERFACES
// ================================================================================================
#[derive(Debug, PartialEq)]
pub struct TraceState {
    registers       : Vec<u128>,
    state_width     : usize,
    op_flags        : [u128; NUM_LD_OPS],
    op_flags_set    : bool,
}

// TRACE STATE IMPLEMENTATION
// ================================================================================================
impl TraceState {
    pub fn new(stack_depth: usize) -> TraceState {
        let state_width = DECODER_WIDTH + stack_depth;
        let num_registers = DECODER_WIDTH + cmp::max(stack_depth, MIN_STACK_DEPTH);
        
        return TraceState {
            registers   : vec![field::ZERO; num_registers],
            state_width : state_width,
            op_flags    : [field::ZERO; NUM_LD_OPS],
            op_flags_set: false
        };
    }

    pub fn from_raw_state(mut state: Vec<u128>) -> TraceState {
        let state_width = state.len();
        let stack_depth = state_width - DECODER_WIDTH;

        if stack_depth < MIN_STACK_DEPTH {
            state.resize(state.len() + (MIN_STACK_DEPTH - stack_depth), field::ZERO);
        }

        return TraceState {
            registers   : state,
            state_width : state_width,
            op_flags    : [field::ZERO; NUM_LD_OPS],
            op_flags_set: false
        };
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------
    pub fn get_op_code(&self) -> u128 {
        return self.registers[OP_CODE_INDEX];
    }

    pub fn get_op_acc(&self) -> &[u128] {
        return &self.registers[OP_ACC_RANGE];
    }

    pub fn get_program_hash(&self) -> &[u128] {
        return &self.registers[PROG_HASH_RANGE];
    }

    pub fn get_op_bits(&self) -> &[u128] {
        return &self.registers[OP_BITS_RANGE];
    }

    pub fn get_op_flags(&self) -> [u128; NUM_LD_OPS] {
        if !self.op_flags_set {
            unsafe {
                let mutable_self = &mut *(self as *const _ as *mut TraceState);
                mutable_self.set_op_flags();
                mutable_self.op_flags_set = true;
            }
        }
        return self.op_flags;
    }

    pub fn get_stack(&self) -> &[u128] {
        return &self.registers[DECODER_WIDTH..];
    }

    pub fn get_user_stack(&self) -> &[u128] {
        // TODO: move user stack offset into a separate constant
        return &self.registers[(DECODER_WIDTH + 1)..];
    }

    pub fn compute_stack_depth(trace_register_count: usize) -> usize {
        return trace_register_count - DECODER_WIDTH;
    }

    // RAW STATE
    // --------------------------------------------------------------------------------------------
    pub fn registers(&self) -> &[u128] {
        return &self.registers[..self.state_width];
    }

    pub fn set_register(&mut self, index: usize, value: u128) {
        self.registers[index] = value;
        self.op_flags_set = false;
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------
    fn set_op_flags(&mut self) {
        // TODO: needs to be optimized

        // initialize op_flags to 1
        let mut op_flags = [field::ONE; NUM_LD_OPS];

        // expand the bits
        let op_bits = self.get_op_bits();
        for i in 0..5 {
            
            let segment_length = usize::pow(2, (i + 1) as u32);

            let inv_bit = field::sub(field::ONE, op_bits[i]);
            for j in 0..(segment_length / 2) {
                op_flags[j] = field::mul(op_flags[j], inv_bit);
            }

            for j in (segment_length / 2)..segment_length {
                op_flags[j] = field::mul(op_flags[j], op_bits[i]);
            }

            for j in (segment_length..NUM_LD_OPS).step_by(segment_length) {
                op_flags.copy_within(0..segment_length, j);
            }
        }

        self.op_flags = op_flags;
    }
}

impl fmt::Display for TraceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]\t{:?}\t{:?}\t{:?}",
            self.get_op_code(), 
            self.get_op_bits(),
            self.get_op_acc(),
            self.get_stack())
    }
}