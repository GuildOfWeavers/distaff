use crate::math::field::{ sub, add, mul };
use crate::stark::{ TraceState, MAX_CONSTRAINT_DEGREE };
use crate::stark::utils::hash_acc::{
    apply_mds, apply_sbox, apply_inv_mds, STATE_WIDTH, NUM_ROUNDS, ARK_EX
};

// CONSTANTS
// ================================================================================================

/// Degree of hash accumulator constraints.
pub const CONSTRAINT_DEGREES: [usize; STATE_WIDTH] = [6; STATE_WIDTH];

const NUM_ROUNDS_EX: usize = MAX_CONSTRAINT_DEGREE * NUM_ROUNDS;

// EVALUATOR FUNCTION
// ================================================================================================
pub fn evaluate(current: &TraceState, next: &TraceState, step: usize) -> [u64; STATE_WIDTH] {

    let op_code = current.get_op_code();
    let mut current_acc = [0; STATE_WIDTH];
    current_acc.copy_from_slice(current.get_op_acc());
    let mut next_acc = [0; STATE_WIDTH];
    next_acc.copy_from_slice(next.get_op_acc());

    current_acc[0] = add(current_acc[0], op_code);
    current_acc[1] = mul(current_acc[1], op_code);
    add_constants(&mut current_acc, step % NUM_ROUNDS_EX, 0);
    apply_sbox(&mut current_acc);
    apply_mds(&mut current_acc);

    apply_inv_mds(&mut next_acc);
    apply_sbox(&mut next_acc);
    sub_constants(&mut next_acc, step % NUM_ROUNDS_EX, STATE_WIDTH);

    for i in 0..STATE_WIDTH {
        next_acc[i] = sub(next_acc[i], current_acc[i]);
    }

    return next_acc;
}

// HELPER FUNCTIONS
// ================================================================================================
pub fn add_constants(state: &mut[u64; STATE_WIDTH], step: usize, offset: usize) {
    for i in 0..STATE_WIDTH {
        state[i] = add(state[i], ARK_EX[offset + i][step]);
    }
}

pub fn sub_constants(state: &mut[u64; STATE_WIDTH], step: usize, offset: usize) {
    for i in 0..STATE_WIDTH {
        state[i] = sub(state[i], ARK_EX[offset + i][step]);
    }
}