use crate::math::{ field, polynom, fft };
use crate::utils::{ filled_vector };
use crate::{ BASE_CYCLE_LENGTH };

#[inline(always)]
pub fn is_zero(v: u128) -> u128 {
    return v;
}

#[inline(always)]
pub fn is_binary(v: u128) -> u128 {
    return field::sub(field::mul(v, v), v);
}

#[inline(always)]
pub fn binary_not(v: u128) -> u128 {
    return field::sub(field::ONE, v);
}

#[inline(always)]
pub fn are_equal(v1: u128, v2: u128) -> u128 {
    return field::sub(v1, v2);
}


pub trait EvaluationResult {

    fn agg_constraint(&mut self, index: usize, flag: u128, value: u128);

}

impl EvaluationResult for [u128] {

    fn agg_constraint(&mut self, index: usize, flag: u128, value: u128) {
        self[index] = field::add(self[index], field::mul(flag, value));
    }

}

impl EvaluationResult for Vec<u128> {

    fn agg_constraint(&mut self, index: usize, flag: u128, value: u128) {
        self[index] = field::add(self[index], field::mul(flag, value));
    }

}

pub fn extend_constants(constants: &[[u128; BASE_CYCLE_LENGTH]], extension_factor: usize) -> (Vec<Vec<u128>>, Vec<Vec<u128>>)
{
    let root = field::get_root_of_unity(BASE_CYCLE_LENGTH);
    let inv_twiddles = fft::get_inv_twiddles(root, BASE_CYCLE_LENGTH);

    let domain_size = BASE_CYCLE_LENGTH * extension_factor;
    let domain_root = field::get_root_of_unity(domain_size);
    let twiddles = fft::get_twiddles(domain_root, domain_size);

    let mut polys = Vec::with_capacity(constants.len());
    let mut evaluations = Vec::with_capacity(constants.len());

    for constant in constants.iter() {
        let mut extended_constant = filled_vector(BASE_CYCLE_LENGTH, domain_size, field::ZERO);
        extended_constant.copy_from_slice(constant);

        polynom::interpolate_fft_twiddles(&mut extended_constant, &inv_twiddles, true);
        polys.push(extended_constant.clone());

        unsafe { extended_constant.set_len(extended_constant.capacity()); }
        polynom::eval_fft_twiddles(&mut extended_constant, &twiddles, true);

        evaluations.push(extended_constant);
    }

    return (polys, evaluations);
}