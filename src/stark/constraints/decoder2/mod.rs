use crate::math::{ field, polynom };
use crate::processor::opcodes2::{ FlowOps, UserOps };
use crate::stark::trace::{ trace_state2::TraceState };
use crate::utils::accumulator::{ get_extended_constants };
use super::utils::{ are_equal, is_zero, is_binary, binary_not, EvaluationResult };

mod op_bits;
use op_bits::{ enforce_op_bits };

mod sponge;
use sponge::{ enforce_hacc };

mod flow_ops;
use flow_ops::{
    enforce_begin,
    enforce_tend,
    enforce_fend,
    enforce_loop,
    enforce_wrap,
    enforce_break,
    enforce_void
};

#[cfg(test)]
mod tests;

// TODO: move to global constants
const SPONGE_WIDTH: usize = 4;
const SPONGE_CYCLE_LENGTH: usize = 16;

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
    6, 6, 6, 6,                     // sponge transition constraints
];

const STACK_CONSTRAINT_DEGREE: usize = 4;

// TYPES AND INTERFACES
// ================================================================================================
pub struct Decoder {
    ctx_depth           : usize,
    loop_depth          : usize,
    trace_length        : usize,
    ark_cycle_length    : usize,
    ark_values          : Vec<[u128; 2 * SPONGE_WIDTH]>,
    ark_polys           : Vec<Vec<u128>>,
    constraint_degrees  : Vec<usize>,
}

// DECODER CONSTRAINT EVALUATOR IMPLEMENTATION
// ================================================================================================
impl Decoder {

    fn new(trace_length: usize, extension_factor: usize, ctx_depth: usize, loop_depth: usize) -> Decoder {

        let mut degrees = Vec::from(&OP_CONSTRAINT_DEGREES[..]);
        degrees.extend_from_slice(&SPONGE_CONSTRAINT_DEGREES[..]);
        degrees.resize(degrees.len() + ctx_depth + loop_depth, STACK_CONSTRAINT_DEGREE);

        // extend rounds constants by the specified extension factor
        let (ark_polys, ark_evaluations) = get_extended_constants(extension_factor);
        let ark_cycle_length = SPONGE_CYCLE_LENGTH * extension_factor;
        let ark_values = transpose_constants(ark_evaluations, ark_cycle_length);

        return Decoder {
            ctx_depth, loop_depth,
            trace_length,
            ark_cycle_length, ark_values, ark_polys,
            constraint_degrees: degrees,
        };
    }

    pub fn ctx_depth(&self) -> usize {
        return self.ctx_depth;
    }

    pub fn loop_depth(&self) -> usize {
        return self.loop_depth;
    }

    pub fn constraint_degrees(&self) -> &[usize] {
        return &self.constraint_degrees;
    }

    // EVALUATOR FUNCTIONS
    // --------------------------------------------------------------------------------------------

    pub fn evaluate(&self, current: &TraceState, next: &TraceState, step: usize, result: &mut [u128]) {

        // determine round constants at the specified x coordinate
        let ark = self.ark_values[step % self.ark_cycle_length];

        // evaluate constraints for decoding op codes
        enforce_op_bits(&mut result[..NUM_OP_CONSTRAINTS], current, next);

        // evaluate constraints for flow control operations
        let result = &mut result[NUM_OP_CONSTRAINTS..];
        let op_flags = current.cf_op_flags();

        enforce_hacc (result, current, next, &ark, op_flags[FlowOps::Hacc as usize]);
        enforce_begin(result, current, next, op_flags[FlowOps::Begin as usize]);
        enforce_tend (result, current, next, op_flags[FlowOps::Tend as usize]);
        enforce_fend (result, current, next, op_flags[FlowOps::Fend as usize]);
        enforce_loop (result, current, next, op_flags[FlowOps::Loop as usize]);
        enforce_wrap (result, current, next, op_flags[FlowOps::Wrap as usize]);
        enforce_break(result, current, next, op_flags[FlowOps::Break as usize]);
        enforce_void (result, current, next, op_flags[FlowOps::Void as usize]);

    }

    pub fn evaluate_at(&self, current: &TraceState, next: &TraceState, x: u128, result: &mut [u128]) {

        // determine round constants at the specified x coordinate
        let num_cycles = (self.trace_length / SPONGE_CYCLE_LENGTH) as u128;
        let x = field::exp(x, num_cycles);
        let mut ark = [field::ZERO; 2 * SPONGE_WIDTH];
        for i in 0..ark.len() {
            ark[i] = polynom::eval(&self.ark_polys[i], x);
        }

        // evaluate constraints for decoding op codes
        enforce_op_bits(&mut result[..NUM_OP_CONSTRAINTS], current, next);

        // evaluate constraints for flow control operations
        let result = &mut result[NUM_OP_CONSTRAINTS..];
        let op_flags = current.cf_op_flags();

        enforce_hacc (result, current, next, &ark, op_flags[FlowOps::Hacc as usize]);
        enforce_begin(result, current, next, op_flags[FlowOps::Begin as usize]);
        enforce_tend (result, current, next, op_flags[FlowOps::Tend as usize]);
        enforce_fend (result, current, next, op_flags[FlowOps::Fend as usize]);
        enforce_loop (result, current, next, op_flags[FlowOps::Loop as usize]);
        enforce_wrap (result, current, next, op_flags[FlowOps::Wrap as usize]);
        enforce_break(result, current, next, op_flags[FlowOps::Break as usize]);
        enforce_void (result, current, next, op_flags[FlowOps::Void as usize]);
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn transpose_constants(ark: Vec<Vec<u128>>, cycle_length: usize) -> Vec<[u128; 2 * SPONGE_WIDTH]> {
    let mut ark_values = Vec::new();
    for i in 0..cycle_length {
        ark_values.push([field::ZERO; 2 * SPONGE_WIDTH]);
        for j in 0..(2 * SPONGE_WIDTH) {
            ark_values[i][j] = ark[j][i];
        }
    }
    return ark_values;
}