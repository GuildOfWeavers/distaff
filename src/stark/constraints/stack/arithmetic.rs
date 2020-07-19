use crate::math::{ field };
use super::{ are_equal, enforce_no_change, EvaluationResult };

// ARITHMETIC OPERATION
// ================================================================================================

/// Enforces constraints for ADD operation. The constraints are based on the first 2 elements of
/// the stack; the rest of the stack is shifted left by 1 element.
pub fn enforce_add(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {

    let x = current[0];
    let y = current[1];
    let op_result = field::add(x, y);
    result.agg_constraint(0, op_flag, are_equal(next[0], op_result));

    // ensure that the rest of the stack is shifted left by 1 element
    let n = next.len() - 1;
    enforce_no_change(&mut result[1..n], &current[2..], &next[1..n], op_flag);
}

/// Enforces constraints for MUL operation. The constraints are based on the first 2 elements of
/// the stack; the rest of the stack is shifted left by 1 element.
pub fn enforce_mul(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {

    let x = current[0];
    let y = current[1];
    let op_result = field::mul(x, y);
    result.agg_constraint(0, op_flag, are_equal(next[0], op_result));

    // ensure that the rest of the stack is shifted left by 1 element
    let n = next.len() - 1;
    enforce_no_change(&mut result[1..n], &current[2..], &next[1..n], op_flag);
}

/// Enforces constraints for INV operation. The constraints are based on the first element of
/// the stack; the rest of the stack is unaffected.
pub fn enforce_inv(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {

    // Constraints for INV operation is defined as: x * inv(x) = 1; this also means
    // that if x = 0, the constraint will not be satisfied
    let x = current[0];
    let inv_x = next[0];
    result.agg_constraint(0, op_flag, are_equal(field::ONE, field::mul(inv_x, x)));

    // ensure nothing changed beyond the first item of the stack 
    enforce_no_change(&mut result[1..], &current[1..], &next[1..], op_flag);
}

/// Enforces constraints for NEG operation. The constraints are based on the first element of
/// the stack; the rest of the stack is unaffected.
pub fn enforce_neg(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {

    // Constraint for NEG operation is defined as: x + neg(x) = 0
    let x = current[0];
    let neg_x = next[0];
    result.agg_constraint(0, op_flag, field::add(neg_x, x));

    // ensure nothing changed beyond the first item of the stack 
    enforce_no_change(&mut result[1..], &current[1..], &next[1..], op_flag);
}