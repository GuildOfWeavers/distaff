use crate::math::{ field };
use super::utils::{ agg_op_constraint, is_binary, are_equal, enforce_no_change };

// CONSTANTS
// ================================================================================================

const POW2_IDX      : usize = 0;
const X_BIT_IDX     : usize = 1;
const Y_BIT_IDX     : usize = 2;
const NOT_SET_IDX   : usize = 3;
const GT_IDX        : usize = 4;
const LT_IDX        : usize = 5;
const Y_ACC_IDX     : usize = 6;
const X_ACC_IDX     : usize = 7;

// CONSTRAINT EVALUATORS
// ================================================================================================

/// Evaluates constraints for EQ operation. These enforce that when x == y, top of the stack at
/// the next step is set to 1, otherwise top of the stack at the next step is set to 0.
pub fn enforce_eq(evaluations: &mut [u128], current: &[u128], next: &[u128], aux: u128, op_flag: u128) -> u128 {

    // compute difference between top two values of the stack
    let x = current[0];
    let y = current[1];
    let diff = field::sub(x, y);

    // aux stack register contains inverse of the difference, or when
    // the values are equal, it will contain value 1
    let inv_diff = aux;

    // the operation is defined as 1 - diff * inv(diff)
    let op_result = field::sub(field::ONE, field::mul(diff, inv_diff));
    evaluations[0] = agg_op_constraint(evaluations[0], op_flag, are_equal(next[0], op_result));

    // stack items beyond 2nd item are shifted the the left by 1
    let n = next.len() - 1;
    enforce_no_change(&mut evaluations[1..n], &current[2..], &next[1..n], op_flag);

    // we also need to make sure that result * diff = 0; this ensures that when diff != 0
    // the result must be set to 0
    let aux_constraint = field::mul(op_flag, field::mul(next[0], diff));
    return aux_constraint;
}

/// Evaluates constraints for CMP operation.
pub fn enforce_cmp(evaluations: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {

    // layout of first 8 registers
    // [pow, bit_a, bit_b, not_set, gt, lt, acc_b, acc_a]

    // x and y bits are binary
    let x_bit = next[X_BIT_IDX];
    let y_bit = next[Y_BIT_IDX];
    evaluations[0] = agg_op_constraint(evaluations[0], op_flag, is_binary(x_bit));
    evaluations[1] = agg_op_constraint(evaluations[1], op_flag, is_binary(y_bit));

    // comparison trackers were updated correctly
    let not_set = next[NOT_SET_IDX];
    let bit_gt = field::mul(x_bit, field::sub(field::ONE, y_bit));
    let bit_lt = field::mul(y_bit, field::sub(field::ONE, x_bit));

    let gt = field::add(current[GT_IDX], field::mul(bit_gt, not_set));
    let lt = field::add(current[LT_IDX], field::mul(bit_lt, not_set));
    evaluations[2] = agg_op_constraint(evaluations[2], op_flag, are_equal(next[GT_IDX], gt));
    evaluations[3] = agg_op_constraint(evaluations[3], op_flag, are_equal(next[LT_IDX], lt));

    // binary representation accumulators were updated correctly
    let power_of_two = current[POW2_IDX];
    let x_acc = field::add(current[X_ACC_IDX], field::mul(x_bit, power_of_two));
    let y_acc = field::add(current[Y_ACC_IDX], field::mul(y_bit, power_of_two));
    evaluations[4] = agg_op_constraint(evaluations[4], op_flag, are_equal(next[Y_ACC_IDX], y_acc));
    evaluations[5] = agg_op_constraint(evaluations[5], op_flag, are_equal(next[X_ACC_IDX], x_acc));

    // when GT or LT register is set to 1, not_set flag is cleared
    let not_set_check = field::mul(field::sub(field::ONE, current[LT_IDX]), field::sub(field::ONE, current[GT_IDX]));
    evaluations[6] = agg_op_constraint(evaluations[6], op_flag, are_equal(not_set, not_set_check));

    // power of 2 register was updated correctly
    let power_of_two_constraint = are_equal(field::mul(next[POW2_IDX], 2), power_of_two);
    evaluations[7] = agg_op_constraint(evaluations[7], op_flag, power_of_two_constraint);

    // registers beyond the 7th register were not affected
    enforce_no_change(&mut evaluations[8..], &current[8..], &next[8..], op_flag);
}

pub fn enforce_binacc(evaluations: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {

    // layout of first 3 registers:
    // [power of two, value bit, accumulated value]
    // value bit is located in the next state (not current state)

    // power of 2 register was updated correctly
    let power_of_two = current[0];
    let power_of_two_constraint = are_equal(field::mul(next[0], 2), power_of_two);
    evaluations[0] = agg_op_constraint(evaluations[0], op_flag, power_of_two_constraint);

    // the bit was a binary value
    let bit = next[1];
    evaluations[1] = agg_op_constraint(evaluations[1], op_flag, is_binary(bit));

    // binary representation accumulator was updated correctly
    let acc = field::add(current[2], field::mul(bit, power_of_two));
    evaluations[2] = agg_op_constraint(evaluations[2], op_flag, are_equal(next[2], acc));

    // registers beyond 2nd register remained the same
    enforce_no_change(&mut evaluations[3..], &current[3..], &next[3..], op_flag);
}