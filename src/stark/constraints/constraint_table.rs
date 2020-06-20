use crate::math::{ FiniteField, parallel, fft, polynom };
use crate::stark::{ TraceTable, TraceState };
use crate::utils::{ Hasher, Accumulator, uninit_vector };
use super::{ ConstraintEvaluator, ConstraintPoly };

// TYPES AND INTERFACES
// ================================================================================================
pub struct ConstraintTable<T: FiniteField + Accumulator + Hasher> {
    evaluator       : ConstraintEvaluator<T>,
    i_evaluations   : Vec<T>,   // combined evaluations of boundary constraints at the first step
    f_evaluations   : Vec<T>,   // combined evaluations of boundary constraints at the last step
    t_evaluations   : Vec<T>,   // combined evaluations of transition constraints
}

// CONSTRAINT TABLE IMPLEMENTATION
// ================================================================================================
impl <T> ConstraintTable<T>
    where T: FiniteField + Accumulator + Hasher
{
    pub fn new(trace: &TraceTable<T>, trace_root: &[u8; 32], inputs: &[T], outputs: &[T]) -> ConstraintTable<T> {
        let evaluator = ConstraintEvaluator::from_trace(trace, trace_root, inputs, outputs);
        let evaluation_domain_size = evaluator.domain_size();
        return ConstraintTable {
            evaluator       : evaluator,
            i_evaluations   : uninit_vector(evaluation_domain_size),
            f_evaluations   : uninit_vector(evaluation_domain_size),
            t_evaluations   : uninit_vector(evaluation_domain_size),
        };
    }

    /// Returns the total number of transition and boundary constraints.
    pub fn constraint_count(&self) -> usize {
        return self.evaluator.constraint_count();
    }

    /// Returns the size of the evaluation domain = trace_length * MAX_CONSTRAINT_DEGREE
    pub fn evaluation_domain_size(&self) -> usize {
        return self.evaluator.domain_size();
    }

    /// Returns the length of the un-extended execution trace.
    pub fn trace_length(&self) -> usize {
        return self.evaluator.trace_length();
    }

    /// Evaluates transition and boundary constraints at the specified step.
    pub fn evaluate(&mut self, current: &TraceState<T>, next: &TraceState<T>, x: T, step: usize) {
        let (init_bound, last_bound) = self.evaluator.evaluate_boundaries(current, x);
        self.i_evaluations[step] = init_bound;
        self.f_evaluations[step] = last_bound;
        self.t_evaluations[step] = self.evaluator.evaluate_transition(current, next, x, step);
    }

    /// Interpolates all constraint evaluations into polynomials and combines all these 
    /// polynomials into a single polynomial using pseudo-random linear combination.
    pub fn combine_polys(mut self) -> ConstraintPoly<T> {

        let combination_root = T::get_root_of_unity(self.evaluation_domain_size());
        let inv_twiddles = fft::get_inv_twiddles(combination_root, self.evaluation_domain_size());
        
        let mut combined_poly = uninit_vector(self.evaluation_domain_size());
        
        // 1 ----- boundary constraints for the initial step --------------------------------------
        // interpolate initial step boundary constraint combination into a polynomial, divide the 
        // polynomial by Z(x) = (x - 1), and add it to the result
        polynom::interpolate_fft_twiddles(&mut self.i_evaluations, &inv_twiddles, true);
        polynom::syn_div_in_place(&mut self.i_evaluations, T::ONE);
        combined_poly.copy_from_slice(&self.i_evaluations);

        // 2 ----- boundary constraints for the final step ----------------------------------------
        // interpolate final step boundary constraint combination into a polynomial, divide the 
        // polynomial by Z(x) = (x - x_at_last_step), and add it to the result
        polynom::interpolate_fft_twiddles(&mut self.f_evaluations, &inv_twiddles, true);
        let x_at_last_step = self.evaluator.get_x_at_last_step();
        polynom::syn_div_in_place(&mut self.f_evaluations, x_at_last_step);
        parallel::add_in_place(&mut combined_poly, &self.f_evaluations, 1);

        // 3 ----- transition constraints ---------------------------------------------------------
        // interpolate transition constraint combination into a polynomial, divide the polynomial
        // by Z(x) = (x^steps - 1) / (x - x_at_last_step), and add it to the result
        let trace_length = self.trace_length();
        polynom::interpolate_fft_twiddles(&mut self.t_evaluations, &inv_twiddles, true);
        polynom::syn_div_expanded_in_place(&mut self.t_evaluations, trace_length, &[x_at_last_step]);
        parallel::add_in_place(&mut combined_poly, &self.t_evaluations, 1);

        return ConstraintPoly::new(combined_poly);
    }

}