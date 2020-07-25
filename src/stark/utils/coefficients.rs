use std::{ ops::Range };
use crate::{
    math::field,
    utils::RangeSlider,
    MAX_REGISTER_COUNT, MAX_PUBLIC_INPUTS,
    SPONGE_WIDTH,
    MAX_CONTEXT_DEPTH, MAX_LOOP_DEPTH, MAX_STACK_DEPTH,
    MIN_CONTEXT_DEPTH, MIN_LOOP_DEPTH, MIN_STACK_DEPTH,
    NUM_CF_OP_BITS, NUM_LD_OP_BITS, NUM_HD_OP_BITS,
    stark::constraints::{ NUM_STATIC_DECODER_CONSTRAINTS, NUM_AUX_STACK_CONSTRAINTS },
};

// CONSTANTS
// ================================================================================================
const NUM_OP_BITS: usize = NUM_CF_OP_BITS + NUM_LD_OP_BITS + NUM_HD_OP_BITS;
const MAX_USER_STACK_IO_CONSTRAINTS: usize = MAX_PUBLIC_INPUTS; // same as MAX_OUTPUTS
const NUM_BOUNDARY_CONSTRAINTS: usize =
    1   // for op_counter
    + SPONGE_WIDTH
    + NUM_OP_BITS
    + MAX_CONTEXT_DEPTH
    + MAX_LOOP_DEPTH
    + MAX_USER_STACK_IO_CONSTRAINTS;

const NUM_TRANSITION_CONSTRAINTS: usize =
    0 
    + NUM_STATIC_DECODER_CONSTRAINTS
    + MAX_CONTEXT_DEPTH
    + MAX_LOOP_DEPTH
    + MAX_STACK_DEPTH
    + NUM_AUX_STACK_CONSTRAINTS;

const NUM_CONSTRAINTS: usize = NUM_TRANSITION_CONSTRAINTS + 2 * NUM_BOUNDARY_CONSTRAINTS;

// TYPES AND INTERFACES
// ================================================================================================
pub struct ConstraintCoefficients {
    pub i_boundary  : BoundaryCoefficients,
    pub f_boundary  : BoundaryCoefficients,
    pub transition  : Vec<u128>,
}

pub struct BoundaryCoefficients {
    pub op_counter  : [u128; 2],
    pub sponge      : [u128; SPONGE_WIDTH * 2],
    pub op_bits     : [u128; NUM_OP_BITS * 2],
    pub ctx_stack   : [u128; MAX_CONTEXT_DEPTH * 2],
    pub loop_stack  : [u128; MAX_LOOP_DEPTH * 2],
    pub user_stack  : [u128; MAX_USER_STACK_IO_CONSTRAINTS * 2],
}

pub struct CompositionCoefficients {
    pub trace1      : [u128; 2 * MAX_REGISTER_COUNT],
    pub trace2      : [u128; 2 * MAX_REGISTER_COUNT],
    pub t1_degree   : u128,
    pub t2_degree   : u128,
    pub constraints : u128,
}

// IMPLEMENTATIONS
// ================================================================================================
impl ConstraintCoefficients {
    pub fn new(seed: [u8; 32], ctx_depth: usize, loop_depth: usize, stack_depth: usize) -> ConstraintCoefficients {

        // generate a pseudo-random list of coefficients
        let coefficients = field::prng_vector(seed, 2 * NUM_CONSTRAINTS);

        // copy coefficients for boundary constraints
        let (i_boundary, i) = build_boundary_coefficients(&coefficients);
        let (f_boundary, i) = build_boundary_coefficients(&coefficients[i..]);

        // copy coefficients for transition constraints
        let transition = build_transition_coefficients(&coefficients[i..], ctx_depth, loop_depth, stack_depth);

        return ConstraintCoefficients { i_boundary, f_boundary, transition };
    }
}

impl CompositionCoefficients {
    pub fn new(seed: [u8; 32]) -> CompositionCoefficients {
        // generate a pseudo-random list of coefficients
        let coefficients = field::prng_vector(seed, 1 + 4 * MAX_REGISTER_COUNT + 3);

        // skip the first value because it is used up by deep point z
        let start_index = 1;

        // copy coefficients to their respective segments
        let end_index = start_index + 2 * MAX_REGISTER_COUNT;
        let mut trace1 = [field::ZERO; 2 * MAX_REGISTER_COUNT];
        trace1.copy_from_slice(&coefficients[start_index..end_index]);

        let start_index = end_index;
        let end_index = start_index + 2 * MAX_REGISTER_COUNT;
        let mut trace2 = [field::ZERO; 2 * MAX_REGISTER_COUNT];
        trace2.copy_from_slice(&coefficients[start_index..end_index]);

        let index = end_index;
        let t1_degree = coefficients[index];
        let t2_degree = coefficients[index + 1];
        let constraints = coefficients[index + 2];

        return CompositionCoefficients { trace1, trace2, t1_degree, t2_degree, constraints };
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn build_boundary_coefficients(coefficients: &[u128]) -> (BoundaryCoefficients, usize)
{
    let mut result = BoundaryCoefficients {
        op_counter  : [0; 2],
        sponge      : [0; SPONGE_WIDTH * 2],
        op_bits     : [0; NUM_OP_BITS * 2],
        ctx_stack   : [0; MAX_CONTEXT_DEPTH * 2],
        loop_stack  : [0; MAX_LOOP_DEPTH * 2],
        user_stack  : [0; MAX_USER_STACK_IO_CONSTRAINTS * 2],
    };

    let mut range: Range<usize> = Range { start: 0, end: 2 };
    result.op_counter.copy_from_slice(&coefficients[range.clone()]);

    range = range.slide(SPONGE_WIDTH * 2);
    result.sponge.copy_from_slice(&coefficients[range.clone()]);

    range = range.slide(NUM_OP_BITS * 2);
    result.op_bits.copy_from_slice(&coefficients[range.clone()]);

    range = range.slide(MAX_CONTEXT_DEPTH * 2);
    result.ctx_stack.copy_from_slice(&coefficients[range.clone()]);

    range = range.slide(MAX_LOOP_DEPTH * 2);
    result.loop_stack.copy_from_slice(&coefficients[range.clone()]);

    range = range.slide(MAX_USER_STACK_IO_CONSTRAINTS * 2);
    result.user_stack.copy_from_slice(&coefficients[range.clone()]);

    return (result, range.end);
}

fn build_transition_coefficients(coefficients: &[u128], ctx_depth: usize, loop_depth: usize, stack_depth: usize) -> Vec<u128>{

    let ctx_depth = std::cmp::max(ctx_depth, MIN_CONTEXT_DEPTH);
    let loop_depth = std::cmp::max(loop_depth, MIN_LOOP_DEPTH);
    let stack_depth = std::cmp::max(stack_depth, MIN_STACK_DEPTH);

    // compute number of used transition constraints
    let num_constraints = NUM_STATIC_DECODER_CONSTRAINTS
        + ctx_depth
        + loop_depth
        + stack_depth
        + NUM_AUX_STACK_CONSTRAINTS;

    // we need 2 coefficients per constraint
    let mut result = vec![0; num_constraints * 2];

    // copy coefficients to the result vector skipping over coefficients for unused constraints;
    // the order of constraints is assumed to be:
    // 1. static decoder constraints (e.g. op counter, op bit constraints, sponge constraints etc.)
    // 2. context stack constraints - the number depends on the actual context depth
    // 3. loop stack constraints - the number depends on the actual loop depth
    // 4. aux stack constraints
    // 5. user stack constraints - the number depends on the actual stack depth

    let mut s_range = new_range(0, NUM_STATIC_DECODER_CONSTRAINTS * 2);
    let mut t_range = new_range(0, NUM_STATIC_DECODER_CONSTRAINTS * 2);
    result[t_range.clone()].copy_from_slice(&coefficients[s_range.clone()]);

    s_range = s_range.slide(ctx_depth * 2);
    t_range = t_range.slide(ctx_depth * 2);
    result[t_range.clone()].copy_from_slice(&coefficients[s_range.clone()]);
    
    s_range = new_range(s_range.start + MAX_CONTEXT_DEPTH * 2, loop_depth * 2);
    t_range = t_range.slide(loop_depth * 2);
    result[t_range.clone()].copy_from_slice(&coefficients[s_range.clone()]);

    s_range = new_range(s_range.start + MAX_LOOP_DEPTH * 2, NUM_AUX_STACK_CONSTRAINTS * 2);
    t_range = t_range.slide(NUM_AUX_STACK_CONSTRAINTS * 2);
    result[t_range.clone()].copy_from_slice(&coefficients[s_range.clone()]);

    s_range = s_range.slide(stack_depth * 2);
    t_range = t_range.slide(stack_depth * 2);
    result[t_range.clone()].copy_from_slice(&coefficients[s_range.clone()]);

    return result;
}

fn new_range(start: usize, length: usize) -> Range<usize> {
    return Range { start, end: start + length };
}