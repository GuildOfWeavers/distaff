use crate::math::{ field, parallel, fft, polys };
use crate::stark::{ TraceState };
use crate::utils::{ uninit_vector, zero_filled_vector };
use super::{ ConstraintEvaluator, MAX_CONSTRAINT_DEGREE };

// TYPES AND INTERFACES
// ================================================================================================
pub struct ConstraintTable {
    evaluator       : ConstraintEvaluator,
    domain          : Vec<u64>,
    domain_stride   : usize,
    trace_reg_comb  : Vec<u64>,
    init_bound_comb : Vec<u64>,
    final_bound_comb: Vec<u64>,
    transition_comb : Vec<u64>,
}

// CONSTRAINT TABLE IMPLEMENTATION
// ================================================================================================
impl ConstraintTable {

    pub fn new(evaluator: ConstraintEvaluator, evaluation_domain: Vec<u64>) -> ConstraintTable {

        let composition_domain_size = evaluator.domain_size();
        let evaluation_domain_size = evaluation_domain.len();
        let extension_factor = evaluation_domain.len() / evaluator.trace_length();
        let domain_stride = extension_factor / MAX_CONSTRAINT_DEGREE;

        assert!(composition_domain_size == evaluation_domain_size / domain_stride,
            "composition and evaluation domains are inconsistent");

        return ConstraintTable {
            evaluator       : evaluator,
            domain          : evaluation_domain,
            trace_reg_comb  : zero_filled_vector(composition_domain_size, evaluation_domain_size),
            init_bound_comb : zero_filled_vector(composition_domain_size, evaluation_domain_size),
            final_bound_comb: zero_filled_vector(composition_domain_size, evaluation_domain_size),
            transition_comb : zero_filled_vector(composition_domain_size, evaluation_domain_size),
            domain_stride   : domain_stride,
        };
    }

    pub fn composition_degree(&self) -> usize {
        return self.evaluator.composition_degree();
    }

    pub fn constraint_count(&self) -> usize {
        return self.evaluator.constraint_count();
    }

    pub fn domain(&self) -> &[u64] {
        return &self.domain;
    }

    pub fn domain_stride(&self) -> usize {
        return self.domain_stride;
    }

    // CONSTRAINT EVALUATION
    // --------------------------------------------------------------------------------------------
    pub fn evaluate(&mut self, current: &TraceState, next: &TraceState, domain_step: usize) {
        let x = self.domain[domain_step];
        let step = domain_step / self.domain_stride;

        self.trace_reg_comb[step] = self.evaluator.combine_trace_registers(current, x);
        let (init_bound, last_bound) = self.evaluator.evaluate_boundaries(current, x);
        self.init_bound_comb[step] = init_bound;
        self.final_bound_comb[step] = last_bound;
        self.transition_comb[step] = self.evaluator.evaluate_transition(current, next, x, step);
    }

    // CONSTRAINT COMPOSITION
    // -------------------------------------------------------------------------------------------
    pub fn compose(&mut self) -> Vec<u64> {

        let composition_root = field::get_root_of_unity(self.domain.len() as u64);
        let inv_twiddles = fft::get_inv_twiddles(composition_root, self.domain.len());

        let domain_root = field::get_root_of_unity(self.domain.len() as u64);
        let twiddles = fft::get_twiddles(domain_root, self.domain.len());

        // 1 ----- trace register combination -----------------------------------------------------
        // extend linear combination of trace registers to the full evaluation domain
        extend_evaluations(&mut self.trace_reg_comb, &inv_twiddles, &twiddles);
        let mut result = uninit_vector(self.domain.len());
        result.copy_from_slice(&self.trace_reg_comb);

        // 2 ----- boundary constraints for the initial step --------------------------------------
        // divide constraint evaluations by Z(x) = (x - 1), extend them to evaluation domain, and
        // add them to the result
        polys::interpolate_fft_twiddles(&mut self.init_bound_comb, &inv_twiddles, true);
        polys::syn_div_in_place(&mut self.init_bound_comb, field::neg(field::ONE));
        unsafe { self.init_bound_comb.set_len(self.init_bound_comb.capacity()); }
        polys::eval_fft_twiddles(&mut self.init_bound_comb, &twiddles, true);
        parallel::add_in_place(&mut result, &self.init_bound_comb, 1);

        // 3 ----- boundary constraints for the final step ----------------------------------------
        // divide constraint evaluations by Z(x) = (x - x_at_last_step), extend them to evaluation
        // domain, and add them to the result
        polys::interpolate_fft_twiddles(&mut self.final_bound_comb, &inv_twiddles, true);
        let x_at_last_step = self.get_x_at_last_step();
        polys::syn_div_in_place(&mut self.final_bound_comb, field::neg(x_at_last_step));
        unsafe { self.final_bound_comb.set_len(self.final_bound_comb.capacity()); }
        polys::eval_fft_twiddles(&mut self.final_bound_comb, &twiddles, true);
        parallel::add_in_place(&mut result, &self.final_bound_comb, 1);

        // 4 ----- transition constraints ---------------------------------------------------------
        // extend constraint evaluations to the full evaluation domain, divide them by zero poly
        // Z(x) = (x^steps - 1) / (x - x_at_last_step), and add them to the result
        extend_evaluations(&mut self.transition_comb, &inv_twiddles, &twiddles);
        let z_inverses = self.get_transition_inv_z();
        parallel::mul_in_place(&mut self.transition_comb, &z_inverses, 1);
        parallel::add_in_place(&mut result, &self.transition_comb, 1);

        return result;
    }

    // ZERO POLYNOMIALS
    // -------------------------------------------------------------------------------------------
    
    /// Computes inverse evaluations of Z(x) polynomial for transition constraints; Z(x) = 
    /// (x^steps - 1) / (x - x_at_last_step), so, inv(Z(x)) = inv(x^steps - 1) * (x - x_at_last_step)
    fn get_transition_inv_z(&self) -> Vec<u64> {

        // compute (x - x_at_last_step)
        let mut last_step_z = self.domain.clone();
        parallel::sub_const_in_place(&mut last_step_z, self.get_x_at_last_step(), 1);

        // compute (x^steps - 1); TODO: can be parallelized
        let steps = self.evaluator.trace_length();
        let mut result = uninit_vector(self.domain.len());
        for i in 0..result.len() {
            let x_to_the_steps = self.domain[(i * steps) % self.domain.len()];
            result[i] = field::sub(x_to_the_steps, field::ONE);
        }

        // invert the numerators
        let mut result = parallel::inv(&result, 1);

        // multiply the result by (x - x_at_last_step)
        parallel::mul_in_place(&mut result, &last_step_z, 1);

        return result;
    }

    // HELPER METHODS
    // -------------------------------------------------------------------------------------------
    fn get_x_at_last_step(&self) -> u64 {
        let extension_factor = self.domain.len() / self.evaluator.trace_length();
        return self.domain[self.domain.len() - extension_factor];
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn extend_evaluations(mut evaluations: &mut Vec<u64>, inv_twiddles: &[u64], twiddles: &[u64]) {
    polys::interpolate_fft_twiddles(&mut evaluations, &inv_twiddles, true);
    unsafe { evaluations.set_len(evaluations.capacity()); }
    polys::eval_fft_twiddles(&mut evaluations, &twiddles, true);
}