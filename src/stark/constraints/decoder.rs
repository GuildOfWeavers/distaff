use crate::math::field::{ sub, mul, ONE };
use crate::stark::{ TraceState };
use crate::processor::{ opcodes };

// CONSTANTS
// ================================================================================================
pub const CONSTRAINT_DEGREES: [usize; 9] = [
    2, 2, 2, 2, 2,  // op_bits are binary
    2,              // push_flag is binary
    5,              // push_flag is set after a PUSH operation
    2,              // push_flag gets reset on the next step
    2,              // when push_flag = 0, op_bits are a binary decomposition of op_code
];

// EVALUATOR FUNCTION
// ================================================================================================
pub fn evaluate(current: &TraceState, next: &TraceState, table: &mut Vec<Vec<u64>>, step: usize) {

    // constraint counter
    let mut i = 0;

    // 5 constraints, degree 2: op_bits must be binary
    let op_bits = current.get_op_bits();
    for _ in 0..5 {
        table[i][step] = is_binary(op_bits[i]);
        i += 1;
    }

    // 1 constraint, degree 2: push_flag must be binary
    table[i][step] = is_binary(current.get_push_flag());
    i += 1;

    // 1 constraint, degree 5: push_flag must be set to 1 after a PUSH operation
    let op_flags = current.get_op_flags();
    table[i][step] = sub(op_flags[opcodes::PUSH as usize], next.get_push_flag());
    i += 1;

    // 1 constraint, degree 2: push_flag cannot be 1 for two consecutive operations
    table[i][step] = mul(current.get_push_flag(), next.get_push_flag());
    i += 1;

    // 1 constraint, degree 2: when push_flag = 0, op_bits must be a binary decomposition
    // of op_code, otherwise all op_bits must be 0 (NOOP)
    let op_bits_value = current.get_op_bits_value();
    let op_code = mul(current.get_op_code(), binary_not(current.get_push_flag()));
    table[i][step] = sub(op_code, op_bits_value);
    i += 1;

    debug_assert!(CONSTRAINT_DEGREES.len() == i, "number of decoder constraints is invalid");
}

// HELPER FUNCTIONS
// ================================================================================================
fn is_binary(v: u64) -> u64 {
    return sub(mul(v, v), v);
}

fn binary_not(v: u64) -> u64 {
    return sub(ONE, v);
}
