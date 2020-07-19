use crate::math::{ field };
use super::{ are_equal, is_binary, binary_not, enforce_no_change, EvaluationResult };

// BOOLEAN OPERATION
// ================================================================================================

/// Enforces constraints for NOT operation. The constraints are based on the first element of
/// the stack, but also evaluates an auxiliary constraint which guarantees that the first
/// element of the stack is binary.
pub fn enforce_not(result: &mut [u128], aux: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {

    // NOT operation is defined simply as: 1 - x; this means 0 becomes 1, and 1 becomes 0
    let x = current[0];
    let op_result = binary_not(x);
    result.agg_constraint(0, op_flag, are_equal(next[0], op_result));

    // ensure nothing changed beyond the first item of the stack 
    enforce_no_change(&mut result[1..], &current[1..], &next[1..], op_flag);

    // we also need to make sure that the operand is binary (i.e. 0 or 1)
    aux.agg_constraint(0, op_flag, is_binary(x));
}

/// Enforces constraints for AND operation. The constraints are based on the first two elements
/// of the stack, but also evaluates auxiliary constraints which guarantee that both elements
/// are binary.
pub fn enforce_and(result: &mut [u128], aux: &mut[u128], current: &[u128], next: &[u128], op_flag: u128) {

    // AND operation is the same as: x * y
    let x = current[0];
    let y = current[1];
    let op_result = field::mul(x, y);
    result.agg_constraint(0, op_flag, are_equal(next[0], op_result));

    // ensure that the rest of the stack is shifted left by 1 element
    let n = next.len() - 1;
    enforce_no_change(&mut result[1..n], &current[2..], &next[1..n], op_flag);

    // ensure that both operands are binary values
    aux.agg_constraint(0, op_flag, is_binary(x));
    aux.agg_constraint(1, op_flag, is_binary(y));
}

/// Enforces constraints for OR operation. The constraints are based on the first two elements
/// of the stack, but also evaluates auxiliary constraints which guarantee that both elements
/// are binary.
pub fn enforce_or(result: &mut [u128], aux: &mut[u128], current: &[u128], next: &[u128], op_flag: u128) {

    // OR operation is the same as: 1 - (1 - x) * (1 - y)
    let x = current[0];
    let y = current[1];
    let op_result = binary_not(field::mul(binary_not(x), binary_not(y)));
    result.agg_constraint(0, op_flag, are_equal(next[0], op_result));

    // ensure that the rest of the stack is shifted left by 1 element
    let n = next.len() - 1;
    enforce_no_change(&mut result[1..n], &current[2..], &next[1..n], op_flag);

    // ensure that both operands are binary values
    aux.agg_constraint(0, op_flag, is_binary(x));
    aux.agg_constraint(1, op_flag, is_binary(y));
}