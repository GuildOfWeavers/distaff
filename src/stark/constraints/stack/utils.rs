use super::{ are_equal, EvaluationResult };

// TODO: replace with explicit stack shift operations
#[inline(always)]
pub fn enforce_no_change(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    for i in 0..result.len() {
        result.agg_constraint(i, op_flag, are_equal(next[i], current[i]));
    }
}