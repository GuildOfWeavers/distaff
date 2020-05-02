use crate::math::field;
use crate::stark::{ MAX_REGISTER_COUNT, MAX_INPUTS, MAX_OUTPUTS, MAX_TRANSITION_CONSTRAINTS };
use crate::utils::{ CopyInto };

// TYPES AND INTERFACES
// ================================================================================================
pub struct ConstraintCoefficients {
    pub inputs      : [u64; 2 * MAX_INPUTS],
    pub outputs     : [u64; 2 * MAX_OUTPUTS],
    pub transition  : [u64; 2 * MAX_TRANSITION_CONSTRAINTS],
    pub trace       : [u64; 2 * MAX_REGISTER_COUNT],    // TODO: remove
}

pub struct CompositionCoefficients {
    pub trace1      : [u64; 2 * MAX_REGISTER_COUNT],
    pub trace2      : [u64; 2 * MAX_REGISTER_COUNT],
    pub t1_degree   : u64,
    pub t2_degree   : u64,
    pub constraints : u64,
}

// IMPLEMENTATIONS
// ================================================================================================
impl ConstraintCoefficients {

    pub fn new(seed: &[u64; 4]) -> ConstraintCoefficients {

        // generate a pseudo-random list of coefficients
        let coefficients = field::prng_vector(seed.copy_into(),
            2 * (MAX_INPUTS + MAX_OUTPUTS + MAX_TRANSITION_CONSTRAINTS + MAX_REGISTER_COUNT));
        
        // copy coefficients to their respective segments
        let end_index = 2 * MAX_INPUTS;
        let mut inputs = [0u64; 2 * MAX_INPUTS];
        inputs.copy_from_slice(&coefficients[..end_index]);

        let start_index = end_index;
        let end_index = start_index + 2 * MAX_OUTPUTS;
        let mut outputs = [0u64; 2 * MAX_OUTPUTS];
        outputs.copy_from_slice(&coefficients[start_index..end_index]);

        let start_index = end_index;
        let end_index = start_index + 2 * MAX_TRANSITION_CONSTRAINTS;
        let mut transition = [0u64; 2 * MAX_TRANSITION_CONSTRAINTS];
        transition.copy_from_slice(&coefficients[start_index..end_index]);

        let start_index = end_index;
        let mut trace = [0u64; 2 * MAX_REGISTER_COUNT];
        trace.copy_from_slice(&coefficients[start_index..]);

        return ConstraintCoefficients { inputs, outputs, transition, trace };
    }
}

impl CompositionCoefficients {

    pub fn new(seed: &[u64; 4]) -> CompositionCoefficients {
        // generate a pseudo-random list of coefficients
        let coefficients = field::prng_vector(seed.copy_into(), 4 * MAX_REGISTER_COUNT + 3);

        // copy coefficients to their respective segments
        let end_index = 2 * MAX_REGISTER_COUNT;
        let mut trace1 = [0u64; 2 * MAX_REGISTER_COUNT];
        trace1.copy_from_slice(&coefficients[..end_index]);

        let start_index = end_index;
        let end_index = start_index + 2 * MAX_REGISTER_COUNT;
        let mut trace2 = [0u64; 2 * MAX_REGISTER_COUNT];
        trace2.copy_from_slice(&coefficients[start_index..end_index]);

        let index = end_index;
        let t1_degree = coefficients[index];
        let t2_degree = coefficients[index + 1];
        let constraints = coefficients[index + 2];

        return CompositionCoefficients { trace1, trace2, t1_degree, t2_degree, constraints };
    }
}