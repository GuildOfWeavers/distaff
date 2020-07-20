use crate::utils::hasher::{ apply_sbox, apply_mds, apply_inv_mds };
use super::{
    field, are_equal, EvaluationResult,
    HASH_STATE_WIDTH
};

/// Evaluates constraints for a single round of a modified Rescue hash function. Hash state is
/// assumed to be in the first 6 registers of user stack; the rest of the stack does not change.
pub fn enforce_rescr(result: &mut [u128], current: &[u128], next: &[u128], ark: &[u128], op_flag: u128)
{
    // evaluate the first half of Rescue round
    let mut old_state = [field::ZERO; HASH_STATE_WIDTH];
    old_state.copy_from_slice(&current[..HASH_STATE_WIDTH]);
    for i in 0..HASH_STATE_WIDTH {
        old_state[i] = field::add(old_state[i], ark[i]);
    }
    apply_sbox(&mut old_state);
    apply_mds(&mut old_state);

    // evaluate inverse of the second half of Rescue round
    let mut new_state = [field::ZERO; HASH_STATE_WIDTH];
    new_state.copy_from_slice(&next[..HASH_STATE_WIDTH]);
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
    for i in HASH_STATE_WIDTH..result.len() {
        result.agg_constraint(i, op_flag, are_equal(next[i], current[i]));
    }
}