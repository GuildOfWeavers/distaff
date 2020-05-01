use crate::math::field;
use crate::stark::{ MAX_REGISTER_COUNT, MAX_INPUTS, MAX_OUTPUTS };
use crate::utils::{ CopyInto };
use super::{ MAX_TRANSITION_CONSTRAINTS };

// TYPES AND INTERFACES
// ================================================================================================
pub struct CompositionCoefficients {
    pub inputs      : [u64; 2 * MAX_INPUTS],
    pub outputs     : [u64; 2 * MAX_OUTPUTS],
    pub transition  : [u64; 2 * MAX_TRANSITION_CONSTRAINTS],
    pub trace       : [u64; 2 * MAX_REGISTER_COUNT],
}

// IMPLEMENTATIONS
// ================================================================================================
impl CompositionCoefficients {

    pub fn new(trace_root: &[u64; 4]) -> CompositionCoefficients {

        // generate a pseudo-random list of coefficients
        let coefficients = field::prng_vector(trace_root.copy_into(),
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

        return CompositionCoefficients { inputs, outputs, transition, trace };
    }
}