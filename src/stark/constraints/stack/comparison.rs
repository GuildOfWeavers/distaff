use super::{ field, are_equal, is_binary, binary_not, enforce_no_change, EvaluationResult };

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

// ASSERTIONS
// ================================================================================================

/// Enforces constraints for ASSERT operation. The constraints are similar to DROP operation, but
/// have an auxiliary constraint which enforces that 1 - x = 0, where x is the top of the stack.
pub fn enforce_assert(result: &mut [u128], aux: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    let n = next.len() - 1;
    enforce_no_change(&mut result[0..n], &current[1..], &next[0..n], op_flag);
    aux.agg_constraint(0, op_flag, are_equal(field::ONE, current[0]));
}

/// Enforces constraints for ASSERTEQ operation. The stack is shifted by 2 registers the left and
/// an auxiliary constraint enforces that the first element of the stack is equal to the second.
pub fn enforce_asserteq(result: &mut [u128], aux: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    let n = next.len() - 2;
    enforce_no_change(&mut result[0..n], &current[2..], &next[0..n], op_flag);
    aux.agg_constraint(0, op_flag, are_equal(current[0], current[1]));
}

// EQUALITY
// ================================================================================================

/// Evaluates constraints for EQ operation. These enforce that when x == y, top of the stack at
/// the next step is set to 1, otherwise top of the stack at the next step is set to 0.
pub fn enforce_eq(result: &mut [u128], aux: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {

    // compute difference between top two values of the stack
    let x = current[1];
    let y = current[2];
    let diff = field::sub(x, y);

    // when x == y, the first stack register contains inverse of the difference
    let inv_diff = current[0];

    // the operation is defined as 1 - diff * inv(diff)
    let op_result = binary_not(field::mul(diff, inv_diff));
    result.agg_constraint(0, op_flag, are_equal(next[0], op_result));

    // stack items beyond 3nd item are shifted the the left by 2
    let n = next.len() - 2;
    enforce_no_change(&mut result[1..n], &current[3..], &next[1..n], op_flag);

    // we also need to make sure that result * diff = 0; this ensures that when diff != 0
    // the result must be set to 0
    aux.agg_constraint(0, op_flag, field::mul(next[0], diff));
}

// INEQUALITY
// ================================================================================================

/// Evaluates constraints for CMP operation.
pub fn enforce_cmp(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {

    // layout of first 8 registers
    // [pow, bit_a, bit_b, not_set, gt, lt, acc_b, acc_a]

    // x and y bits are binary
    let x_bit = next[X_BIT_IDX];
    let y_bit = next[Y_BIT_IDX];
    result.agg_constraint(0, op_flag, is_binary(x_bit));
    result.agg_constraint(1, op_flag, is_binary(y_bit));

    // comparison trackers were updated correctly
    let not_set = next[NOT_SET_IDX];
    let bit_gt = field::mul(x_bit, binary_not(y_bit));
    let bit_lt = field::mul(y_bit, binary_not(x_bit));

    let gt = field::add(current[GT_IDX], field::mul(bit_gt, not_set));
    let lt = field::add(current[LT_IDX], field::mul(bit_lt, not_set));
    result.agg_constraint(2, op_flag, are_equal(next[GT_IDX], gt));
    result.agg_constraint(3, op_flag, are_equal(next[LT_IDX], lt));

    // binary representation accumulators were updated correctly
    let power_of_two = current[POW2_IDX];
    let x_acc = field::add(current[X_ACC_IDX], field::mul(x_bit, power_of_two));
    let y_acc = field::add(current[Y_ACC_IDX], field::mul(y_bit, power_of_two));
    result.agg_constraint(4, op_flag, are_equal(next[Y_ACC_IDX], y_acc));
    result.agg_constraint(5, op_flag, are_equal(next[X_ACC_IDX], x_acc));

    // when GT or LT register is set to 1, not_set flag is cleared
    let not_set_check = field::mul(binary_not(current[LT_IDX]), binary_not(current[GT_IDX]));
    result.agg_constraint(6, op_flag, are_equal(not_set, not_set_check));

    // power of 2 register was updated correctly
    let power_of_two_constraint = are_equal(field::mul(next[POW2_IDX], 2), power_of_two);
    result.agg_constraint(7, op_flag, power_of_two_constraint);

    // registers beyond the 7th register were not affected
    for i in 8..result.len() {
        result.agg_constraint(i, op_flag, are_equal(current[i], next[i]));
    }
}

/// Evaluates constraints for BINACC operation.
pub fn enforce_binacc(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {

    // layout of first 3 registers:
    // [power of two, value bit, accumulated value]
    // value bit is located in the next state (not current state)

    // power of 2 register was updated correctly
    let power_of_two = current[0];
    let power_of_two_constraint = are_equal(field::mul(next[0], 2), power_of_two);
    result.agg_constraint(0, op_flag, power_of_two_constraint);

    // the bit was a binary value
    let bit = next[1];
    result.agg_constraint(1, op_flag, is_binary(bit));

    // binary representation accumulator was updated correctly
    let acc = field::add(current[2], field::mul(bit, power_of_two));
    result.agg_constraint(2, op_flag, are_equal(next[2], acc));

    // registers beyond 2nd register remained the same
    for i in 3..result.len() {
        result.agg_constraint(i, op_flag, are_equal(current[i], next[i]));
    }
}