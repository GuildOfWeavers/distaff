use std::fmt;
use crate::math::field::{ add, sub, mul, ONE };

// CONSTANTS
// ================================================================================================
const NUM_OP_BITS: usize = 5;
const NUM_LD_OPS: usize = 32;

// TYPES AND INTERFACES
// ================================================================================================
#[derive(Debug, PartialEq)]
pub struct TraceState {
    pub op_code     : u64,
    pub push_flag   : u64,
    pub op_bits     : [u64; NUM_OP_BITS],
    pub stack       : Vec<u64>,
}

// TRACE STATE IMPLEMENTATION
// ================================================================================================
impl TraceState {

    pub fn new(stack_depth: usize) -> TraceState {
        return TraceState {
            op_code         : 0,
            push_flag       : 0,
            op_bits         : [0; NUM_OP_BITS],
            stack           : vec![0; stack_depth],
        };
    }

    pub fn get_op_bits_value(&self) -> u64 {
        let mut value = self.op_bits[0];
        value = add(value, mul(self.op_bits[1],  2));
        value = add(value, mul(self.op_bits[2],  4));
        value = add(value, mul(self.op_bits[3],  8));
        value = add(value, mul(self.op_bits[4], 16));
        return value;
    }

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
}

impl fmt::Display for TraceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]\t[{}]\t{:?}\t{:?}", self.op_code, self.push_flag, self.op_bits, self.stack)
    }
}