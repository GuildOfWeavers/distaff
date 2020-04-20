use crate::math::field;
use crate::stark::TraceState;
use crate::utils::uninit_vector;
use super::{ decoder, stack, hash_acc };

// CONSTANTS
// ================================================================================================
pub const MAX_CONSTRAINT_DEGREE: usize = 8;

// TYPES AND INTERFACES
// ================================================================================================
pub struct Evaluator {
    coefficients    : Coefficients,
    domain_size     : usize,
    stack_depth     : usize,

    t_constraint_deg: Vec<usize>,
    t_adjustment_deg: Vec<u64>,
    t_evaluations   : Vec<Vec<u64>>,

    inputs          : Vec<u64>,
    outputs         : Vec<u64>,
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

    pub fn new(trace_root: &[u64; 4], trace_length: usize, stack_depth: usize, inputs: &[u64], outputs: &[u64]) -> Evaluator {

        let domain_size = trace_length * MAX_CONSTRAINT_DEGREE;

        let t_constraint_deg = get_transition_constraint_degrees(stack_depth);
        let t_adjustment_deg = t_constraint_deg[..].iter().map(|&d| 
            get_incremental_constraint_degree(d, trace_length)).collect();

        let t_evaluations = if cfg!(debug_assertions) {
            t_constraint_deg[..].iter().map(|_| uninit_vector(domain_size)).collect()
        }
        else {
            Vec::new()
        };

        return Evaluator {
            coefficients    : Coefficients::new(trace_root),
            domain_size     : domain_size,
            stack_depth     : stack_depth,
            t_constraint_deg: t_constraint_deg,
            t_adjustment_deg: t_adjustment_deg,
            t_evaluations   : t_evaluations,
            inputs          : inputs.to_vec(),
            outputs         : outputs.to_vec(),
        };
    }

    pub fn constraint_count(&self) -> usize {
        return self.t_constraint_deg.len() + self.inputs.len() + self.outputs.len();
    }

    pub fn domain_size(&self) -> usize {
        return self.domain_size;
    }

    pub fn trace_length(&self) -> usize {
        return self.domain_size / MAX_CONSTRAINT_DEGREE;
    }

    pub fn composition_degree(&self) -> usize {
        return self.domain_size - self.trace_length() - 1;
    }

    // TRANSITION CONSTRAINTS
    // -------------------------------------------------------------------------------------------
    pub fn evaluate_transition(&self, current: &TraceState, next: &TraceState, x: u64, step: usize) -> u64 {
        let mut result = 0;

        let op_dec = decoder::evaluate(&current, &next);
        let op_acc = hash_acc::evaluate(&current, &next, step);
        let stack = stack::evaluate(&current, &next, self.stack_depth);
        let evaluations = [ &op_dec[..], &op_acc[..], &stack[..self.stack_depth] ].concat(); // TODO: build more efficiently

        let should_be_zero = (step % MAX_CONSTRAINT_DEGREE == 0) && (step < self.domain_size - MAX_CONSTRAINT_DEGREE);

        if should_be_zero {
            for i in 0..evaluations.len() {
                assert!(evaluations[i] == 0, "transition constraint at step {} didn't evaluate to 0", step / MAX_CONSTRAINT_DEGREE);
            }
        }
        else {
            let mut x_powers = [0u64; MAX_CONSTRAINT_DEGREE + 1];   // TODO: group constraints by degree
            let cc = self.coefficients.transition;
    
            for i in 0..evaluations.len() {
                result = field::add(result, field::mul(evaluations[i], cc[i * 2]));
    
                if x_powers[self.t_constraint_deg[i]] == 0 {
                    x_powers[self.t_constraint_deg[i]] = field::exp(x, self.t_adjustment_deg[i]);
                }
                let xp = x_powers[self.t_constraint_deg[i]];
                let adj_eval = field::mul(evaluations[i], xp);
                result = field::add(result, field::mul(adj_eval, cc[i * 2 + 1]));
            }
        }

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
        let adj_degree = get_incremental_constraint_degree(1, self.trace_length()); // TODO: cache
        let xp = field::mul(x, adj_degree);

        // 1 ----- compute combination of input constraints ---------------------------------------
        let mut result_raw = 0;
        let mut result_adj = 0;

        // separately compute P(x) - input for adjusted and un-adjusted terms
        for i in 0..self.inputs.len() {
            let val = field::sub(stack[i], self.inputs[i]);
            result_raw = field::add(result_raw, field::mul(val, cc[i * 2]));
            result_adj = field::add(result_adj, field::mul(val, cc[i * 2 + 1]));
        }

        // raise the degree of adjusted terms and sum all the terms together
        result_adj = field::mul(result_adj, xp);
        let i_result = field::add(result_raw, result_adj);

        // 2 ----- compute combination of output constraints ---------------------------------------
        let mut result_raw = 0;
        let mut result_adj = 0;

        // separately compute P(x) - output for adjusted and un-adjusted terms
        for i in 0..self.outputs.len() {
            let val = field::sub(stack[i], self.outputs[i]);
            result_raw = field::add(result_raw, field::mul(val, cc[i * 2]));
            result_adj = field::add(result_adj, field::mul(val, cc[i * 2 + 1]));
        }

        // raise the degree of adjusted terms and sum all the terms together
        result_adj = field::mul(result_adj, xp);
        let o_result = field::add(result_raw, result_adj);

        return (i_result, o_result);
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
        let adj_degree = get_incremental_constraint_degree(1, self.trace_length()); // TODO: cache
        let xp = field::mul(x, adj_degree);
        result_adj = field::mul(result_adj, xp);

        // sum both parts together and return
        return field::add(result_raw, result_adj);
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
fn get_transition_constraint_degrees(stack_depth: usize) -> Vec<usize> {
    let degrees = [
        &decoder::CONSTRAINT_DEGREES[..],
        &hash_acc::CONSTRAINT_DEGREES[..],
        &stack::CONSTRAINT_DEGREES[..stack_depth]
    ].concat();
    return degrees;
}

fn get_incremental_constraint_degree(degree: usize, trace_length: usize) -> u64 {
    let target_degree = trace_length * MAX_CONSTRAINT_DEGREE - 1;
    let constraint_degree = (trace_length - 1) * degree;
    return (target_degree - constraint_degree - 1) as u64;
}