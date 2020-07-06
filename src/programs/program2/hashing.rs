use crate::math::field;
use crate::utils::accumulator::{ add_constants, apply_sbox, apply_mds, apply_inv_sbox };
use crate::{ ACC_STATE_WIDTH };
use super::{ ProgramBlock, BASE_CYCLE_LENGTH };

pub const STATE_WIDTH: usize = ACC_STATE_WIDTH;
pub const CYCLE_LENGTH: usize = BASE_CYCLE_LENGTH;
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
        acc_hash_round(&mut state, i);
    }
    return state;
}

pub fn hash_seq(blocks: &Vec<ProgramBlock>, is_loop_body: bool) -> u128 {

    let mut state = [0u128; STATE_WIDTH];
    state = blocks[0].hash(state);
    
    for i in 1..blocks.len() {
        let block = &blocks[i];
        if block.is_span() {
            acc_hash_round(&mut state, CYCLE_LENGTH - 1);    
        }
        state = block.hash(state);
    }

    if !is_loop_body {
        acc_hash_round(&mut state, CYCLE_LENGTH - 1);
    }

    return state[0];
}

pub fn acc_hash_round(state: &mut [u128; STATE_WIDTH], step: usize) {
    
    let ark_idx = step % CYCLE_LENGTH;

    // apply first half of Rescue round
    add_constants(state, ark_idx, 0);
    apply_sbox(state);
    apply_mds(state);

    // apply second half of Rescue round
    add_constants(state, ark_idx, STATE_WIDTH);
    apply_inv_sbox(state);
    apply_mds(state);
}