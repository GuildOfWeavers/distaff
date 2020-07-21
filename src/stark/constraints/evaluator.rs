use crate::{
    math::field,
    utils::uninit_vector,
    stark::{ StarkProof, TraceTable, TraceState, ConstraintCoefficients },
    PROGRAM_DIGEST_SIZE,
};
use super::{ decoder::Decoder, stack::Stack, MAX_CONSTRAINT_DEGREE };

// TYPES AND INTERFACES
// ================================================================================================
pub struct Evaluator {
    decoder         : Decoder,
    stack           : Stack,

    coefficients    : ConstraintCoefficients,
    domain_size     : usize,
    extension_factor: usize,

    t_constraint_num: usize,
    t_degree_groups : Vec<(u128, Vec<usize>)>,
    t_evaluations   : Vec<Vec<u128>>,

    b_constraint_num: usize,
    program_hash    : Vec<u128>,
    op_count        : u128,
    inputs          : Vec<u128>,
    outputs         : Vec<u128>,
    b_degree_adj    : u128,
}

// EVALUATOR IMPLEMENTATION
// ================================================================================================
impl Evaluator {

    pub fn from_trace(trace: &TraceTable, trace_root: &[u8; 32], inputs: &[u128], outputs: &[u128]) -> Evaluator
    {
        let last_state = trace.get_last_state();

        let stack_depth = trace.stack_depth();
        let trace_length = trace.unextended_length();
        let extension_factor = MAX_CONSTRAINT_DEGREE;

        // instantiate decoder and stack constraint evaluators 
        let decoder = Decoder::new(trace_length, extension_factor, trace.ctx_depth(), trace.loop_depth());
        let stack = Stack::new(trace_length, extension_factor, stack_depth);

        // build a list of transition constraint degrees
        let t_constraint_degrees = [
            decoder.constraint_degrees(), stack.constraint_degrees()
        ].concat();

        // if we are in debug mode, initialize vectors to hold individual evaluations
        // of transition constraints
        let domain_size = trace_length * extension_factor;
        let t_evaluations = if cfg!(debug_assertions) {
            t_constraint_degrees.iter().map(|_| uninit_vector(domain_size)).collect()
        }
        else {
            Vec::new()
        };

        return Evaluator {
            decoder         : decoder,
            stack           : stack,
            coefficients    : ConstraintCoefficients::new(*trace_root),
            domain_size     : domain_size,
            extension_factor: extension_factor,
            t_constraint_num: t_constraint_degrees.len(),
            t_degree_groups : group_transition_constraints(t_constraint_degrees, trace_length),
            t_evaluations   : t_evaluations,
            b_constraint_num: get_boundary_constraint_num(&inputs, &outputs),
            program_hash    : last_state.program_hash().to_vec(),
            op_count        : last_state.op_counter(),
            inputs          : inputs.to_vec(),
            outputs         : outputs.to_vec(),
            b_degree_adj    : get_boundary_constraint_adjustment_degree(trace_length),
        };
    }

    pub fn from_proof(proof: &StarkProof, program_hash: &[u8; 32], inputs: &[u128], outputs: &[u128]) -> Evaluator
    {
        let stack_depth = proof.stack_depth();
        let trace_length = proof.trace_length();
        let extension_factor = proof.options().extension_factor();
        
        // instantiate decoder and stack constraint evaluators 
        let decoder = Decoder::new(trace_length, extension_factor, proof.ctx_depth(), proof.loop_depth());
        let stack = Stack::new(trace_length, extension_factor, stack_depth);

        // build a list of transition constraint degrees
        let t_constraint_degrees = [
            decoder.constraint_degrees(), stack.constraint_degrees()
        ].concat();

        return Evaluator {
            decoder         : decoder,
            stack           : stack,
            coefficients    : ConstraintCoefficients::new(*proof.trace_root()),
            domain_size     : proof.domain_size(),
            extension_factor: extension_factor,
            t_constraint_num: t_constraint_degrees.len(),
            t_degree_groups : group_transition_constraints(t_constraint_degrees, trace_length),
            t_evaluations   : Vec::new(),
            b_constraint_num: get_boundary_constraint_num(&inputs, &outputs),
            program_hash    : parse_program_hash(program_hash),
            op_count        : proof.op_count(),
            inputs          : inputs.to_vec(),
            outputs         : outputs.to_vec(),
            b_degree_adj    : get_boundary_constraint_adjustment_degree(trace_length),
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

    pub fn get_x_at_last_step(&self) -> u128 {
        let trace_root = field::get_root_of_unity(self.trace_length());
        return field::exp(trace_root, (self.trace_length() - 1) as u128);
    }

    // CONSTRAINT EVALUATORS
    // -------------------------------------------------------------------------------------------

    /// Computes pseudo-random linear combination of transition constraints D_i at point x as:
    /// cc_{i * 2} * D_i + cc_{i * 2 + 1} * D_i * x^p for all i, where cc_j are the coefficients
    /// used in the linear combination and x^p is a degree adjustment factor (different for each degree).
    pub fn evaluate_transition(&self, current: &TraceState, next: &TraceState, x: u128, step: usize) -> u128 {
        
        // evaluate transition constraints
        let mut evaluations = vec![field::ZERO; self.t_constraint_num];
        self.decoder.evaluate(&current, &next, step, &mut evaluations);
        self.stack.evaluate(&current, &next, step, &mut evaluations[self.decoder.constraint_count()..]);

        // when in debug mode, save transition evaluations before they are combined
        #[cfg(debug_assertions)]
        self.save_transition_evaluations(&evaluations, step);

        // if the constraints should evaluate to all zeros at this step,
        // make sure they do, and return
        if self.should_evaluate_to_zero_at(step) {
            let step = step / self.extension_factor;
            for i in 0..evaluations.len() {
                assert!(evaluations[i] == field::ZERO, "transition constraint at step {} were not satisfied", step);
            }
            return field::ZERO;
        }

        // compute a pseudo-random linear combination of all transition constraints
        return self.combine_transition_constraints(&evaluations, x);
    }

    /// Computes pseudo-random liner combination of transition constraints at point x. This function
    /// is similar to the one above but it can also be used to evaluate constraints at any point
    /// in the filed (not just in the evaluation domain). However, it is also much slower.
    pub fn evaluate_transition_at(&self, current: &TraceState, next: &TraceState, x: u128) -> u128 {
        // evaluate transition constraints
        let mut evaluations = vec![field::ZERO; self.t_constraint_num];
        self.decoder.evaluate_at(&current, &next, x, &mut evaluations);
        self.stack.evaluate_at(&current, &next, x, &mut evaluations[self.decoder.constraint_count()..]);

        // compute a pseudo-random linear combination of all transition constraints
        return self.combine_transition_constraints(&evaluations, x);
    }

    /// Computes pseudo-random linear combination of boundary constraints B_i at point x  separately
    /// for the first and for the last steps of the program; the constraints are computed as:
    /// cc_{i * 2} * B_i + cc_{i * 2 + 1} * B_i * x^p for all i, where cc_j are the coefficients
    /// used in the linear combination and x^p is a degree adjustment factor.
    pub fn evaluate_boundaries(&self, current: &TraceState, x: u128) -> (u128, u128) {
        
        // compute degree adjustment factor
        let xp = field::exp(x, self.b_degree_adj);

        // 1 ----- compute combination of boundary constraints for the first step ------------------
        let mut i_result = field::ZERO;
        let mut result_adj = field::ZERO;

        let cc = self.coefficients.i_boundary;
        let mut cc_idx = 0;

        // make sure operation sponge registers are set to zeros 
        let op_acc = current.sponge();
        for i in 0..op_acc.len() {
            i_result = field::add(i_result, field::mul(op_acc[i], cc[cc_idx]));
            result_adj = field::add(result_adj, field::mul(op_acc[i], cc[cc_idx + 1]));
            cc_idx += 2;
        }

        // make sure stack registers are set to inputs
        let user_stack = current.user_stack();
        for i in 0..self.inputs.len() {
            let val = field::sub(user_stack[i], self.inputs[i]);
            i_result = field::add(i_result, field::mul(val, cc[cc_idx]));
            result_adj = field::add(result_adj, field::mul(val, cc[cc_idx + 1]));
            cc_idx += 2;
        }

        // raise the degree of adjusted terms and sum all the terms together
        i_result = field::add(i_result, field::mul(result_adj, xp));

        // 2 ----- compute combination of boundary constraints for the last step -------------------
        let mut f_result = field::ZERO;
        let mut result_adj = field::ZERO;

        let cc = self.coefficients.f_boundary;
        let mut cc_idx = 0;

        // make sure control flow op_bits are set VOID (111)
        let op_bits = current.cf_op_bits();
        for i in 0..op_bits.len() {
            let val = field::sub(op_bits[i], field::ONE);
            f_result = field::add(f_result, field::mul(val, cc[cc_idx]));
            result_adj = field::add(result_adj, field::mul(val, cc[cc_idx + 1]));
            cc_idx += 2;
        }

        // make sure low-degree op_bits are set to NOOP (11111)
        let op_bits = current.ld_op_bits();
        for i in 0..op_bits.len() {
            let val = field::sub(op_bits[i], field::ONE);
            f_result = field::add(f_result, field::mul(val, cc[cc_idx]));
            result_adj = field::add(result_adj, field::mul(val, cc[cc_idx + 1]));
            cc_idx += 2;
        }

        // make sure high-degree op_bits are set to NOOP (11)
        let op_bits = current.ld_op_bits();
        for i in 0..op_bits.len() {
            let val = field::sub(op_bits[i], field::ONE);
            f_result = field::add(f_result, field::mul(val, cc[cc_idx]));
            result_adj = field::add(result_adj, field::mul(val, cc[cc_idx + 1]));
            cc_idx += 2;
        }

        // make sure operation sponge contains program hash
        let program_hash = current.program_hash();
        for i in 0..self.program_hash.len() {
            let val = field::sub(program_hash[i], self.program_hash[i]);
            f_result = field::add(f_result, field::mul(val, cc[cc_idx]));
            result_adj = field::add(result_adj, field::mul(val, cc[cc_idx + 1]));
            cc_idx += 2;
        }

        // make sure stack registers are set to outputs
        for i in 0..self.outputs.len() {
            let val = field::sub(user_stack[i], self.outputs[i]);
            f_result = field::add(f_result, field::mul(val, cc[cc_idx]));
            result_adj = field::add(result_adj, field::mul(val, cc[cc_idx + 1]));
            cc_idx += 2;
        }

        // make sure op_count register is set to the claimed value of operations
        let val = field::sub(current.op_counter(), self.op_count);
        f_result = field::add(f_result, field::mul(val, cc[cc_idx]));
        result_adj = field::add(result_adj, field::mul(val, cc[cc_idx + 1]));

        // raise the degree of adjusted terms and sum all the terms together
        f_result = field::add(f_result, field::mul(result_adj, xp));

        return (i_result, f_result);
    }

    // HELPER METHODS
    // -------------------------------------------------------------------------------------------
    fn should_evaluate_to_zero_at(&self, step: usize) -> bool {
        return (step & (self.extension_factor - 1) == 0) // same as: step % extension_factor == 0
            && (step != self.domain_size - self.extension_factor);
    }

    fn combine_transition_constraints(&self, evaluations: &Vec<u128>, x: u128) -> u128 {
        let cc = self.coefficients.transition;
        let mut result = field::ZERO;

        let mut i = 0;
        for (incremental_degree, constraints) in self.t_degree_groups.iter() {

            // for each group of constraints with the same degree, separately compute
            // combinations of D(x) and D(x) * x^p
            let mut result_adj = field::ZERO;
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

        return result;
    }

    #[cfg(debug_assertions)]
    fn save_transition_evaluations(&self, evaluations: &[u128], step: usize) {
        unsafe {
            let mutable_self = &mut *(self as *const _ as *mut Evaluator);
            for i in 0..evaluations.len() {
                mutable_self.t_evaluations[i][step] = evaluations[i];
            }
        }
    }

    #[cfg(debug_assertions)]
    pub fn get_transition_evaluations(&self) -> &Vec<Vec<u128>> {
        return &self.t_evaluations;
    }

    #[cfg(debug_assertions)]
    pub fn get_transition_degrees(&self) -> Vec<usize> {
        return [
            self.decoder.constraint_degrees(), self.stack.constraint_degrees()
        ].concat();
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn group_transition_constraints(degrees: Vec<usize>, trace_length: usize) -> Vec<(u128, Vec<usize>)> {
    let mut groups = [
        Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
    ];

    for (i, &degree) in degrees.iter().enumerate() {
        groups[degree].push(i);
    }

    let target_degree = get_transition_constraint_target_degree(trace_length);

    let mut result = Vec::new();
    for (degree, constraints) in groups.iter().enumerate() {
        if constraints.len() == 0 { continue; }
        let constraint_degree = (trace_length - 1) * degree;    
        let incremental_degree = (target_degree - constraint_degree) as u128;
        result.push((incremental_degree, constraints.clone()));
    }

    return result;
}

fn get_boundary_constraint_adjustment_degree(trace_length: usize) -> u128 {
    let target_degree = get_boundary_constraint_target_degree(trace_length);
    let boundary_constraint_degree = trace_length - 1;
    return (target_degree - boundary_constraint_degree) as u128;
}

/// target degree for boundary constraints is set so that when divided by boundary
/// constraint divisor (degree 1 polynomial), the degree will be equal to
/// deg(combination domain) - deg(trace)
fn get_boundary_constraint_target_degree(trace_length: usize) -> usize {
    let combination_degree = (MAX_CONSTRAINT_DEGREE - 1) * trace_length;
    let divisor_degree = 1;
    return combination_degree + divisor_degree;
}

/// target degree for transition constraints is set so when divided transition 
/// constraint divisor (deg(trace) - 1 polynomial), the degree will be equal to
/// deg(combination domain) - deg(trace)
fn get_transition_constraint_target_degree(trace_length: usize) -> usize {
    let combination_degree = (MAX_CONSTRAINT_DEGREE - 1) * trace_length;
    let divisor_degree = trace_length - 1;
    return combination_degree + divisor_degree;
}

fn parse_program_hash(program_hash: &[u8; 32]) -> Vec<u128> {
    return vec![
        field::from_bytes(&program_hash[..16]),
        field::from_bytes(&program_hash[16..]),
    ];
}

fn get_boundary_constraint_num(inputs: &[u128], outputs: &[u128]) -> usize {
    return
        PROGRAM_DIGEST_SIZE 
        + inputs.len() + outputs.len()
        + 1 /* for op_count */;
}