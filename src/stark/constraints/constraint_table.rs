use std::cmp;
use crate::math::{ field, parallel, fft, polys, field::ONE };
use crate::stark::{ TraceState, TraceTable };
use crate::utils::{ uninit_vector, zero_filled_vector };
use super::{ decoder, stack, hash_acc };

// CONSTANTS
// ================================================================================================
pub const MAX_CONSTRAINT_DEGREE: usize = 8;

// TYPES AND INTERFACES
// ================================================================================================
pub struct ConstraintTable {
    pub decoder : Vec<Vec<u64>>,
    pub op_acc  : Vec<Vec<u64>>,
    pub stack   : Vec<Vec<u64>>,

    io_constraints  : [Vec<Vec<u64>>; 3],
    extension_factor: usize,
}

// CONSTRAINT TABLE IMPLEMENTATION
// ================================================================================================
impl ConstraintTable {

    pub fn new(trace: &TraceTable) -> ConstraintTable {
        assert!(trace.is_extended(), "execution trace hasn't been extended yet");
        
        let trace_length = trace.len() / trace.extension_factor(); // original trace length
        let domain_size = trace_length * MAX_CONSTRAINT_DEGREE;

        return ConstraintTable {
            decoder : create_vectors(decoder::CONSTRAINT_DEGREES.len(), domain_size),
            op_acc  : create_vectors(hash_acc::CONSTRAINT_DEGREES.len(), domain_size),
            stack   : create_vectors(trace.max_stack_depth(), domain_size),
            io_constraints  : [ vec![], vec![], vec![] ],
            extension_factor: trace.extension_factor(),
        };
    }

    pub fn constraint_count(&self) -> usize {
        return self.decoder.len() + self.op_acc.len() + self.stack.len()
            + self.io_constraints[0].len() 
            + self.io_constraints[1].len()
            + self.io_constraints[2].len();
    }

    pub fn len(&self) -> usize {
        return self.decoder[0].len();
    }

    fn trace_length(&self) -> usize {
        return self.len() / MAX_CONSTRAINT_DEGREE;
    }

    fn domain_size(&self) -> usize {
        return self.trace_length() * self.extension_factor;
    }

    // CONSTRAINT EVALUATION
    // --------------------------------------------------------------------------------------------
    pub fn evaluate_transition(&mut self, current: &TraceState, next: &TraceState, step: usize) {
        let should_be_zero = (step % MAX_CONSTRAINT_DEGREE == 0)
            && (step < self.len() - MAX_CONSTRAINT_DEGREE);

        let op_dec = decoder::evaluate(&current, &next);
        copy_constraints(&op_dec, &mut self.decoder, step, should_be_zero);
        
        let op_acc = hash_acc::evaluate(&current, &next, step);
        copy_constraints(&op_acc, &mut self.op_acc, step, should_be_zero);

        let stack = stack::evaluate(&current, &next, self.stack.len());
        copy_constraints(&stack, &mut self.stack, step, should_be_zero);
    }

    pub fn set_io_constraints(&mut self, inputs: &[u64], outputs: &[u64]) {

        // compute root of unity for the evaluation domain
        let domain_root = field::get_root_of_unity(self.domain_size() as u64);

        // compute last value in the evaluation domain
        let last_position = self.domain_size() - self.extension_factor;
        let x_at_last_step = field::exp(domain_root, last_position as u64);

        // create polynomials for input/output constraints
        let num_io_constraints = cmp::min(inputs.len(), outputs.len());
        for i in 0..num_io_constraints {
            let i_poly = polys::interpolate(&[field::ONE, x_at_last_step], &[inputs[i], outputs[i]]);
            self.io_constraints[0].push(i_poly);
        }
        
        // create polynomials for input constraints only
        for i in num_io_constraints..inputs.len() {
            let i_poly = vec![inputs[i]];
            self.io_constraints[1].push(i_poly);
        }

        // create polynomials for output constraints only
        for i in num_io_constraints..outputs.len() {
            let i_poly = vec![outputs[i]];
            self.io_constraints[2].push(i_poly);
        }
    }

    // CONSTRAINT COMPOSITION
    // -------------------------------------------------------------------------------------------
    pub fn get_composition_poly(&self, seed: &[u64; 4], trace: &TraceTable) -> Vec<u64> {

        let domain_size = self.trace_length() * self.extension_factor;

        let composition_root = field::get_root_of_unity(self.len() as u64);
        let mut result = zero_filled_vector(self.len(), domain_size);

        // 1 ----- normalize constraint degrees ---------------------------------------------------

        // compute pseudo-random coefficients for random linear combination of constraints
        let seed = unsafe { &*(seed as *const _ as *const [u8; 32]) };
        let mut coefficients = field::prng_vector(*seed, 2 * self.constraint_count()).into_iter();

        // group constraints by their degree: 0 through 8
        let constraint_groups = self.group_constraints();

        // adjust the degree of constraints and merge them into a single linear combination
        for (degree, constraints) in constraint_groups.iter().enumerate() {
            if constraints.len() == 0 { continue; }

            // adjust degree basis
            let degree = degree * self.trace_length();

            // merge constraint evaluations into random leaner combination of constraints;
            // this computes: result = result + constraint * coefficient
            for constraint in constraints.into_iter() {
                parallel::mul_acc(&mut result, &constraint, coefficients.next().unwrap(), 1);
            }

            if degree < self.len() {
                // compute x^incremental_degree for all x values in the composition domain
                let incremental_degree = self.len() - degree;
                let x_root = field::exp(composition_root, incremental_degree as u64);
                let x_di = field::get_power_series(x_root, self.len());

                // merge constraint evaluations adjusted by the incremental degree into random
                // linear combination; this computes:
                // result = result + constraint * x^incremental_degree * coefficient
                for constraint in constraints.into_iter() {
                    let constraint = parallel::mul(constraint, &x_di, 1);
                    parallel::mul_acc(&mut result, &constraint, coefficients.next().unwrap(), 1);
                }
            }
        }

        // 2 ----- extend linear combination of constraints to evaluation domain ------------------
        let inv_twiddles = fft::get_inv_twiddles(composition_root, self.len());
        polys::interpolate_fft_twiddles(&mut result, &inv_twiddles, true);

        let domain_root = field::get_root_of_unity(domain_size as u64);
        let twiddles = fft::get_twiddles(domain_root, domain_size);
        unsafe { result.set_len(domain_size); }
        polys::eval_fft_twiddles(&mut result, &twiddles, true);

        // 3 ----- divide linear combination of constraints by zero polynomial --------------------
        let domain = field::get_power_series(domain_root, domain_size);
        let z_inverse = self.get_inv_zero_poly(&domain);
        parallel::mul_in_place(&mut result, &z_inverse, 1);

        // 4 ----- merge input/output constraints into the linear combination ---------------------
        let x_at_last_step = *domain.last().unwrap();
        let mut stack_register_idx = 0;

        let incremental_degree = domain_size - self.trace_length();
        let x_root = field::exp(domain_root, incremental_degree as u64);
        let x_di = field::get_power_series(x_root, domain_size);

        // evaluate and merge input-output constraints into the linear combination
        if self.io_constraints[0].len() > 0 {
            // compute inverses of Z(x) evaluations
            let z_poly = polys::mul(&[field::neg(ONE), ONE], &[field::neg(x_at_last_step), ONE]);
            let z_values = eval_poly(&z_poly, &twiddles);
            let z_inverses = parallel::inv(&z_values, 1);

            // B(x) = (P(x) - I(x)) / Z(x)
            for i_poly in self.io_constraints[0].iter() {
                // evaluate boundary constraint
                let p_values = trace.get_stack_register_trace(stack_register_idx);
                let mut b_values = eval_boundary_constraint(&i_poly, &p_values, &z_inverses, &twiddles);

                // merge constraint evaluations into linear combination
                parallel::mul_acc(&mut result, &b_values, coefficients.next().unwrap(), 1);

                // adjust evaluation degree and merge the adjusted evaluations into linear combination
                parallel::mul_in_place(&mut b_values, &x_di, 1);
                parallel::mul_acc(&mut result, &b_values, coefficients.next().unwrap(), 1);

                stack_register_idx += 1;
            }
        }
        
        // evaluate and merge input-only constraints into the linear combination
        if self.io_constraints[1].len() > 0 {

        }

        // evaluate and merge output-only constraints into the linear combination
        if self.io_constraints[2].len() > 0 {

        }

        // 5 ----- merge program hash constraint into the linear combination ----------------------

        return result;
    }

    fn group_constraints(&self) -> [Vec<Vec<u64>>; MAX_CONSTRAINT_DEGREE + 1] {
        let mut result = [
            Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(),
            Vec::new(), Vec::new(), Vec::new(), Vec::new()
        ];

        let constraints = [
            &self.decoder[..],
            &self.op_acc[..],
            &self.stack[..]
        ].concat();

        let degrees = [
            &decoder::CONSTRAINT_DEGREES[..],
            &hash_acc::CONSTRAINT_DEGREES[..],
            &stack::CONSTRAINT_DEGREES[..self.stack.len()]
        ].concat();

        for (constraint, degree) in constraints.into_iter().zip(degrees.into_iter()) {
            result[degree].push(constraint);
        }

        return result;
    }

    fn get_inv_zero_poly(&self, domain: &[u64]) -> Vec<u64> {

        // compute x^trace_length for all x values in the domain
        let mut x_to_the_steps = uninit_vector(domain.len());
        for i in 0..x_to_the_steps.len() {
            x_to_the_steps[i] = domain[(i * self.trace_length()) % domain.len()];
        }

        // get value of x at the last step
        let x_at_last_step = *domain.last().unwrap();

        // compute z numerators and denominators separately
        let mut numerators = x_to_the_steps;
        parallel::sub_const_in_place(&mut numerators, field::ONE, 1);

        let mut denominators = domain.to_vec(); // TODO
        parallel::sub_const_in_place(&mut denominators, x_at_last_step, 1);

        // invert denominators and multiply the inverses by the numerators
        let mut result = parallel::inv(&denominators, 1);
        parallel::mul_in_place(&mut result, &numerators, 1);

        return result;
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn create_vectors(num_columns: usize, trace_length: usize) -> Vec<Vec<u64>> {
    let mut result = Vec::with_capacity(num_columns);
    for _ in 0..num_columns {
        result.push(uninit_vector(trace_length));
    }
    return result;
}

fn copy_constraints(source: &[u64], target: &mut Vec<Vec<u64>>, step: usize, should_be_zero: bool) {
    if should_be_zero {
        for i in 0..source.len() {
            assert!(source[i] == 0, "constraint at step {} didn't evaluate to 0", step / MAX_CONSTRAINT_DEGREE);
            target[i][step] = source[i];
        }
    }
    else {
        for i in 0..source.len() {
            target[i][step] = source[i];
        }
    }
}

fn eval_boundary_constraint(i_poly: &[u64], p_values: &[u64], z_inverses: &[u64], twiddles: &[u64]) -> Vec<u64> {
    let mut i_values = eval_poly(i_poly, twiddles);
    // TODO: implement parallel subtraction
    for i in 0..p_values.len() {
        i_values[i] = field::sub(p_values[i], i_values[i]);
    }
    parallel::mul_in_place(&mut i_values, &z_inverses, 1);
    return i_values;
}

fn eval_poly(poly: &[u64], twiddles: &[u64]) -> Vec<u64> {
    let domain_size = twiddles.len() * 2;
    let mut values = zero_filled_vector(domain_size, domain_size);
    values[..poly.len()].copy_from_slice(poly);
    polys::eval_fft_twiddles(&mut values, &twiddles, true);
    return values;
}