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

/// Evaluates constraints for EQ operation. These enforce that when x == y, top of the stack at
/// the next step is set to 1, otherwise top of the stack at the next step is set to 0.
pub fn enforce_eq(evaluations: &mut [u128], current: &[u128], next: &[u128], aux: u128, op_flag: u128) -> u128 {

    // compute difference between top two values of the stack
    let x = current[0];
    let y = current[1];
    let diff = u128::sub(x, y);

    // aux stack register contains inverse of the difference, or when
    // the values are equal, it will contain value 1
    let inv_diff = aux;

    // the operation is defined as 1 - diff * inv(diff)
    let op_result = u128::sub(u128::ONE, u128::mul(diff, inv_diff));
    evaluations[0] = agg_op_constraint(evaluations[0], op_flag, are_equal(next[0], op_result));

    // stack items beyond 2nd item are shifted the the left by 1
    let n = next.len() - 1;
    enforce_no_change(&mut evaluations[1..n], &current[2..], &next[1..n], op_flag);

    // we also need to make sure that result * diff = 0; this ensures that when diff != 0
    // the result must be set to 0
    let aux_constraint = u128::mul(op_flag, u128::mul(next[0], diff));
    return aux_constraint;
}

/// Evaluates constraints for CMP operation.
pub fn enforce_cmp(evaluations: &mut [u128], current: &[u128], next: &[u128], aux: u128, op_flag: u128) -> u128 {

    // x and y bits are binary
    let x_bit = next[X_BIT_IDX];
    let y_bit = next[Y_BIT_IDX];
    evaluations[0] = agg_op_constraint(evaluations[0], op_flag, is_binary(x_bit));
    evaluations[1] = agg_op_constraint(evaluations[1], op_flag, is_binary(y_bit));

    // comparison trackers were updated correctly
    let not_set = aux;
    let bit_gt = u128::mul(x_bit, u128::sub(u128::ONE, y_bit));
    let bit_lt = u128::mul(y_bit, u128::sub(u128::ONE, x_bit));

    let gt = u128::add(current[GT_IDX], u128::mul(bit_gt, not_set));
    let lt = u128::add(current[LT_IDX], u128::mul(bit_lt, not_set));
    evaluations[2] = agg_op_constraint(evaluations[2], op_flag, are_equal(next[GT_IDX], gt));
    evaluations[3] = agg_op_constraint(evaluations[3], op_flag, are_equal(next[LT_IDX], lt));

    // binary representation accumulators were updated correctly
    let power_of_two = current[POW2_IDX];
    let x_acc = u128::add(current[X_ACC_IDX], u128::mul(x_bit, power_of_two));
    let y_acc = u128::add(current[Y_ACC_IDX], u128::mul(y_bit, power_of_two));
    evaluations[4] = agg_op_constraint(evaluations[4], op_flag, are_equal(next[Y_ACC_IDX], y_acc));
    evaluations[5] = agg_op_constraint(evaluations[5], op_flag, are_equal(next[X_ACC_IDX], x_acc));

    // power of 2 register was updated correctly
    let power_of_two_constraint = are_equal(u128::mul(next[POW2_IDX], u128::from_usize(2)), power_of_two);
    evaluations[6] = agg_op_constraint(evaluations[6], op_flag, power_of_two_constraint);

    // registers beyond the 7th register were not affected
    enforce_no_change(&mut evaluations[7..], &current[7..], &next[7..], op_flag);

    // when GT or LT register is set to 1, not_set flag is cleared
    let not_set_check = u128::mul(u128::sub(u128::ONE, current[LT_IDX]), u128::sub(u128::ONE, current[GT_IDX]));
    let aux_constraint = u128::mul(op_flag, u128::sub(not_set, not_set_check));
    return aux_constraint;
}

pub fn enforce_binacc(evaluations: &mut [u128], current: &[u128], next: &[u128], aux: u128, op_flag: u128) -> u128 {

    let bit = aux;

    // power of 2 register was updated correctly
    let power_of_two = current[0];
    let power_of_two_constraint = are_equal(u128::mul(next[0], u128::from_usize(2)), power_of_two);
    evaluations[0] = agg_op_constraint(evaluations[0], op_flag, power_of_two_constraint);

    // binary representation accumulator was updated correctly
    let acc = u128::add(current[1], u128::mul(bit, power_of_two));
    evaluations[1] = agg_op_constraint(evaluations[4], op_flag, are_equal(next[1], acc));

    // registers beyond 2nd register remained the same
    enforce_no_change(&mut evaluations[2..], &current[2..], &next[2..], op_flag);

    // the bit was a binary value
    let aux_constraint = u128::mul(op_flag, is_binary(bit));
    return aux_constraint;
}