use crate::math::{ field, polynom, fft };
use crate::processor::opcodes::{ FlowOps, UserOps };
use crate::stark::trace::{ TraceState };
use crate::utils::{ filled_vector, accumulator::ARK };
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
const BASE_CYCLE_LENGTH: usize = 16;

const CYCLE_MASK_IDX: usize = 0;
const PREFIX_MASK_IDX: usize = 1;
const PUSH_MASK_IDX: usize = 2;

// CONSTANTS
// ================================================================================================
const NUM_OP_CONSTRAINTS: usize = 14;
const OP_CONSTRAINT_DEGREES: [usize; NUM_OP_CONSTRAINTS] = [
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2,   // all op bits are binary
    7,                              // ld_ops and hd_ops cannot be all 0s
    8,                              // when cf_ops are not all 0s, ld_ops and hd_ops must be all 1s
    2,                              // VOID can be followed only by VOID
    4,                              // operations happen on allowed step multiples
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
        degrees.resize(degrees.len() + ctx_depth + loop_depth, STACK_CONSTRAINT_DEGREE);

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

    pub fn ctx_depth(&self) -> usize {
        return self.ctx_depth;
    }

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

        enforce_hacc (result, current, next, &ark, op_flags[FlowOps::Hacc as usize]);
        enforce_begin(result, current, next, op_flags[FlowOps::Begin as usize]);
        enforce_tend (result, current, next, op_flags[FlowOps::Tend as usize]);
        enforce_fend (result, current, next, op_flags[FlowOps::Fend as usize]);
        enforce_loop (result, current, next, op_flags[FlowOps::Loop as usize]);
        enforce_wrap (result, current, next, op_flags[FlowOps::Wrap as usize]);
        enforce_break(result, current, next, op_flags[FlowOps::Break as usize]);
        enforce_void (result, current, next, op_flags[FlowOps::Void as usize]);
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
fn extend_constants(constants: &[[u128; BASE_CYCLE_LENGTH]], extension_factor: usize) -> (Vec<Vec<u128>>, Vec<Vec<u128>>)
{
    let root = field::get_root_of_unity(BASE_CYCLE_LENGTH);
    let inv_twiddles = fft::get_inv_twiddles(root, BASE_CYCLE_LENGTH);

    let domain_size = BASE_CYCLE_LENGTH * extension_factor;
    let domain_root = field::get_root_of_unity(domain_size);
    let twiddles = fft::get_twiddles(domain_root, domain_size);

    let mut polys = Vec::with_capacity(constants.len());
    let mut evaluations = Vec::with_capacity(constants.len());

    for constant in constants.iter() {
        let mut extended_constant = filled_vector(BASE_CYCLE_LENGTH, domain_size, field::ZERO);
        extended_constant.copy_from_slice(constant);

        polynom::interpolate_fft_twiddles(&mut extended_constant, &inv_twiddles, true);
        polys.push(extended_constant.clone());

        unsafe { extended_constant.set_len(extended_constant.capacity()); }
        polynom::eval_fft_twiddles(&mut extended_constant, &twiddles, true);

        evaluations.push(extended_constant);
    }

    return (polys, evaluations);
}

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