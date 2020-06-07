use crate::math::{ FiniteField };
use super::utils::{ agg_op_constraint, is_binary, enforce_no_change };

/// Evaluates constraints for EQ operation. This enforces that when x == y, top of the stack
/// is set to 1, otherwise top of the stack is set to 0.
pub fn enforce_eq<T: FiniteField>(evaluations: &mut [T], current: &[T], next: &[T], aux: T, op_flag: T) -> T {

    // compute difference between top two values of the stack
    let x = current[0];
    let y = current[1];
    let diff = T::sub(x, y);

    // aux stack register contains inverse of the difference, or when
    // the values are equal, it will contain value 1
    let inv_diff = aux;

    let op_result = T::sub(T::ONE, T::mul(diff, inv_diff));
    evaluations[0] = agg_op_constraint(evaluations[0], op_flag, T::sub(next[0], op_result));

    let n = next.len() - 1;
    enforce_no_change(&mut evaluations[1..n], &current[2..], &next[1..n], op_flag);

    let aux_constraint = T::mul(T::mul(next[0], diff), op_flag);
    return aux_constraint;
}

pub fn enforce_cmp<T: FiniteField>(evaluations: &mut [T], current: &[T], next: &[T], aux: T, op_flag: T) -> T {

    let a_bit = next[0];
    let b_bit = next[1];
    evaluations[0] = agg_op_constraint(evaluations[0], op_flag, is_binary(a_bit));
    evaluations[1] = agg_op_constraint(evaluations[1], op_flag, is_binary(b_bit));

    let bit_gt = T::mul(a_bit, T::sub(T::ONE, b_bit));
    let bit_lt = T::mul(b_bit, T::sub(T::ONE, a_bit));
    let not_set = aux;

    let gt = T::add(current[2], T::mul(bit_gt, not_set));
    let lt = T::add(current[3], T::mul(bit_lt, not_set));
    evaluations[2] = agg_op_constraint(evaluations[2], op_flag, T::sub(next[2], gt));
    evaluations[3] = agg_op_constraint(evaluations[3], op_flag, T::sub(next[3], lt));

    let power_of_two = current[6];
    let a_acc = T::add(current[4], T::mul(a_bit, power_of_two));
    let b_acc = T::add(current[5], T::mul(b_bit, power_of_two));
    evaluations[4] = agg_op_constraint(evaluations[4], op_flag, T::sub(next[4], a_acc));
    evaluations[5] = agg_op_constraint(evaluations[5], op_flag, T::sub(next[5], b_acc));

    let power_of_two_check = T::mul(next[6], T::from_usize(2));
    evaluations[6] = agg_op_constraint(evaluations[6], op_flag, T::sub(power_of_two, power_of_two_check));

    enforce_no_change(&mut evaluations[7..], &current[7..], &next[7..], op_flag);

    let not_set_check = T::mul(T::sub(T::ONE, current[2]), T::sub(T::ONE, current[3]));
    let aux_constraint = T::mul(op_flag, T::sub(not_set, not_set_check));
    return aux_constraint;
}