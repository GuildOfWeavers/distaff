use crate::math::{ field };

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