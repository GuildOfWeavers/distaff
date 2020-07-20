use super::{ are_equal, enforce_no_change, EvaluationResult };

// STACK MANIPULATION OPERATIONS
// ================================================================================================

/// Enforces constraints for DUP operation. The constraints are based on the first element
/// of the stack; the old stack is shifted right by 1 element.
pub fn enforce_dup(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    result.agg_constraint(0, op_flag, are_equal(next[0], current[0]));
    enforce_no_change(&mut result[1..], &current[0..], &next[1..], op_flag);
}

/// Enforces constraints for DUP2 operation. The constraints are based on the first 2 element
/// of the stack; the old stack is shifted right by 2 element.
pub fn enforce_dup2(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    result.agg_constraint(0, op_flag, are_equal(next[0], current[0]));
    result.agg_constraint(1, op_flag, are_equal(next[1], current[1]));
    enforce_no_change(&mut result[2..], &current[0..], &next[2..], op_flag);
}

/// Enforces constraints for DUP4 operation. The constraints are based on the first 4 element
/// of the stack; the old stack is shifted right by 4 element.
pub fn enforce_dup4(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    result.agg_constraint(0, op_flag, are_equal(next[0], current[0]));
    result.agg_constraint(1, op_flag, are_equal(next[1], current[1]));
    result.agg_constraint(2, op_flag, are_equal(next[2], current[2]));
    result.agg_constraint(3, op_flag, are_equal(next[3], current[3]));
    enforce_no_change(&mut result[4..], &current[0..], &next[4..], op_flag);
}

/// Enforces constraints for PAD2 operation. The constraints are based on the first 2 element
/// of the stack; the old stack is shifted right by 2 element.
pub fn enforce_pad2(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    result.agg_constraint(0, op_flag, next[0]);
    result.agg_constraint(1, op_flag, next[1]);
    enforce_no_change(&mut result[2..], &current[0..], &next[2..], op_flag);
}

// Enforces constraints for DROP operation. The stack is simply shifted left by 1 element.
pub fn enforce_drop(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    let n = next.len() - 1;
    enforce_no_change(&mut result[0..n], &current[1..], &next[0..n], op_flag);
}

// Enforces constraints for DROP4 operation. The stack is simply shifted left by 4 element.
pub fn enforce_drop4(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    let n = next.len() - 4;
    enforce_no_change(&mut result[0..n], &current[4..], &next[0..n], op_flag);
}

/// Enforces constraints for SWAP operation. The constraints are based on the first 2 element
/// of the stack; the rest of the stack is unaffected.
pub fn enforce_swap(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    result.agg_constraint(0, op_flag, are_equal(next[0], current[1]));
    result.agg_constraint(0, op_flag, are_equal(next[1], current[0]));
    enforce_no_change(&mut result[2..], &current[2..], &next[2..], op_flag);
}

/// Enforces constraints for SWAP2 operation. The constraints are based on the first 4 element
/// of the stack; the rest of the stack is unaffected.
pub fn enforce_swap2(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    result.agg_constraint(0, op_flag, are_equal(next[0], current[2]));
    result.agg_constraint(1, op_flag, are_equal(next[1], current[3]));
    result.agg_constraint(2, op_flag, are_equal(next[2], current[0]));
    result.agg_constraint(3, op_flag, are_equal(next[3], current[1]));
    enforce_no_change(&mut result[4..], &current[4..], &next[4..], op_flag);
}

/// Enforces constraints for SWAP4 operation. The constraints are based on the first 8 element
/// of the stack; the rest of the stack is unaffected.
pub fn enforce_swap4(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    result.agg_constraint(0, op_flag, are_equal(next[0], current[4]));
    result.agg_constraint(1, op_flag, are_equal(next[1], current[5]));
    result.agg_constraint(2, op_flag, are_equal(next[2], current[6]));
    result.agg_constraint(3, op_flag, are_equal(next[3], current[7]));
    result.agg_constraint(4, op_flag, are_equal(next[4], current[0]));
    result.agg_constraint(5, op_flag, are_equal(next[5], current[1]));
    result.agg_constraint(6, op_flag, are_equal(next[6], current[2]));
    result.agg_constraint(7, op_flag, are_equal(next[7], current[3]));
    enforce_no_change(&mut result[8..], &current[8..], &next[8..], op_flag);
}

/// Enforces constraints for ROLL4 operation. The constraints are based on the first 4 element
/// of the stack; the rest of the stack is unaffected.
pub fn enforce_roll4(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    result.agg_constraint(0, op_flag, are_equal(next[0], current[3]));
    result.agg_constraint(1, op_flag, are_equal(next[1], current[0]));
    result.agg_constraint(2, op_flag, are_equal(next[2], current[1]));
    result.agg_constraint(3, op_flag, are_equal(next[3], current[2]));
    enforce_no_change(&mut result[4..], &current[4..], &next[4..], op_flag);
}

/// Enforces constraints for ROLL8 operation. The constraints are based on the first 8 element
/// of the stack; the rest of the stack is unaffected.
pub fn enforce_roll8(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    result.agg_constraint(0, op_flag, are_equal(next[0], current[7]));
    result.agg_constraint(1, op_flag, are_equal(next[1], current[0]));
    result.agg_constraint(2, op_flag, are_equal(next[2], current[1]));
    result.agg_constraint(3, op_flag, are_equal(next[3], current[2]));
    result.agg_constraint(4, op_flag, are_equal(next[4], current[3]));
    result.agg_constraint(5, op_flag, are_equal(next[5], current[4]));
    result.agg_constraint(6, op_flag, are_equal(next[6], current[5]));
    result.agg_constraint(7, op_flag, are_equal(next[7], current[6]));
    enforce_no_change(&mut result[8..], &current[8..], &next[8..], op_flag);
}