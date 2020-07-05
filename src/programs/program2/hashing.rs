use crate::math::field;
use crate::utils::accumulator::{ add_constants, apply_sbox, apply_mds, apply_inv_sbox };
use crate::{ opcodes, ACC_STATE_WIDTH };
use super::{ ProgramBlock };

pub const STATE_WIDTH: usize = ACC_STATE_WIDTH;
pub const CYCLE_LENGTH: usize = 16;
pub const ACC_NUM_ROUNDS: usize = 14;
pub const ACC_ROUND_OFFSET: usize = 1;

pub fn hash_op(state: &mut [u128; STATE_WIDTH], op_code: u8, op_value: u128, step: usize) {

    let ark_idx = step % CYCLE_LENGTH;

    // apply first half of Rescue round
    add_constants(state, ark_idx, 0);
    apply_sbox(state);
    apply_mds(state);

    // inject value into the state
    state[0] = field::add(state[0], op_code as u128);
    state[1] = field::mul(state[1], op_value);

    // apply second half of Rescue round
    add_constants(state, ark_idx, STATE_WIDTH);
    apply_inv_sbox(state);
    apply_mds(state);
}

pub fn hash_acc(h: u128, v0: u128, v1: u128) -> [u128; STATE_WIDTH] {
    let mut state = [h, v0, v1, 0];
    for i in ACC_ROUND_OFFSET..(ACC_ROUND_OFFSET + ACC_NUM_ROUNDS) {
        // apply first half of Rescue round
        add_constants(&mut state, i, 0);
        apply_sbox(&mut state);
        apply_mds(&mut state);

        // apply second half of Rescue round
        add_constants(&mut state, i, STATE_WIDTH);
        apply_inv_sbox(&mut state);
        apply_mds(&mut state);
    }
    return state;
}

pub fn hash_seq(blocks: &Vec<ProgramBlock>) -> u128 {

    let mut state = [0u128; STATE_WIDTH];
    for block in blocks {
        state = block.hash(state);
    }

    if !blocks.last().unwrap().is_span() {
        hash_op(&mut state, opcodes::NOOP, 0, 15);
    }

    return state[0];
}