use crate::math::{ field, parallel, fft, polys };
use crate::stark::{ TraceState };
use crate::utils::{ uninit_vector, zero_filled_vector };
use super::{ ConstraintEvaluator };

// TYPES AND INTERFACES
// ================================================================================================
pub struct ConstraintTable {
    evaluator       : ConstraintEvaluator,
    trace_reg_comb  : Vec<u64>,
    init_bound_comb : Vec<u64>,
    final_bound_comb: Vec<u64>,
    transition_comb : Vec<u64>,
}

// CONSTRAINT TABLE IMPLEMENTATION
// ================================================================================================
impl ConstraintTable {

    pub fn new(evaluator: ConstraintEvaluator, extension_factor: usize) -> ConstraintTable {
        let composition_domain_size = evaluator.domain_size();
        let evaluation_domain_size = evaluator.trace_length() * extension_factor;
        return ConstraintTable {
            evaluator       : evaluator,
            trace_reg_comb  : zero_filled_vector(composition_domain_size, evaluation_domain_size),
            init_bound_comb : zero_filled_vector(composition_domain_size, evaluation_domain_size),
            final_bound_comb: zero_filled_vector(composition_domain_size, evaluation_domain_size),
            transition_comb : zero_filled_vector(composition_domain_size, evaluation_domain_size),
        };
    }

    pub fn composition_degree(&self) -> usize {
        return self.evaluator.composition_degree();
    }

    pub fn constraint_count(&self) -> usize {
        return self.evaluator.constraint_count();
    }

    pub fn domain_size(&self) -> usize {
        return self.evaluator.domain_size();
    }

    // CONSTRAINT EVALUATION
    // --------------------------------------------------------------------------------------------
    pub fn evaluate(&mut self, current: &TraceState, next: &TraceState, x: u64, step: usize) {
        self.trace_reg_comb[step] = self.evaluator.combine_trace_registers(current, x);
        let (init_bound, last_bound) = self.evaluator.evaluate_boundaries(current, x);
        self.init_bound_comb[step] = init_bound;
        self.final_bound_comb[step] = last_bound;
        self.transition_comb[step] = self.evaluator.evaluate_transition(current, next, x, step);
    }

    // CONSTRAINT COMPOSITION
    // -------------------------------------------------------------------------------------------
    pub fn compose(&mut self, domain: &[u64]) -> Vec<u64> {

        let composition_root = field::get_root_of_unity(self.domain_size() as u64);
        let inv_twiddles = fft::get_inv_twiddles(composition_root, self.domain_size());

        let domain_root = field::get_root_of_unity(domain.len() as u64);
        let twiddles = fft::get_twiddles(domain_root, domain.len());

        // 1 ----- trace register combination -----------------------------------------------------
        // extend linear combination of trace registers to the full evaluation domain
        extend_evaluations(&mut self.trace_reg_comb, &inv_twiddles, &twiddles);
        let mut result = uninit_vector(domain.len());
        result.copy_from_slice(&self.trace_reg_comb);

        // 2 ----- boundary constraints for the initial step --------------------------------------
        // extend constraint evaluations to the full evaluation domain, divide them by zero poly
        // Z(x) = (x - 1), and add them to the result
        extend_evaluations(&mut self.init_bound_comb, &inv_twiddles, &twiddles);
        let z_inverses = self.get_init_bound_inv_z(&domain);
        parallel::mul_in_place(&mut self.init_bound_comb, &z_inverses, 1);
        parallel::add_in_place(&mut result, &self.init_bound_comb, 1);

        // 3 ----- boundary constraints for the final step ----------------------------------------
        // extend constraint evaluations to the full evaluation domain, divide them by zero poly
        // Z(x) = (x - x_at_last_step), and add them to the result
        extend_evaluations(&mut self.final_bound_comb, &inv_twiddles, &twiddles);
        let last_step_z = self.get_final_bound_z(&domain);
        let z_inverses = parallel::inv(&last_step_z, 1);
        parallel::mul_in_place(&mut self.final_bound_comb, &z_inverses, 1);
        parallel::add_in_place(&mut result, &self.final_bound_comb, 1);

        // 4 ----- transition constraints ---------------------------------------------------------
        // extend constraint evaluations to the full evaluation domain, divide them by zero poly
        // Z(x) = (x^steps - 1) / (x - x_at_last_step), and add them to the result
        extend_evaluations(&mut self.transition_comb, &inv_twiddles, &twiddles);
        let z_inverses = self.get_transition_inv_z(&domain, &last_step_z);
        parallel::mul_in_place(&mut self.transition_comb, &z_inverses, 1);
        parallel::add_in_place(&mut result, &self.transition_comb, 1);

        return result;
    }

    // ZERO POLYNOMIALS
    // -------------------------------------------------------------------------------------------

    /// Computes inverse evaluations of Z(x) polynomial for init boundary constraints; 
    /// Z(x) = (x - 1), so, inv(Z(x)) = inv(x - 1)
    fn get_init_bound_inv_z(&self, domain: &[u64]) -> Vec<u64> {
    
        // compute (x - 1) for all values in the domain
        let mut result = domain.to_vec();
        parallel::sub_const_in_place(&mut result, field::ONE, 1);

        // invert the result
        result = parallel::inv(&result, 1);

        return result;
    }
    
    /// Computes evaluations of Z(x) polynomial for final boundary constraints;
    /// Z(x) = (x - x_at_last_step)
    fn get_final_bound_z(&self, domain: &[u64]) -> Vec<u64> {
    
        let extension_factor = domain.len() / self.evaluator.trace_length();

        // compute (x - 1) for all values in the domain
        let mut result = domain.to_vec();
        let x_at_last_step = domain[domain.len() - extension_factor];
        parallel::sub_const_in_place(&mut result, x_at_last_step, 1);

        return result;
    }

    /// Computes inverse evaluations of Z(x) polynomial for transition constraints; Z(x) = 
    /// (x^steps - 1) / (x - x_at_last_step), so, inv(Z(x)) = inv(x^steps - 1) * (x - x_at_last_step)
    fn get_transition_inv_z(&self, domain: &[u64], last_step_z: &[u64]) -> Vec<u64> {

        // compute (x^steps - 1); TODO: can be parallelized
        let steps = self.evaluator.trace_length();
        let mut result = uninit_vector(domain.len());
        for i in 0..result.len() {
            let x_to_the_steps = domain[(i * steps) % domain.len()];
            result[i] = field::sub(x_to_the_steps, field::ONE);
        }

        // invert the numerators
        let mut result = parallel::inv(&result, 1);

        // multiply the result by (x - x_at_last_step)
        parallel::mul_in_place(&mut result, last_step_z, 1);

        return result;
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn extend_evaluations(mut evaluations: &mut Vec<u64>, inv_twiddles: &[u64], twiddles: &[u64]) {
    polys::interpolate_fft_twiddles(&mut evaluations, &inv_twiddles, true);
    unsafe { evaluations.set_len(evaluations.capacity()); }
    polys::eval_fft_twiddles(&mut evaluations, &twiddles, true);
}