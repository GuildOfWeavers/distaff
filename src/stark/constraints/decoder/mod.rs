use std::{ cmp };
use crate::{
    math::{ field, polynom },
    processor::opcodes::{ FlowOps, UserOps },
    stark::trace::TraceState,
    utils::sponge::ARK, SPONGE_WIDTH, BASE_CYCLE_LENGTH, MIN_CONTEXT_DEPTH, MIN_LOOP_DEPTH,
};
use super::utils::{
    are_equal, is_zero, is_binary, binary_not, extend_constants, EvaluationResult,
    enforce_stack_copy, enforce_left_shift, enforce_right_shift,
};

mod op_bits;
use op_bits::{ enforce_op_bits };

mod sponge;
use sponge::{ enforce_hacc };

mod flow_ops;
use flow_ops::{
    enforce_begin, enforce_tend, enforce_fend, enforce_void,
    enforce_loop, enforce_wrap, enforce_break,
};

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================
const NUM_OP_CONSTRAINTS: usize = 15;
const OP_CONSTRAINT_DEGREES: [usize; NUM_OP_CONSTRAINTS] = [
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2,   // all op bits are binary
    3,                              // op_counter should be incremented for HACC operations
    8,                              // ld_ops and hd_ops cannot be all 0s
    8,                              // when cf_ops are not all 0s, ld_ops and hd_ops must be all 1s
    6,                              // VOID can be followed only by VOID
    4,                              // operations happen on allowed step multiples
];

const NUM_SPONGE_CONSTRAINTS: usize = 4;
const SPONGE_CONSTRAINT_DEGREES: [usize; NUM_SPONGE_CONSTRAINTS] = [
    6, 7, 6, 6,                     // sponge transition constraints
];

const LOOP_IMAGE_CONSTRAINT_DEGREE: usize = 4;
const STACK_CONSTRAINT_DEGREE: usize = 4;

const CYCLE_MASK_IDX : usize = 0;
const PREFIX_MASK_IDX: usize = 1;
const PUSH_MASK_IDX  : usize = 2;

pub const NUM_STATIC_DECODER_CONSTRAINTS: usize =
    NUM_OP_CONSTRAINTS
    + NUM_SPONGE_CONSTRAINTS
    + 1;    // for loop image constraint

// TYPES AND INTERFACES
// ================================================================================================
pub struct Decoder {
    ctx_depth           : usize,
    loop_depth          : usize,
    trace_length        : usize,
    cycle_length        : usize,
    ark_values          : Vec<[u128; 2 * SPONGE_WIDTH]>,
    ark_polys           : Vec<Vec<u128>>,
    mask_values         : Vec<[u128; 3]>,
    mask_polys          : Vec<Vec<u128>>,
    constraint_degrees  : Vec<usize>,
}

// DECODER CONSTRAINT EVALUATOR IMPLEMENTATION
// ================================================================================================
impl Decoder {

    pub fn new(trace_length: usize, extension_factor: usize, ctx_depth: usize, loop_depth: usize) -> Decoder 
    {
        // build an array of constraint degrees for the decoder
        let mut degrees = Vec::from(&OP_CONSTRAINT_DEGREES[..]);
        degrees.extend_from_slice(&SPONGE_CONSTRAINT_DEGREES[..]);
        degrees.push(LOOP_IMAGE_CONSTRAINT_DEGREE);
        degrees.resize(degrees.len()
            + cmp::max(ctx_depth, MIN_CONTEXT_DEPTH)
            + cmp::max(loop_depth, MIN_LOOP_DEPTH),
            STACK_CONSTRAINT_DEGREE);

        // determine extended cycle length
        let cycle_length = BASE_CYCLE_LENGTH * extension_factor;

        // extend rounds constants by the specified extension factor
        let (ark_polys, ark_evaluations) = extend_constants(&ARK, extension_factor);
        let ark_values = transpose_ark_constants(ark_evaluations, cycle_length);

        // extend mask constants by the specified extension factor
        let (mask_polys, mask_evaluations) = extend_constants(&MASKS, extension_factor);
        let mask_values = transpose_mask_constants(mask_evaluations, cycle_length);

        return Decoder {
            ctx_depth, loop_depth,
            trace_length, cycle_length,
            ark_values, ark_polys,
            mask_values, mask_polys,
            constraint_degrees: degrees,
        };
    }

    #[cfg(test)]
    pub fn ctx_depth(&self) -> usize {
        return self.ctx_depth;
    }

    #[cfg(test)]
    pub fn loop_depth(&self) -> usize {
        return self.loop_depth;
    }

    pub fn constraint_count(&self) -> usize {
        return self.constraint_degrees.len();
    }

    pub fn constraint_degrees(&self) -> &[usize] {
        return &self.constraint_degrees;
    }

    // EVALUATOR FUNCTIONS
    // --------------------------------------------------------------------------------------------

    /// Evaluates decoder transition constraints at the specified step of the evaluation domain and
    /// saves the evaluations into `result`.
    pub fn evaluate(&self, current: &TraceState, next: &TraceState, step: usize, result: &mut [u128])
    {
        // determine round and mask constants at the specified step
        let ark = self.ark_values[step % self.cycle_length];
        let masks = self.mask_values[step % self.cycle_length];

        // evaluate constraints for decoding op codes
        enforce_op_bits(&mut result[..NUM_OP_CONSTRAINTS], current, next, &masks);

        // evaluate constraints for flow control operations
        let result = &mut result[NUM_OP_CONSTRAINTS..];
        let op_flags = current.cf_op_flags();

        enforce_hacc (result, current, next, &ark, op_flags[FlowOps::Hacc.op_index() ]);
        enforce_begin(result, current, next,       op_flags[FlowOps::Begin.op_index()]);
        enforce_tend (result, current, next,       op_flags[FlowOps::Tend.op_index() ]);
        enforce_fend (result, current, next,       op_flags[FlowOps::Fend.op_index() ]);
        enforce_loop (result, current, next,       op_flags[FlowOps::Loop.op_index() ]);
        enforce_wrap (result, current, next,       op_flags[FlowOps::Wrap.op_index() ]);
        enforce_break(result, current, next,       op_flags[FlowOps::Break.op_index()]);
        enforce_void (result, current, next,       op_flags[FlowOps::Void.op_index() ]);
    }

    /// Evaluates decoder transition constraints at the specified x coordinate and saves the
    /// evaluations into `result`. Unlike the function above, this function can evaluate constraints
    /// at any out-of-domain point, but it is much slower than the previous function.
    pub fn evaluate_at(&self, current: &TraceState, next: &TraceState, x: u128, result: &mut [u128])
    {
        // map x to the corresponding coordinate in constant cycles
        let num_cycles = (self.trace_length / BASE_CYCLE_LENGTH) as u128;
        let x = field::exp(x, num_cycles);

        // determine round constants at the specified x coordinate
        let mut ark = [field::ZERO; 2 * SPONGE_WIDTH];
        for i in 0..ark.len() {
            ark[i] = polynom::eval(&self.ark_polys[i], x);
        }

        // determine mask constants at the specified x coordinate
        let mut masks = [field::ZERO; 3];
        for i in 0..masks.len() {
            masks[i] = polynom::eval(&self.mask_polys[i], x);
        }

        // evaluate constraints for decoding op codes
        enforce_op_bits(&mut result[..NUM_OP_CONSTRAINTS], current, next, &masks);

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
fn transpose_ark_constants(constants: Vec<Vec<u128>>, cycle_length: usize) -> Vec<[u128; 2 * SPONGE_WIDTH]>
{
    let mut values = Vec::new();
    for i in 0..cycle_length {
        values.push([field::ZERO; 2 * SPONGE_WIDTH]);
        for j in 0..(2 * SPONGE_WIDTH) {
            values[i][j] = constants[j][i];
        }
    }
    return values;
}

fn transpose_mask_constants(constants: Vec<Vec<u128>>, cycle_length: usize) -> Vec<[u128; 3]>
{
    let mut values = Vec::new();
    for i in 0..cycle_length {
        values.push([field::ZERO; 3]);
        for j in 0..3 {
            values[i][j] = constants[j][i];
        }
    }
    return values;
}

// CYCLE MASKS
// ================================================================================================
const MASKS: [[u128; BASE_CYCLE_LENGTH]; 3] = [
    [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],   // multiples of 16
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],   // one less than multiple of 16
    [0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1],   // multiples of 8
];