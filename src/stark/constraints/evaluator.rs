use crate::math::field;
use crate::stark::TraceState;
use crate::utils::uninit_vector;
use super::{ decoder::Decoder, stack::Stack };

// CONSTANTS
// ================================================================================================
pub const MAX_CONSTRAINT_DEGREE: usize = 8;

// TYPES AND INTERFACES
// ================================================================================================
pub struct Evaluator {
    decoder         : Decoder,
    stack           : Stack,

    coefficients    : Coefficients,
    domain_size     : usize,
    extension_factor: usize,

    t_constraint_num: usize,
    t_degree_groups : Vec<(u64, Vec<usize>)>,
    t_evaluations   : Vec<Vec<u64>>,

    b_constraint_num: usize,
    program_hash    : [u64; 4],
    inputs          : Vec<u64>,
    outputs         : Vec<u64>,
    b_degree_incr   : u64,
}

pub struct Coefficients {
    pub trace       : [u64; 256],
    pub transition  : [u64; 256],
    pub inputs      : [u64; 32],
    pub outputs     : [u64; 32],
}

// EVALUATOR IMPLEMENTATION
// ================================================================================================
impl Evaluator {

    pub fn new(
        trace_root  : &[u64; 4],
        trace_length: usize,
        stack_depth : usize,
        ext_factor  : usize,
        program_hash: &[u64; 4],
        inputs      : &[u64],
        outputs     : &[u64]) -> Evaluator
    {
        let domain_size = trace_length * ext_factor;

        // put together degrees of all transition constraints
        let t_constraint_degrees = [
            Decoder::constraint_degrees(),
            Stack::constraint_degrees(stack_depth)
        ].concat();

        // if we are in debug mode, initialize vectors to hold individual evaluations
        // of transition constraints
        let t_evaluations = if cfg!(debug_assertions) {
            t_constraint_degrees[..].iter().map(|_| uninit_vector(domain_size)).collect()
        }
        else {
            Vec::new()
        };

        return Evaluator {
            decoder         : Decoder::new(ext_factor),
            stack           : Stack::new(stack_depth),
            coefficients    : Coefficients::new(trace_root),
            domain_size     : domain_size,
            extension_factor: ext_factor,
            t_constraint_num: t_constraint_degrees.len(),
            t_degree_groups : group_transition_constraints(t_constraint_degrees, trace_length),
            t_evaluations   : t_evaluations,
            b_constraint_num: inputs.len() + outputs.len() + program_hash.len(),
            program_hash    : *program_hash,
            inputs          : inputs.to_vec(),
            outputs         : outputs.to_vec(),
            b_degree_incr   : get_incremental_constraint_degree(1, trace_length),
        };
    }

    pub fn constraint_count(&self) -> usize {
        return self.t_constraint_num + self.b_constraint_num;
    }

    pub fn domain_size(&self) -> usize {
        return self.domain_size;
    }

    pub fn trace_length(&self) -> usize {
        return self.domain_size / self.extension_factor;
    }

    pub fn composition_degree(&self) -> usize {
        return (MAX_CONSTRAINT_DEGREE - 1) * self.trace_length() - 1;
    }

    // CONSTRAINT EVALUATORS
    // -------------------------------------------------------------------------------------------

    /// Computes pseudo-random linear combination of transition constraints D_i at point x as:
    /// cc_{i * 2} * D_i + cc_{i * 2 + 1} * D_i * x^p for all i, where cc_j are the coefficients
    /// used in the linear combination and x^p is a degree adjustment factor (different for each degree).
    pub fn evaluate_transition(&self, current: &TraceState, next: &TraceState, x: u64, step: usize) -> u64 {
        
        // evaluate transition constraints
        let mut evaluations = vec![0; self.t_constraint_num];
        self.decoder.evaluate(&current, &next, step, &mut evaluations);
        self.stack.evaluate(&current, &next, &mut evaluations[self.decoder.constraint_count()..]);

        // if the constraints should evaluate to all zeros at this step,
        // make sure they do, and return
        if self.should_evaluate_to_zero_at(step) {
            let step = step / self.extension_factor;
            for i in 0..evaluations.len() {
                assert!(evaluations[i] == 0, "transition constraint at step {} didn't evaluate to 0", step);
            }
            return 0;
        }

        // compute a pseudo-random linear combination of all transition constraints
        let cc = self.coefficients.transition;
        let mut result = 0;
        
        let mut i = 0;
        for (incremental_degree, constraints) in self.t_degree_groups.iter() {

            // for each group of constraints with the same degree, separately compute
            // combinations of D(x) and D(x) * x^p
            let mut result_adj = 0;
            for &constraint_idx in constraints.iter() {
                let evaluation = evaluations[constraint_idx];
                result = field::add(result, field::mul(evaluation, cc[i * 2]));
                result_adj = field::add(result_adj, field::mul(evaluation, cc[i * 2 + 1]));
                i += 1;
            }

            // increase the degree of D(x) * x^p
            let xp = field::exp(x, *incremental_degree);
            result = field::add(result, field::mul(result_adj, xp));
        }

        // if we are in debug mode, record all constraint evaluations
        if cfg!(debug_assertions) {
            let mutable_self = unsafe { &mut *(self as *const _ as *mut Evaluator) };
            for i in 0..evaluations.len() {
                mutable_self.t_evaluations[i][step] = evaluations[i];
            }
        }

        return result;
    }

    /// Computes pseudo-random linear combination of boundary constraints B_i at point x 
    /// separately for input and output constraints; the constraints are computed as:
    /// cc_{i * 2} * B_i + cc_{i * 2 + 1} * B_i * x^p for all i, where cc_j are the coefficients
    /// used in the linear combination and x^p is a degree adjustment factor.
    pub fn evaluate_boundaries(&self, current: &TraceState, x: u64) -> (u64, u64) {
        
        let cc = self.coefficients.inputs;
        let stack = current.get_stack();
        
        // compute adjustment factor
        let xp = field::mul(x, self.b_degree_incr);

        // 1 ----- compute combination of input constraints ---------------------------------------
        let mut i_result = 0;
        let mut result_adj = 0;

        // separately compute P(x) - input for adjusted and un-adjusted terms
        for i in 0..self.inputs.len() {
            let val = field::sub(stack[i], self.inputs[i]);
            i_result = field::add(i_result, field::mul(val, cc[i * 2]));
            result_adj = field::add(result_adj, field::mul(val, cc[i * 2 + 1]));
        }

        // raise the degree of adjusted terms and sum all the terms together
        i_result = field::add(i_result, field::mul(result_adj, xp));

        // 2 ----- compute combination of output constraints ---------------------------------------
        let mut f_result = 0;
        let mut result_adj = 0;

        // separately compute P(x) - output for adjusted and un-adjusted terms
        for i in 0..self.outputs.len() {
            let val = field::sub(stack[i], self.outputs[i]);
            f_result = field::add(f_result, field::mul(val, cc[i * 2]));
            result_adj = field::add(result_adj, field::mul(val, cc[i * 2 + 1]));
        }

        // raise the degree of adjusted terms and sum all the terms together
        f_result = field::add(f_result, field::mul(result_adj, xp));

        // 3 ----- compute combination of program hash constraints --------------------------------
        let mut result_adj = 0;

        // because we check program hash at the last step, we add the constraints to the
        // constraint evaluations to the output constraint combination
        let program_hash = current.get_program_hash();
        for i in 0..self.program_hash.len() {
            let val = field::sub(program_hash[i], self.program_hash[i]);
            f_result = field::add(f_result, field::mul(val, cc[i * 2]));
            result_adj = field::add(result_adj, field::mul(val, cc[i * 2 + 1]));
        }

        f_result = field::add(f_result, field::mul(result_adj, xp));

        return (i_result, f_result);
    }

    /// Computes a pseudo-random linear combination of all trace registers P_i at point x as:
    /// cc_{i * 2} * P_i + cc_{i * 2 + 1} * P_i * x^p for all i, where cc_j are the coefficients
    /// used in the linear combination and x^p is a degree adjustment factor.
    pub fn combine_trace_registers(&self, current: &TraceState, x: u64) -> u64 {
        
        let cc = self.coefficients.trace;

        let mut result_raw = 0;
        let mut result_adj = 0;

        // separately sum up adjusted and un-adjusted terms
        let registers = current.registers();
        for i in 0..registers.len() {
            result_raw = field::add(result_raw, field::mul(registers[i], cc[i * 2]));
            result_adj = field::add(result_adj, field::mul(registers[i], cc[i * 2 + 1]));
        }

        // multiply adjusted terms by degree adjustment factor
        let xp = field::mul(x, self.b_degree_incr);
        result_adj = field::mul(result_adj, xp);

        // sum both parts together and return
        return field::add(result_raw, result_adj);
    }

    // HELPER METHODS
    // -------------------------------------------------------------------------------------------
    fn should_evaluate_to_zero_at(&self, step: usize) -> bool {
        return (step & (self.extension_factor - 1) == 0) // same as: step % extension_factor == 0
            && (step != self.domain_size - self.extension_factor);
    }
}

// COEFFICIENTS IMPLEMENTATION
// ================================================================================================
impl Coefficients {

    pub fn new(seed: &[u64; 4]) -> Coefficients {

        // generate a pseudo-random list of coefficients
        let seed = unsafe { &*(seed as *const _ as *const [u8; 32]) };
        let coefficients = field::prng_vector(*seed, 256 + 256 + 32 + 32);
        
        // copy coefficients to their respective segments
        let mut trace = [0u64; 256];
        trace.copy_from_slice(&coefficients[0..256]);

        let mut transition = [0u64; 256];
        transition.copy_from_slice(&coefficients[256..512]);

        let mut inputs = [0u64; 32];
        inputs.copy_from_slice(&coefficients[512..544]);

        let mut outputs = [0u64; 32];
        outputs.copy_from_slice(&coefficients[544..576]);

        return Coefficients { trace, transition, inputs, outputs };

    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn group_transition_constraints(degrees: Vec<usize>, trace_length: usize) -> Vec<(u64, Vec<usize>)> {
    let mut groups = [
        Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
    ];

    for (i, &degree) in degrees.iter().enumerate() {
        groups[degree].push(i);
    }

    let mut result = Vec::new();
    for (degree, constraints) in groups.iter().enumerate() {
        if constraints.len() == 0 { continue; }
        let incremental_degree = get_incremental_constraint_degree(degree, trace_length);
        result.push((incremental_degree, constraints.clone()));
    }

    return result;
}

fn get_incremental_constraint_degree(degree: usize, trace_length: usize) -> u64 {
    let target_degree = trace_length * MAX_CONSTRAINT_DEGREE - 1;
    let constraint_degree = (trace_length - 1) * degree;
    return (target_degree - constraint_degree - 1) as u64;
}