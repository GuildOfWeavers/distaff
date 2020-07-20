use crate::utils::hasher::{ apply_sbox, apply_mds, apply_inv_mds };
use super::{
    field, are_equal, EvaluationResult, enforce_stack_copy,
    HASH_STATE_WIDTH
};

/// Evaluates constraints for a single round of a modified Rescue hash function. Hash state is
/// assumed to be in the first 6 registers of user stack; the rest of the stack does not change.
pub fn enforce_rescr(result: &mut [u128], old_stack: &[u128], new_stack: &[u128], ark: &[u128], op_flag: u128)
{
    // evaluate the first half of Rescue round
    let mut old_state = [field::ZERO; HASH_STATE_WIDTH];
    old_state.copy_from_slice(&old_stack[..HASH_STATE_WIDTH]);
    for i in 0..HASH_STATE_WIDTH {
        old_state[i] = field::add(old_state[i], ark[i]);
    }
    apply_sbox(&mut old_state);
    apply_mds(&mut old_state);

    // evaluate inverse of the second half of Rescue round
    let mut new_state = [field::ZERO; HASH_STATE_WIDTH];
    new_state.copy_from_slice(&new_stack[..HASH_STATE_WIDTH]);
    apply_inv_mds(&mut new_state);
    apply_sbox(&mut new_state);
    for i in 0..HASH_STATE_WIDTH {
        new_state[i] = field::sub(new_state[i], ark[HASH_STATE_WIDTH + i]);
    }

    // compar the results of both rounds
    for i in 0..HASH_STATE_WIDTH {
        result.agg_constraint(i, op_flag, are_equal(new_state[i], old_state[i]));
    }

    // make sure the rest of the stack didn't change
    enforce_stack_copy(result, old_stack, new_stack, HASH_STATE_WIDTH, op_flag);
}