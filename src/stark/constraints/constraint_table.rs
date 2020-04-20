use crate::math::{ field, parallel, fft, polys };
use crate::stark::{ TraceState };
use crate::utils::{ uninit_vector, zero_filled_vector };
use super::{ ConstraintEvaluator };

// TYPES AND INTERFACES
// ================================================================================================
pub struct ConstraintTable {
    evaluator       : ConstraintEvaluator,
    transition_comb : Vec<u64>,
    boundary_comb   : Vec<u64>,
    trace_comb      : Vec<u64>,
}

// CONSTRAINT TABLE IMPLEMENTATION
// ================================================================================================
impl ConstraintTable {

    pub fn new(evaluator: ConstraintEvaluator) -> ConstraintTable {
        let composition_domain_size = evaluator.domain_size();
        return ConstraintTable {
            evaluator       : evaluator,
            transition_comb : uninit_vector(composition_domain_size),
            boundary_comb   : uninit_vector(composition_domain_size),
            trace_comb      : uninit_vector(composition_domain_size),
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
        self.transition_comb[step] = self.evaluator.evaluate_transition(current, next, x, step);
        self.boundary_comb[step] = self.evaluator.evaluate_boundaries(current, next, x, step);
        self.trace_comb[step] = self.evaluator.combine_trace_registers(current, x);
    }

    // CONSTRAINT COMPOSITION
    // -------------------------------------------------------------------------------------------
    pub fn compose(&self, domain: &[u64]) -> Vec<u64> {

        let composition_root = field::get_root_of_unity(self.domain_size() as u64);
        let mut result = zero_filled_vector(self.domain_size(), domain.len());

        // 1 ----- normalize constraint degrees ---------------------------------------------------
        result.copy_from_slice(&self.transition_comb);

        // 2 ----- extend linear combination of constraints to evaluation domain ------------------
        let inv_twiddles = fft::get_inv_twiddles(composition_root, self.domain_size());
        polys::interpolate_fft_twiddles(&mut result, &inv_twiddles, true);

        let domain_root = field::get_root_of_unity(domain.len() as u64);
        let twiddles = fft::get_twiddles(domain_root, domain.len());
        unsafe { result.set_len(domain.len()); }
        polys::eval_fft_twiddles(&mut result, &twiddles, true);

        // 3 ----- divide linear combination of constraints by zero polynomial --------------------
        let z_inverses = self.get_inv_z_evaluations(&domain);
        parallel::mul_in_place(&mut result, &z_inverses, 1);

        return result;
    }

    /// Computes inverse evaluations of Z(x) polynomial; Z(x) = (x^steps - 1) / (x - x_at_last_step),
    /// so, inv(Z(x)) = inv(x^steps - 1) * (x - x_at_last_step)
    fn get_inv_z_evaluations(&self, domain: &[u64]) -> Vec<u64> {

        let steps = self.evaluator.trace_length();
        let extension_factor = domain.len() / self.evaluator.trace_length();

        // compute (x^steps - 1); TODO: can be parallelized
        let mut result = uninit_vector(domain.len());
        for i in 0..result.len() {
            let x_to_the_steps = domain[(i * steps) % domain.len()];
            result[i] = field::sub(x_to_the_steps, field::ONE);
        }

        // invert the numerators
        let mut result = parallel::inv(&result, 1);

        // multiply the result by (x - x_at_last_step), TODO: can be done in parallel
        let x_at_last_step = domain[domain.len() - extension_factor];
        for i in 0..result.len() {
            result[i] = field::mul(result[i], field::sub(domain[i], x_at_last_step));
        }

        return result;
    }
}