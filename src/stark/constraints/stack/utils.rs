use crate::math::{ FiniteField };

#[inline(always)]
pub fn enforce_no_change<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
    for i in 0..result.len() {
        result[i] = agg_op_constraint(result[i], op_flag, T::sub(next[i], current[i]));
    }
}

#[inline(always)]
pub fn agg_op_constraint<T: FiniteField>(result: T, op_flag: T, op_constraint: T) -> T {
    return T::add(result, T::mul(op_constraint, op_flag));
}

#[inline(always)]
pub fn is_binary<T: FiniteField>(v: T) -> T {
    return T::sub(T::mul(v, v), v);
}

#[inline(always)]
pub fn are_equal<T: FiniteField>(v1: T, v2: T) -> T {
    return T::sub(v1, v2);
}