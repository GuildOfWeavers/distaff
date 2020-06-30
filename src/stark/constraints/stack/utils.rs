use crate::math::{ field };

#[inline(always)]
pub fn enforce_no_change(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    for i in 0..result.len() {
        result[i] = agg_op_constraint(result[i], op_flag, field::sub(next[i], current[i]));
    }
}

#[inline(always)]
pub fn agg_op_constraint(result: u128, op_flag: u128, op_constraint: u128) -> u128 {
    return field::add(result, field::mul(op_constraint, op_flag));
}

#[inline(always)]
pub fn is_binary(v: u128) -> u128 {
    return field::sub(field::mul(v, v), v);
}

#[inline(always)]
pub fn are_equal(v1: u128, v2: u128) -> u128 {
    return field::sub(v1, v2);
}