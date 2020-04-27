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
            trace_reg_comb  : uninit_vector(composition_domain_size),
            init_bound_comb : uninit_vector(composition_domain_size),
            final_bound_comb: uninit_vector(composition_domain_size),
            transition_comb : uninit_vector(composition_domain_size),
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

    pub fn composition_domain_size(&self) -> usize {
        return self.evaluator.domain_size();
    }

    pub fn trace_length(&self) -> usize {
        return self.evaluator.trace_length();
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

        let composition_root = field::get_root_of_unity(self.composition_domain_size() as u64);
        let inv_twiddles = fft::get_inv_twiddles(composition_root, self.composition_domain_size());

        // 1 ----- trace register combination -----------------------------------------------------
        // interpolate linear combination of trace registers into a polynomial, and copy the
        // polynomial into the result
        polys::interpolate_fft_twiddles(&mut self.trace_reg_comb, &inv_twiddles, true);
        
        let mut result = zero_filled_vector(self.composition_domain_size(), self.domain.len());
        result[0..self.composition_domain_size()].copy_from_slice(&self.trace_reg_comb);

        // 2 ----- boundary constraints for the initial step --------------------------------------
        // interpolate initial step boundary constraint combination into a polynomial, divide the 
        // polynomial by Z(x) = (x - 1), and add it to the result
        polys::interpolate_fft_twiddles(&mut self.init_bound_comb, &inv_twiddles, true);
        polys::syn_div_in_place(&mut self.init_bound_comb, field::neg(field::ONE));
        parallel::add_in_place(&mut result, &self.init_bound_comb, 1);

        // 3 ----- boundary constraints for the final step ----------------------------------------
        // interpolate final step boundary constraint combination into a polynomial, divide the 
        // polynomial by Z(x) = (x - x_at_last_step), and add it to the result
        polys::interpolate_fft_twiddles(&mut self.final_bound_comb, &inv_twiddles, true);
        let x_at_last_step = self.get_x_at_last_step();
        polys::syn_div_in_place(&mut self.final_bound_comb, field::neg(x_at_last_step));
        parallel::add_in_place(&mut result, &self.final_bound_comb, 1);

        // 4 ----- transition constraints ---------------------------------------------------------
        // interpolate transition constraint combination into a polynomial, divide the polynomial
        // by Z(x) = (x^steps - 1) / (x - x_at_last_step), and add it to the result
        let trace_length = self.trace_length();
        polys::interpolate_fft_twiddles(&mut self.transition_comb, &inv_twiddles, true);
        polys::syn_div_expanded_in_place(&mut self.transition_comb, trace_length, &[x_at_last_step]);
        parallel::add_in_place(&mut result, &self.transition_comb, 1);

        // 5 ----- evaluate combination polynomial on evaluation domain ---------------------------
        let domain_root = field::get_root_of_unity(self.domain.len() as u64);
        let twiddles = fft::get_twiddles(domain_root, self.domain.len());
        unsafe { result.set_len(result.capacity()); }
        polys::eval_fft_twiddles(&mut result, &twiddles, true);

        return result;
    }

    // HELPER METHODS
    // -------------------------------------------------------------------------------------------
    fn get_x_at_last_step(&self) -> u64 {
        let extension_factor = self.domain.len() / self.trace_length();
        return self.domain[self.domain.len() - extension_factor];
    }
}