use crate::math::{ FiniteField };
use crate::{ MAX_REGISTER_COUNT, MAX_PUBLIC_INPUTS, MAX_OUTPUTS };
use crate::stark::MAX_TRANSITION_CONSTRAINTS;

// CONSTANTS
// ================================================================================================
const DECODER_WIDTH: usize = 10;
const NUM_CONSTRAINTS: usize = MAX_PUBLIC_INPUTS + MAX_OUTPUTS + MAX_TRANSITION_CONSTRAINTS + 2 * DECODER_WIDTH;

// TYPES AND INTERFACES
// ================================================================================================
pub struct ConstraintCoefficients {
    pub i_boundary  : [u128; 2 * (DECODER_WIDTH + MAX_PUBLIC_INPUTS) ],
    pub f_boundary  : [u128; 2 * (DECODER_WIDTH + MAX_OUTPUTS)],
    pub transition  : [u128; 2 * MAX_TRANSITION_CONSTRAINTS],
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
    pub fn new(seed: [u8; 32]) -> ConstraintCoefficients {

        // generate a pseudo-random list of coefficients
        let coefficients = u128::prng_vector(seed, 2 * NUM_CONSTRAINTS);

        // copy coefficients to their respective segments
        let end_index = 2 * (DECODER_WIDTH + MAX_PUBLIC_INPUTS);
        let mut i_boundary = [u128::ZERO; 2 * (DECODER_WIDTH + MAX_PUBLIC_INPUTS)];
        i_boundary.copy_from_slice(&coefficients[..end_index]);

        let start_index = end_index;
        let end_index = start_index + 2 * (DECODER_WIDTH + MAX_OUTPUTS);
        let mut f_boundary = [u128::ZERO; 2 * (DECODER_WIDTH + MAX_OUTPUTS)];
        f_boundary.copy_from_slice(&coefficients[start_index..end_index]);

        let start_index = end_index;
        let mut transition = [u128::ZERO; 2 * MAX_TRANSITION_CONSTRAINTS];
        transition.copy_from_slice(&coefficients[start_index..]);

        return ConstraintCoefficients { i_boundary, f_boundary, transition };
    }
}

impl CompositionCoefficients {
    pub fn new(seed: [u8; 32]) -> CompositionCoefficients {
        // generate a pseudo-random list of coefficients
        let coefficients = u128::prng_vector(seed, 1 + 4 * MAX_REGISTER_COUNT + 3);

        // skip the first value because it is used up by deep point z
        let start_index = 1;

        // copy coefficients to their respective segments
        let end_index = start_index + 2 * MAX_REGISTER_COUNT;
        let mut trace1 = [u128::ZERO; 2 * MAX_REGISTER_COUNT];
        trace1.copy_from_slice(&coefficients[start_index..end_index]);

        let start_index = end_index;
        let end_index = start_index + 2 * MAX_REGISTER_COUNT;
        let mut trace2 = [u128::ZERO; 2 * MAX_REGISTER_COUNT];
        trace2.copy_from_slice(&coefficients[start_index..end_index]);

        let index = end_index;
        let t1_degree = coefficients[index];
        let t2_degree = coefficients[index + 1];
        let constraints = coefficients[index + 2];

        return CompositionCoefficients { trace1, trace2, t1_degree, t2_degree, constraints };
    }
}