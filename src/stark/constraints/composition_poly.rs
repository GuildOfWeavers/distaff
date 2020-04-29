use crate::math::{ field, polynom, parallel, fft };
use crate::stark::{ TraceTable };
use crate::utils::{ zero_filled_vector };
use super::{ ConstraintPolys, MAX_CONSTRAINT_DEGREE };

// TYPES AND INTERFACES
// ================================================================================================
pub struct CompositionPoly {
    evaluations     : Vec<u64>,
    t_deep_points1  : Vec<u64>,
    t_deep_points2  : Vec<u64>,
    c_deep_points   : Vec<u64>,
}

// COMPOSITION POLY IMPLEMENTATION
// ================================================================================================
impl CompositionPoly {

    pub fn new(trace: &TraceTable, constraints: &ConstraintPolys, z: u64) -> CompositionPoly {
        
        let trace_length = trace.unextended_length();
        let domain_size = trace.domain_size();
        let mut composition_poly = zero_filled_vector(trace_length, domain_size);

        let domain_root = field::get_root_of_unity(trace.domain_size() as u64);
        let next_z = field::mul(z, domain_root); // TODO: trace root?

        let trace_deep_points1 = trace.eval_polys_at(z);
        let trace_deep_points2 = trace.eval_polys_at(next_z); 

        for i in 0..trace.register_count() {
            let mut trace_poly = trace.get_register_poly(i).to_vec();
            trace_poly[0] = field::sub(trace_poly[0], trace_deep_points1[i]);
            polynom::syn_div_in_place(&mut trace_poly, field::neg(z));
            parallel::mul_acc(&mut composition_poly, &trace_poly, 1, 1);    // TODO: apply coefficient

            let mut trace_poly = trace.get_register_poly(i).to_vec();
            trace_poly[0] = field::sub(trace_poly[0], trace_deep_points2[i]);
            polynom::syn_div_in_place(&mut trace_poly, field::neg(next_z));
            parallel::mul_acc(&mut composition_poly, &trace_poly, 1, 1);    // TODO: apply coefficient
        }

        let z_pow = field::exp(z, MAX_CONSTRAINT_DEGREE as u64);
        let constraint_deep_points = constraints.evaluate_at(z_pow);
        for i in 0..constraints.poly_count() {
            let mut constraint_poly = constraints.get_poly(i).to_vec();
            constraint_poly[0] = field::sub(constraint_poly[0], constraint_deep_points[i]);
            polynom::syn_div_in_place(&mut constraint_poly, field::neg(z_pow));
            parallel::mul_acc(&mut composition_poly, &constraint_poly, 1, 1);   // TODO: apply coefficient
        }

        let twiddles = fft::get_twiddles(domain_root, domain_size);
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
}