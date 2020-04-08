use crate::math::field::{ add, sub, mul, ONE };
use crate::trace::{ TraceState, opcodes };

// CONSTANTS
// ================================================================================================
pub const CONSTRAINT_DEGREES: [usize; 9] = [
    2, 2, 2, 2, 2,  // op_bits are binary
    2,              // push_flag is binary
    5,              // push_flag is set after a PUSH operation
    2,              // push_flag gets reset on the next step
    3,              // when push_flag = 0, op_bits are a binary decomposition of op_code
];

// EVALUATOR FUNCTION
// ================================================================================================
pub fn evaluate(current: &TraceState, next: &TraceState, op_flags: &[u64; 32], table: &mut Vec<Vec<u64>>, step: usize) {

    // constraint counter
    let mut i = 0;

    // 5 constraints, degree 2: op_bits must be binary
    for _ in 0..5 {
        table[i][step] = is_binary(current.op_bits[i]);
        i += 1;
    }

    // 1 constraint, degree 2: push_flag must be binary
    table[i][step] = is_binary(current.push_flag);
    i += 1;

    // 1 constraint, degree 5: push_flag must be set to 1 after a PUSH operation
    table[i][step] = sub(op_flags[opcodes::PUSH as usize], next.push_flag);
    i += 1;

    // 1 constraint, degree 2: push_flag cannot be 1 for two consecutive operations
    table[i][step] = mul(current.push_flag, next.push_flag);
    i += 1;

    // 1 constraint, degree 3: when push_flag = 0, op_bits must be a binary decomposition
    // of op_code, otherwise all op_bits must be 0 (NOOP)
    let op_bits_value = binary_composition5(&current.op_bits);
    let op_code = mul(current.op_code, binary_not(current.push_flag));
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

fn binary_composition5(v: &[u64; 5]) -> u64 {
    let mut result = v[0];
    result = add(result, mul(v[1],  2));
    result = add(result, mul(v[2],  4));
    result = add(result, mul(v[3],  8));
    result = add(result, mul(v[4], 16));
    return result;
}