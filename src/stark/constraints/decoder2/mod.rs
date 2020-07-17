use crate::processor::opcodes2::{ FlowOps };
use crate::stark::trace::{ trace_state2::TraceState };
use super::utils::{ are_equal, is_zero, is_binary, binary_not, EvaluationResult };

mod op_bits;
use op_bits::{ enforce_op_bits };

mod sponge;
use sponge::{ enforce_hacc };

mod flow_ops;
use flow_ops::{ enforce_begin, enforce_tend, enforce_fend };

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================
const NUM_OP_CONSTRAINTS: usize = 20;
const OP_CONSTRAINT_DEGREES: [usize; NUM_OP_CONSTRAINTS] = [
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2,   // all op bits are binary
    7,                              // ld_ops and hd_ops cannot be all 0s
    8,                              // when cf_ops are not all 0s, ld_ops and hd_ops must be all 1s
    3,                              // PUSH is allowed only on multiples of 8
    2,                              // VOID can be followed only by VOID
    4, 4, 4, 4, 4, 4,               // cf_ops are aligned correctly
];

const NUM_SPONGE_CONSTRAINTS: usize = 4;
const SPONGE_CONSTRAINT_DEGREES: [usize; NUM_SPONGE_CONSTRAINTS] = [
    6, 7, 6, 6,                     // sponge transition constraints
];

const STACK_CONSTRAINT_DEGREE: usize = 4;

// TYPES AND INTERFACES
// ================================================================================================
pub struct Decoder {
    constraint_degrees  : Vec<usize>,
}

// DECODER CONSTRAINT EVALUATOR IMPLEMENTATION
// ================================================================================================
impl Decoder {

    fn new(ctx_depth: usize, loop_depth: usize) -> Decoder {

        let mut degrees = Vec::from(&OP_CONSTRAINT_DEGREES[..]);
        degrees.extend_from_slice(&SPONGE_CONSTRAINT_DEGREES[..]);
        degrees.resize(degrees.len() + ctx_depth + loop_depth, STACK_CONSTRAINT_DEGREE);

        return Decoder {
            constraint_degrees  : degrees,
        };
    }

    pub fn constraint_degrees(&self) -> &[usize] {
        return &self.constraint_degrees;
    }

    // EVALUATOR FUNCTIONS
    // --------------------------------------------------------------------------------------------

    pub fn evaluate(&self, current: &TraceState, next: &TraceState, step: usize, result: &mut [u128]) {

        // evaluate constraints for decoding op codes
        enforce_op_bits(&mut result[..NUM_OP_CONSTRAINTS], current, next);

        // evaluate constraints for flow control operations
        let result = &mut result[NUM_OP_CONSTRAINTS..];
        let op_flags = current.cf_op_flags();
        enforce_begin(result, current, next, op_flags[FlowOps::Begin as usize]);
        enforce_tend (result, current, next, op_flags[FlowOps::Tend as usize]);
        enforce_fend (result, current, next, op_flags[FlowOps::Fend as usize]);
        enforce_hacc (result, current, next, &vec![], op_flags[FlowOps::Hacc as usize]); // TODO
    }
}