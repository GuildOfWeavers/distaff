use crate::math::{ field, polynom, parallel, fft };
use crate::stark::{ TraceTable, MAX_REGISTER_COUNT };
use crate::utils::{ zero_filled_vector, CopyInto };
use super::{ ConstraintPolys, MAX_CONSTRAINT_DEGREE };

// TYPES AND INTERFACES
// ================================================================================================
pub struct CompositionPoly {
    evaluations     : Vec<u64>,
    t_deep_points1  : Vec<u64>,
    t_deep_points2  : Vec<u64>,
    c_deep_points   : Vec<u64>,
}

struct Coefficients {
    pub constraints : [u64; MAX_CONSTRAINT_DEGREE],
    pub trace1      : [u64; MAX_REGISTER_COUNT * 2],
    pub trace2      : [u64; MAX_REGISTER_COUNT * 2],
}

// COMPOSITION POLY IMPLEMENTATION
// ================================================================================================
impl CompositionPoly {

    pub fn new(trace: &TraceTable, constraints: &ConstraintPolys, seed: &[u64; 4], z: u64) -> CompositionPoly {
        
        // allocate space for the polynomial and create coefficients for linear combination
        let mut composition_poly = zero_filled_vector(trace.unextended_length(), trace.domain_size());
        let cc = Coefficients::new(seed);

        // combine all constraint polynomials as follows:
        // first, divide out deep points for all polynomials A'(x) = (A(x) - A(z^d)) / (x - z^d)
        // then put them all into a random linear combination using pseudo-random coefficients
        let z_pow = field::exp(z, MAX_CONSTRAINT_DEGREE as u64);
        let constraint_deep_points = constraints.evaluate_at(z_pow);
        for i in 0..constraints.poly_count() {
            let mut constraint_poly = constraints.get_poly(i).to_vec();
            constraint_poly[0] = field::sub(constraint_poly[0], constraint_deep_points[i]);
            polynom::syn_div_in_place(&mut constraint_poly, field::neg(z_pow));
            parallel::mul_acc(&mut composition_poly, &constraint_poly, cc.constraints[i], 1);
        }

        // combine all trace polynomials
        let domain_root = field::get_root_of_unity(trace.domain_size() as u64);
        let next_z = field::mul(z, domain_root);

        let trace_deep_points1 = trace.eval_polys_at(z);
        let trace_deep_points2 = trace.eval_polys_at(next_z); 

        for i in 0..trace.register_count() {
            // compute T1(x) = (T(x) - T(z)) / (x - z), multiply it by a pseudo-random coefficient,
            // and add the result into composition polynomial
            let mut trace_poly = trace.get_register_poly(i).to_vec();
            trace_poly[0] = field::sub(trace_poly[0], trace_deep_points1[i]);
            polynom::syn_div_in_place(&mut trace_poly, field::neg(z));
            parallel::mul_acc(&mut composition_poly, &trace_poly, cc.trace1[i], 1);

            // compute T2(x) = (T(x) - T(z * domain_root)) / (x - z * domain_root) multiply it by a
            // pseudo-random coefficient, and add the result into composition polynomial
            let mut trace_poly = trace.get_register_poly(i).to_vec();
            trace_poly[0] = field::sub(trace_poly[0], trace_deep_points2[i]);
            polynom::syn_div_in_place(&mut trace_poly, field::neg(next_z));
            parallel::mul_acc(&mut composition_poly, &trace_poly, cc.trace2[i], 1);
        }

        // evaluate the composition polynomial over the evaluation domain
        let twiddles = fft::get_twiddles(domain_root, trace.domain_size());
        unsafe { composition_poly.set_len(composition_poly.capacity()); }
        polynom::eval_fft_twiddles(&mut composition_poly, &twiddles, true);

        return CompositionPoly {
            evaluations     : composition_poly,
            t_deep_points1  : trace_deep_points1,
            t_deep_points2  : trace_deep_points2,
            c_deep_points   : constraint_deep_points,
        };
    }

    pub fn domain_size(&self) -> usize {
        return self.evaluations.len();
    }

    pub fn trace_deep_points1(&self) -> &[u64] {
        return &self.t_deep_points1;
    }

    pub fn trace_deep_points2(&self) -> &[u64] {
        return &self.t_deep_points2;
    }

    pub fn constraint_deep_points(&self) -> &[u64] {
        return &self.constraint_deep_points();
    }
}

// COEFFICIENTS IMPLEMENTATION
// ================================================================================================
impl Coefficients {

    pub fn new(seed: &[u64; 4]) -> Coefficients {

        // generate a pseudo-random list of coefficients
        let num_coefficients = MAX_CONSTRAINT_DEGREE + 4 * MAX_REGISTER_COUNT;
        let coefficients = field::prng_vector(seed.copy_into(), num_coefficients);

        // copy coefficients to their respective segments
        let mut constraints = [0u64; MAX_CONSTRAINT_DEGREE];
        constraints.copy_from_slice(&coefficients[..MAX_CONSTRAINT_DEGREE]);

        let mut trace1 = [0u64; 2 * MAX_REGISTER_COUNT];
        let end_index = MAX_CONSTRAINT_DEGREE + trace1.len();
        trace1.copy_from_slice(&coefficients[MAX_CONSTRAINT_DEGREE..end_index]);

        let start_index = end_index;
        let mut trace2 = [0u64; 2 * MAX_REGISTER_COUNT];
        trace2.copy_from_slice(&coefficients[start_index..]);

        return Coefficients { constraints, trace1, trace2 };
    }
}