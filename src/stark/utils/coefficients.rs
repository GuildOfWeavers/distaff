use crate::math::{ FiniteField, FieldElement };
use crate::stark::{ MAX_REGISTER_COUNT, MAX_INPUTS, MAX_OUTPUTS, MAX_TRANSITION_CONSTRAINTS };

// CONSTANTS
// ================================================================================================
const NUM_CONSTRAINTS: usize = MAX_INPUTS + MAX_OUTPUTS + MAX_TRANSITION_CONSTRAINTS;

// TYPES AND INTERFACES
// ================================================================================================
pub struct ConstraintCoefficients<T>
    where T: FieldElement + FiniteField<T>
{
    pub inputs      : [T; 2 * MAX_INPUTS],
    pub outputs     : [T; 2 * MAX_OUTPUTS],
    pub transition  : [T; 2 * MAX_TRANSITION_CONSTRAINTS],
}

pub struct CompositionCoefficients<T>
    where T: FieldElement + FiniteField<T>
{
    pub trace1      : [T; 2 * MAX_REGISTER_COUNT],
    pub trace2      : [T; 2 * MAX_REGISTER_COUNT],
    pub t1_degree   : T,
    pub t2_degree   : T,
    pub constraints : T,
}

// IMPLEMENTATIONS
// ================================================================================================
impl <T> ConstraintCoefficients<T>
    where T: FieldElement + FiniteField<T>
{
    pub fn new(seed: [u8; 32]) -> ConstraintCoefficients<T> {

        // generate a pseudo-random list of coefficients
        let coefficients = T::prng_vector(seed, 2 * NUM_CONSTRAINTS);
        
        // copy coefficients to their respective segments
        let end_index = 2 * MAX_INPUTS;
        let mut inputs = [T::ZERO; 2 * MAX_INPUTS];
        inputs.copy_from_slice(&coefficients[..end_index]);

        let start_index = end_index;
        let end_index = start_index + 2 * MAX_OUTPUTS;
        let mut outputs = [T::ZERO; 2 * MAX_OUTPUTS];
        outputs.copy_from_slice(&coefficients[start_index..end_index]);

        let start_index = end_index;
        let mut transition = [T::ZERO; 2 * MAX_TRANSITION_CONSTRAINTS];
        transition.copy_from_slice(&coefficients[start_index..]);

        return ConstraintCoefficients { inputs, outputs, transition };
    }
}

impl <T> CompositionCoefficients<T>
    where T: FieldElement + FiniteField<T>
{
    pub fn new(seed: [u8; 32]) -> CompositionCoefficients<T> {
        // generate a pseudo-random list of coefficients
        let coefficients = T::prng_vector(seed, 1 + 4 * MAX_REGISTER_COUNT + 3);

        // skip the first value because it is used up by deep point z
        let start_index = 1;

        // copy coefficients to their respective segments
        let end_index = start_index + 2 * MAX_REGISTER_COUNT;
        let mut trace1 = [T::ZERO; 2 * MAX_REGISTER_COUNT];
        trace1.copy_from_slice(&coefficients[start_index..end_index]);

        let start_index = end_index;
        let end_index = start_index + 2 * MAX_REGISTER_COUNT;
        let mut trace2 = [T::ZERO; 2 * MAX_REGISTER_COUNT];
        trace2.copy_from_slice(&coefficients[start_index..end_index]);

        let index = end_index;
        let t1_degree = coefficients[index];
        let t2_degree = coefficients[index + 1];
        let constraints = coefficients[index + 2];

        return CompositionCoefficients { trace1, trace2, t1_degree, t2_degree, constraints };
    }
}