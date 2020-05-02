use crate::math::{ field, parallel, fft, polynom };
use crate::stark::{ TraceState };
use crate::utils::{ uninit_vector, zero_filled_vector };
use super::{ ConstraintEvaluator, ConstraintPolys, MAX_CONSTRAINT_DEGREE };

// TYPES AND INTERFACES
// ================================================================================================
pub struct ConstraintTable {
    evaluator       : ConstraintEvaluator,
    domain          : Vec<u64>,
    domain_stride   : usize,
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
        
        let mut result = zero_filled_vector(self.composition_domain_size(), self.domain.len());
        
        // 2 ----- boundary constraints for the initial step --------------------------------------
        // interpolate initial step boundary constraint combination into a polynomial, divide the 
        // polynomial by Z(x) = (x - 1), and add it to the result
        polynom::interpolate_fft_twiddles(&mut self.init_bound_comb, &inv_twiddles, true);
        polynom::syn_div_in_place(&mut self.init_bound_comb, field::neg(field::ONE));
        result[0..self.composition_domain_size()].copy_from_slice(&self.init_bound_comb);

        // 3 ----- boundary constraints for the final step ----------------------------------------
        // interpolate final step boundary constraint combination into a polynomial, divide the 
        // polynomial by Z(x) = (x - x_at_last_step), and add it to the result
        polynom::interpolate_fft_twiddles(&mut self.final_bound_comb, &inv_twiddles, true);
        let x_at_last_step = self.get_x_at_last_step();
        polynom::syn_div_in_place(&mut self.final_bound_comb, field::neg(x_at_last_step));
        parallel::add_in_place(&mut result, &self.final_bound_comb, 1);

        // 4 ----- transition constraints ---------------------------------------------------------
        // interpolate transition constraint combination into a polynomial, divide the polynomial
        // by Z(x) = (x^steps - 1) / (x - x_at_last_step), and add it to the result
        let trace_length = self.trace_length();
        polynom::interpolate_fft_twiddles(&mut self.transition_comb, &inv_twiddles, true);
        polynom::syn_div_expanded_in_place(&mut self.transition_comb, trace_length, &[x_at_last_step]);
        parallel::add_in_place(&mut result, &self.transition_comb, 1);

        // 5 ----- evaluate combination polynomial on evaluation domain ---------------------------
        let domain_root = field::get_root_of_unity(self.domain.len() as u64);
        let twiddles = fft::get_twiddles(domain_root, self.domain.len());
        unsafe { result.set_len(result.capacity()); }
        polynom::eval_fft_twiddles(&mut result, &twiddles, true);

        return result;
    }

    pub fn into_polys(mut self) -> ConstraintPolys {
        
        let mut composition_poly = uninit_vector(self.composition_domain_size());

        let composition_root = field::get_root_of_unity(self.composition_domain_size() as u64);
        let inv_twiddles = fft::get_inv_twiddles(composition_root, self.composition_domain_size());

        // 1 ----- boundary constraints for the initial step --------------------------------------
        // interpolate initial step boundary constraint combination into a polynomial, divide the 
        // polynomial by Z(x) = (x - 1), and copy it into the composition polynomial
        polynom::interpolate_fft_twiddles(&mut self.init_bound_comb, &inv_twiddles, true);
        polynom::syn_div_in_place(&mut self.init_bound_comb, field::neg(field::ONE));
        composition_poly.copy_from_slice(&self.init_bound_comb);

        // 2 ----- boundary constraints for the final step ----------------------------------------
        // interpolate final step boundary constraint combination into a polynomial, divide the 
        // polynomial by Z(x) = (x - x_at_last_step), and add it to the composition polynomial
        polynom::interpolate_fft_twiddles(&mut self.final_bound_comb, &inv_twiddles, true);
        let x_at_last_step = self.get_x_at_last_step();
        polynom::syn_div_in_place(&mut self.final_bound_comb, field::neg(x_at_last_step));
        parallel::add_in_place(&mut composition_poly, &self.final_bound_comb, 1);

        // 3 ----- transition constraints ---------------------------------------------------------
        // interpolate transition constraint combination into a polynomial, divide the polynomial
        // by Z(x) = (x^steps - 1) / (x - x_at_last_step), and add it to the composition polynomial
        let trace_length = self.trace_length();
        polynom::interpolate_fft_twiddles(&mut self.transition_comb, &inv_twiddles, true);
        polynom::syn_div_expanded_in_place(&mut self.transition_comb, trace_length, &[x_at_last_step]);
        parallel::add_in_place(&mut composition_poly, &self.transition_comb, 1);

        // 4 ----- transpose composition polynomial -----------------------------------------------
        // transpose the composition polynomial into 8 polynomials at with degree equal to
        // trace lengths
        let mut a_polys: [Vec<u64>; MAX_CONSTRAINT_DEGREE] = [
            uninit_vector(trace_length), uninit_vector(trace_length),
            uninit_vector(trace_length), uninit_vector(trace_length),
            uninit_vector(trace_length), uninit_vector(trace_length),
            uninit_vector(trace_length), uninit_vector(trace_length),
        ];
        
        for i in (0..composition_poly.len()).step_by(MAX_CONSTRAINT_DEGREE) {
            a_polys[0][i / MAX_CONSTRAINT_DEGREE] = composition_poly[i + 0];
            a_polys[1][i / MAX_CONSTRAINT_DEGREE] = composition_poly[i + 1];
            a_polys[2][i / MAX_CONSTRAINT_DEGREE] = composition_poly[i + 2];
            a_polys[3][i / MAX_CONSTRAINT_DEGREE] = composition_poly[i + 3];
            a_polys[4][i / MAX_CONSTRAINT_DEGREE] = composition_poly[i + 4];
            a_polys[5][i / MAX_CONSTRAINT_DEGREE] = composition_poly[i + 5];
            a_polys[6][i / MAX_CONSTRAINT_DEGREE] = composition_poly[i + 6];
            a_polys[7][i / MAX_CONSTRAINT_DEGREE] = composition_poly[i + 7];
        }

        return ConstraintPolys::new(a_polys, self.domain);
    }

    // HELPER METHODS
    // -------------------------------------------------------------------------------------------
    fn get_x_at_last_step(&self) -> u64 {
        return self.domain[self.domain.len() - self.evaluator.extension_factor()];
    }
}