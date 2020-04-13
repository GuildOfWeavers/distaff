use crate::math::{ field, parallel, fft, polys };
use crate::stark::{ TraceState };
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
}

// CONSTRAINT TABLE IMPLEMENTATION
// ================================================================================================
impl ConstraintTable {

    pub fn new(domain_size: usize, max_stack_depth: usize) -> ConstraintTable {
        debug_assert!(domain_size.is_power_of_two(), "domain size must be a power of 2");
        debug_assert!(domain_size % MAX_CONSTRAINT_DEGREE == 0, "domain size must be divisible by 8");

        return ConstraintTable {
            decoder : create_vectors(decoder::CONSTRAINT_DEGREES.len(), domain_size),
            op_acc  : create_vectors(hash_acc::CONSTRAINT_DEGREES.len(), domain_size),
            stack   : create_vectors(max_stack_depth, domain_size),
        };
    }

    pub fn evaluate(&mut self, current: &TraceState, next: &TraceState, step: usize) {
        let should_be_zero = (step % MAX_CONSTRAINT_DEGREE == 0)
            && (step < self.len() - MAX_CONSTRAINT_DEGREE);

        let op_dec = decoder::evaluate(&current, &next);
        copy_constraints(&op_dec, &mut self.decoder, step, should_be_zero);
        
        let op_acc = hash_acc::evaluate(&current, &next, step);
        copy_constraints(&op_acc, &mut self.op_acc, step, should_be_zero);

        let stack = stack::evaluate(&current, &next, self.stack.len());
        copy_constraints(&stack, &mut self.stack, step, should_be_zero);
    }

    pub fn constraint_count(&self) -> usize {
        return self.decoder.len() + self.op_acc.len() + self.stack.len();
    }

    pub fn len(&self) -> usize {
        return self.decoder[0].len();
    }

    pub fn get_composition_poly(&self, seed: &[u64; 4], extension_factor: usize) -> Vec<u64> {

        let trace_length = self.len() / MAX_CONSTRAINT_DEGREE;
        let domain_size = trace_length * extension_factor;

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
            let degree = degree * trace_length;

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
        let z_inverse = self.get_inv_zero_poly(domain_root, trace_length, extension_factor);
        parallel::mul_in_place(&mut result, &z_inverse, 1);

        // 4 ----- merge boundary constraints into linear combination -----------------------------
        // TODO

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

    fn get_inv_zero_poly(&self, domain_root: u64, trace_length: usize, extension_factor: usize) -> Vec<u64> {

        // build the domain for Z(x)
        let domain_size = trace_length * extension_factor;
        let domain = field::get_power_series(domain_root, domain_size);

        // compute x^trace_length for all x values in the domain
        let mut x_to_the_steps = uninit_vector(domain_size);
        for i in 0..x_to_the_steps.len() {
            x_to_the_steps[i] = domain[(i * trace_length) % domain_size];
        }

        // compute value of x at the last step (in the extended domain)
        let last_position = (trace_length - 1) * extension_factor;
        let x_at_last_step = field::exp(domain_root, last_position as u64);

        // compute z numerators and denominators separately
        let mut numerators = x_to_the_steps;
        parallel::sub_const_in_place(&mut numerators, field::ONE, 1);

        let mut denominators = domain;
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