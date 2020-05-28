use std::mem;
use crate::math::{ FiniteField };
use crate::processor::{ opcodes };
use crate::stark::{ StarkProof, TraceTable, TraceState, ConstraintCoefficients, Accumulator };
use crate::utils::{ uninit_vector };
use super::{ decoder::Decoder, stack::Stack, MAX_CONSTRAINT_DEGREE };

// TYPES AND INTERFACES
// ================================================================================================
pub struct Evaluator<T>
    where T: FiniteField + Accumulator
{
    decoder         : Decoder<T>,
    stack           : Stack<T>,

    coefficients    : ConstraintCoefficients<T>,
    domain_size     : usize,
    extension_factor: usize,

    t_constraint_num: usize,
    t_degree_groups : Vec<(T, Vec<usize>)>,
    t_evaluations   : Vec<Vec<T>>,

    b_constraint_num: usize,
    program_hash    : Vec<T>,
    inputs          : Vec<T>,
    outputs         : Vec<T>,
    b_degree_adj    : T,
}

// EVALUATOR IMPLEMENTATION
// ================================================================================================
impl <T> Evaluator<T>
    where T: FiniteField + Accumulator
{
    pub fn from_trace(trace: &TraceTable<T>, trace_root: &[u8; 32], inputs: &[T], outputs: &[T]) -> Evaluator<T> {

        let stack_depth = trace.max_stack_depth();
        let program_hash = trace.get_program_hash();
        let trace_length = trace.unextended_length();
        let extension_factor = MAX_CONSTRAINT_DEGREE;

        // instantiate decoder and stack constraint evaluators 
        let decoder = Decoder::new(trace_length, extension_factor);
        let stack = Stack::new(stack_depth);

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
            b_constraint_num: inputs.len() + outputs.len() + program_hash.len(),
            program_hash    : program_hash,
            inputs          : inputs.to_vec(),
            outputs         : outputs.to_vec(),
            b_degree_adj    : get_boundary_constraint_adjustment_degree(trace_length),
        };
    }

    pub fn from_proof(proof: &StarkProof<T>, program_hash: &[u8; 32], inputs: &[T], outputs: &[T]) -> Evaluator<T> {
        
        let stack_depth = proof.stack_depth();
        let trace_length = proof.trace_length();
        let extension_factor = proof.options().extension_factor();
        
        // instantiate decoder and stack constraint evaluators 
        let decoder = Decoder::new(trace_length, extension_factor);
        let stack = Stack::new(stack_depth);

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
            b_constraint_num: inputs.len() + outputs.len() + program_hash.len(),
            program_hash    : parse_program_hash(program_hash),
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

    pub fn extension_factor(&self) -> usize {
        return self.extension_factor;
    }

    pub fn transition_evaluations(&self) -> &Vec<Vec<T>> {
        return &self.t_evaluations;
    }

    pub fn get_x_at_last_step(&self) -> T {
        let trace_root = T::get_root_of_unity(self.trace_length());
        return T::exp(trace_root, T::from_usize(self.trace_length() - 1));
    }

    // CONSTRAINT EVALUATORS
    // -------------------------------------------------------------------------------------------

    /// Computes pseudo-random linear combination of transition constraints D_i at point x as:
    /// cc_{i * 2} * D_i + cc_{i * 2 + 1} * D_i * x^p for all i, where cc_j are the coefficients
    /// used in the linear combination and x^p is a degree adjustment factor (different for each degree).
    pub fn evaluate_transition(&self, current: &TraceState<T>, next: &TraceState<T>, x: T, step: usize) -> T {
        
        // evaluate transition constraints
        let mut evaluations = vec![T::ZERO; self.t_constraint_num];
        self.decoder.evaluate(&current, &next, step, &mut evaluations);
        self.stack.evaluate(&current, &next, &mut evaluations[self.decoder.constraint_count()..]);

        // when in debug mode, save transition evaluations before they are combined
        #[cfg(debug_assertions)]
        self.save_transition_evaluations(&evaluations, step);

        // if the constraints should evaluate to all zeros at this step,
        // make sure they do, and return
        if self.should_evaluate_to_zero_at(step) {
            let step = step / self.extension_factor;
            for i in 0..evaluations.len() {
                assert!(evaluations[i] == T::ZERO, "transition constraint at step {} didn't evaluate to 0", step);
            }
            return T::ZERO;
        }

        // compute a pseudo-random linear combination of all transition constraints
        return self.combine_transition_constraints(&evaluations, x);
    }

    /// Computes pseudo-random liner combination of transition constraints at point x. This function
    /// is similar to the one above but it can also be used to evaluate constraints at any point
    /// in the filed (not just in the evaluation domain). However, it is also much slower.
    pub fn evaluate_transition_at(&self, current: &TraceState<T>, next: &TraceState<T>, x: T) -> T {
        // evaluate transition constraints
        let mut evaluations = vec![T::ZERO; self.t_constraint_num];
        self.decoder.evaluate_at(&current, &next, x, &mut evaluations);
        self.stack.evaluate(&current, &next, &mut evaluations[self.decoder.constraint_count()..]);

        // compute a pseudo-random linear combination of all transition constraints
        return self.combine_transition_constraints(&evaluations, x);
    }

    /// Computes pseudo-random linear combination of boundary constraints B_i at point x  separately
    /// for the first and for the last steps of the program; the constraints are computed as:
    /// cc_{i * 2} * B_i + cc_{i * 2 + 1} * B_i * x^p for all i, where cc_j are the coefficients
    /// used in the linear combination and x^p is a degree adjustment factor.
    pub fn evaluate_boundaries(&self, current: &TraceState<T>, x: T) -> (T, T) {
        
        // compute degree adjustment factor
        let xp = T::exp(x, self.b_degree_adj);

        // 1 ----- compute combination of boundary constraints for the first step ------------------
        let mut i_result = T::ZERO;
        let mut result_adj = T::ZERO;

        let cc = self.coefficients.i_boundary;
        let mut cc_idx = 0;

        // make sure op_code and ob_bits are set to BEGIN
        let op_code = current.get_op_code();
        let val = T::sub(op_code, T::from(opcodes::BEGIN));
        i_result = T::add(i_result, T::mul(val, cc[cc_idx]));
        result_adj = T::add(result_adj, T::mul(val, cc[cc_idx]));

        let op_bits = current.get_op_bits();
        for i in 0..op_bits.len() {
            cc_idx += 2;
            let val = T::sub(op_bits[i], T::ONE);
            i_result = T::add(i_result, T::mul(val, cc[cc_idx]));
            result_adj = T::add(result_adj, T::mul(val, cc[cc_idx + 1]));
        }

        // make sure operation accumulator registers are set to zeros 
        let op_acc = current.get_op_acc();
        for i in 0..op_acc.len() {
            cc_idx += 2;
            i_result = T::add(i_result, T::mul(op_acc[i], cc[cc_idx]));
            result_adj = T::add(result_adj, T::mul(op_acc[i], cc[cc_idx + 1]));
        }

        // make sure stack registers are set to inputs
        let stack = current.get_stack();
        for i in 0..self.inputs.len() {
            cc_idx += 2;
            let val = T::sub(stack[i], self.inputs[i]);
            i_result = T::add(i_result, T::mul(val, cc[cc_idx]));
            result_adj = T::add(result_adj, T::mul(val, cc[cc_idx + 1]));
        }

        // raise the degree of adjusted terms and sum all the terms together
        i_result = T::add(i_result, T::mul(result_adj, xp));

        // 2 ----- compute combination of boundary constraints for the last step -------------------
        let mut f_result = T::ZERO;
        let mut result_adj = T::ZERO;

        let cc = self.coefficients.f_boundary;
        let mut cc_idx = 0;

        // make sure op_code and op_bits are set to NOOP
        let op_code = current.get_op_code();
        f_result = T::add(f_result, T::mul(op_code, cc[cc_idx]));
        result_adj = T::add(result_adj, T::mul(op_code, cc[cc_idx + 1]));

        let op_bits = current.get_op_bits();
        for i in 0..op_bits.len() {
            cc_idx += 2;
            f_result = T::add(f_result, T::mul(op_bits[i], cc[cc_idx]));
            result_adj = T::add(result_adj, T::mul(op_bits[i], cc[cc_idx + 1]));
        }

        // make sure operation accumulator contains program hash
        let program_hash = current.get_program_hash();
        for i in 0..self.program_hash.len() {
            cc_idx += 2;
            let val = T::sub(program_hash[i], self.program_hash[i]);
            f_result = T::add(f_result, T::mul(val, cc[cc_idx]));
            result_adj = T::add(result_adj, T::mul(val, cc[cc_idx + 1]));
        }

        // make sure stack registers are set to outputs
        for i in 0..self.outputs.len() {
            cc_idx += 2;
            let val = T::sub(stack[i], self.outputs[i]);
            f_result = T::add(f_result, T::mul(val, cc[cc_idx]));
            result_adj = T::add(result_adj, T::mul(val, cc[cc_idx + 1]));
        }

        // raise the degree of adjusted terms and sum all the terms together
        f_result = T::add(f_result, T::mul(result_adj, xp));

        return (i_result, f_result);
    }

    // HELPER METHODS
    // -------------------------------------------------------------------------------------------
    fn should_evaluate_to_zero_at(&self, step: usize) -> bool {
        return (step & (self.extension_factor - 1) == 0) // same as: step % extension_factor == 0
            && (step != self.domain_size - self.extension_factor);
    }

    fn combine_transition_constraints(&self, evaluations: &Vec<T>, x: T) -> T {
        let cc = self.coefficients.transition;
        let mut result = T::ZERO;

        let mut i = 0;
        for (incremental_degree, constraints) in self.t_degree_groups.iter() {

            // for each group of constraints with the same degree, separately compute
            // combinations of D(x) and D(x) * x^p
            let mut result_adj = T::ZERO;
            for &constraint_idx in constraints.iter() {
                let evaluation = evaluations[constraint_idx];
                result = T::add(result, T::mul(evaluation, cc[i * 2]));
                result_adj = T::add(result_adj, T::mul(evaluation, cc[i * 2 + 1]));
                i += 1;
            }

            // increase the degree of D(x) * x^p
            let xp = T::exp(x, *incremental_degree);
            result = T::add(result, T::mul(result_adj, xp));
        }

        return result;
    }

    #[cfg(debug_assertions)]
    fn save_transition_evaluations(&self, evaluations: &[T], step: usize) {
        unsafe {
            let mutable_self = &mut *(self as *const _ as *mut Evaluator<T>);
            for i in 0..evaluations.len() {
                mutable_self.t_evaluations[i][step] = evaluations[i];
            }
        }
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn group_transition_constraints<T>(degrees: Vec<usize>, trace_length: usize) -> Vec<(T, Vec<usize>)>
    where T: FiniteField
{
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
        let incremental_degree = T::from_usize(target_degree - constraint_degree);
        result.push((incremental_degree, constraints.clone()));
    }

    return result;
}

fn get_boundary_constraint_adjustment_degree<T>(trace_length: usize) -> T
    where T: FiniteField
{
    let target_degree = get_boundary_constraint_target_degree(trace_length);
    let boundary_constraint_degree = trace_length - 1;
    return T::from_usize(target_degree - boundary_constraint_degree);
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

fn parse_program_hash<T>(program_hash: &[u8; 32]) -> Vec<T>
    where T: FiniteField
{
    let element_size = mem::size_of::<T>();
    let num_elements = program_hash.len() / element_size;
    let mut result = Vec::with_capacity(num_elements);
    for i in (0..program_hash.len()).step_by(element_size) {
        result.push(T::from_bytes(&program_hash[i..(i + element_size)]))
    }
    return result;
}