use crate::math::{ field, parallel, fft, polynom };
use crate::stark::{ TraceState };
use crate::utils::{ uninit_vector };
use super::{ ConstraintEvaluator, ConstraintPoly, MAX_CONSTRAINT_DEGREE };

// TYPES AND INTERFACES
// ================================================================================================
pub struct ConstraintTable {
    evaluator       : ConstraintEvaluator,
    domain          : Vec<u64>,
    domain_stride   : usize,
    i_evaluations   : Vec<u64>, // combined evaluations of boundary constraints at the first step
    f_evaluations   : Vec<u64>, // combined evaluations of boundary constraints at the last step
    t_evaluations   : Vec<u64>, // combined evaluations of transition constraints
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
            i_evaluations   : uninit_vector(composition_domain_size),
            f_evaluations   : uninit_vector(composition_domain_size),
            t_evaluations   : uninit_vector(composition_domain_size),
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
        self.i_evaluations[step] = init_bound;
        self.f_evaluations[step] = last_bound;
        self.t_evaluations[step] = self.evaluator.evaluate_transition(current, next, x, step);
    }

    // CONSTRAINT COMPOSITION
    // -------------------------------------------------------------------------------------------
    pub fn into_combination_poly(mut self) -> ConstraintPoly {

        let composition_root = field::get_root_of_unity(self.composition_domain_size() as u64);
        let inv_twiddles = fft::get_inv_twiddles(composition_root, self.composition_domain_size());
        
        let mut combined_poly = uninit_vector(self.composition_domain_size());
        
        // 1 ----- boundary constraints for the initial step --------------------------------------
        // interpolate initial step boundary constraint combination into a polynomial, divide the 
        // polynomial by Z(x) = (x - 1), and add it to the result
        polynom::interpolate_fft_twiddles(&mut self.i_evaluations, &inv_twiddles, true);
        polynom::syn_div_in_place(&mut self.i_evaluations, field::neg(field::ONE));
        combined_poly.copy_from_slice(&self.i_evaluations);

        // 2 ----- boundary constraints for the final step ----------------------------------------
        // interpolate final step boundary constraint combination into a polynomial, divide the 
        // polynomial by Z(x) = (x - x_at_last_step), and add it to the result
        polynom::interpolate_fft_twiddles(&mut self.f_evaluations, &inv_twiddles, true);
        let x_at_last_step = self.get_x_at_last_step();
        polynom::syn_div_in_place(&mut self.f_evaluations, field::neg(x_at_last_step));
        parallel::add_in_place(&mut combined_poly, &self.f_evaluations, 1);

        // 3 ----- transition constraints ---------------------------------------------------------
        // interpolate transition constraint combination into a polynomial, divide the polynomial
        // by Z(x) = (x^steps - 1) / (x - x_at_last_step), and add it to the result
        let trace_length = self.trace_length();
        polynom::interpolate_fft_twiddles(&mut self.t_evaluations, &inv_twiddles, true);
        polynom::syn_div_expanded_in_place(&mut self.t_evaluations, trace_length, &[x_at_last_step]);
        parallel::add_in_place(&mut combined_poly, &self.t_evaluations, 1);

        return ConstraintPoly::new(combined_poly, self.domain);
    }

    // HELPER METHODS
    // -------------------------------------------------------------------------------------------
    fn get_x_at_last_step(&self) -> u64 {
        return self.domain[self.domain.len() - self.evaluator.extension_factor()];
    }
}