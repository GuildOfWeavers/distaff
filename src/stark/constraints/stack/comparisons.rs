use crate::math::{ FiniteField };
use super::utils::{ agg_op_constraint, is_binary, are_equal, enforce_no_change };

// CONSTANTS
// ================================================================================================

const POW2_IDX  : usize = 0;
const X_BIT_IDX : usize = 1;
const Y_BIT_IDX : usize = 2;
const GT_IDX    : usize = 3;
const LT_IDX    : usize = 4;
const Y_ACC_IDX : usize = 5;
const X_ACC_IDX : usize = 6;

// CONSTRAINT EVALUATORS
// ================================================================================================

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

    // x and y bits are binary
    let x_bit = next[X_BIT_IDX];
    let y_bit = next[Y_BIT_IDX];
    evaluations[0] = agg_op_constraint(evaluations[0], op_flag, is_binary(x_bit));
    evaluations[1] = agg_op_constraint(evaluations[1], op_flag, is_binary(y_bit));

    // comparison trackers were updated correctly
    let not_set = aux;
    let bit_gt = T::mul(x_bit, T::sub(T::ONE, y_bit));
    let bit_lt = T::mul(y_bit, T::sub(T::ONE, x_bit));

    let gt = T::add(current[GT_IDX], T::mul(bit_gt, not_set));
    let lt = T::add(current[LT_IDX], T::mul(bit_lt, not_set));
    evaluations[2] = agg_op_constraint(evaluations[2], op_flag, are_equal(next[GT_IDX], gt));
    evaluations[3] = agg_op_constraint(evaluations[3], op_flag, are_equal(next[LT_IDX], lt));

    // binary representation accumulators were updated correctly
    let power_of_two = current[POW2_IDX];
    let x_acc = T::add(current[X_ACC_IDX], T::mul(x_bit, power_of_two));
    let y_acc = T::add(current[Y_ACC_IDX], T::mul(y_bit, power_of_two));
    evaluations[4] = agg_op_constraint(evaluations[4], op_flag, are_equal(next[Y_ACC_IDX], y_acc));
    evaluations[5] = agg_op_constraint(evaluations[5], op_flag, are_equal(next[X_ACC_IDX], x_acc));

    // power of 2 register was updated correctly
    let power_of_two_constraint = are_equal(T::mul(next[POW2_IDX], T::from_usize(2)), power_of_two);
    evaluations[6] = agg_op_constraint(evaluations[6], op_flag, power_of_two_constraint);

    // registers beyond the 7th register were not affected
    enforce_no_change(&mut evaluations[7..], &current[7..], &next[7..], op_flag);

    // when GT or LT register is set to 1, not_set flag is cleared
    let not_set_check = T::mul(T::sub(T::ONE, current[LT_IDX]), T::sub(T::ONE, current[GT_IDX]));
    let aux_constraint = T::mul(op_flag, T::sub(not_set, not_set_check));
    return aux_constraint;
}