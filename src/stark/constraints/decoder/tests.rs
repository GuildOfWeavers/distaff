use crate::utils::sponge::{ apply_round as apply_hacc_round };
use super::{ Decoder, TraceState, UserOps };

// CONSTANTS
// ================================================================================================
const TRACE_LENGTH: usize = 16;
const EXTENSION_FACTOR: usize = 8;

// BEGIN OPERATION
// ================================================================================================
#[test]
fn enforce_begin() {

    let decoder = new_decoder(1, 0);
    let step = 15 * EXTENSION_FACTOR;
    let success_result = vec![0; decoder.constraint_degrees().len()];
    
    // correct transition
    let evaluations = evaluate_transition(&decoder, step,
        vec![0, 3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  0,  11],
        vec![0, 0, 0, 0, 0,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  3,  11]);
    assert_eq!(success_result, evaluations);
    
    // incorrect transition, wrong opcode
    let evaluations = evaluate_transition(&decoder, step,
        vec![0, 3, 5, 7, 9,  1, 1, 0,  1, 1, 1, 1, 1,  1, 1,  0,  11],
        vec![0, 0, 0, 0, 0,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  3,  11]);
    assert_ne!(success_result, evaluations);

    // incorrect transition, context stack not updated
    let evaluations = evaluate_transition(&decoder, step,
        vec![0, 3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  0,  11],
        vec![0, 0, 0, 0, 0,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  0,  11]);
    assert_ne!(success_result, evaluations);

    // incorrect transition, stack updated to wrong value
    let evaluations = evaluate_transition(&decoder, step,
        vec![0, 3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  0,  11],
        vec![0, 0, 0, 0, 0,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  5,  11]);
    assert_ne!(success_result, evaluations);

    // incorrect transition, sponge not cleared
    let evaluations = evaluate_transition(&decoder, step,
        vec![0, 3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  0,  11],
        vec![0, 3, 5, 7, 9,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  3,  11]);
    assert_ne!(success_result, evaluations);
}

// HACC OPERATION
// ================================================================================================
#[test]
fn enforce_hacc() {
    let decoder = new_decoder(1, 0);
    let success_result = vec![0; decoder.constraint_degrees().len()];

    // correct transition, push.9, step = 0
    let push_value = 9;
    let state1     = vec![1,  3, 5, 7, 9,  0, 0, 0,  1, 1, 1, 1, 1,  0, 0,  0,  11];
    let mut state2 = vec![2,  3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  0,  push_value];
    apply_hacc_round(&mut state2[1..5], UserOps::Push as u128, push_value, 0);
    let evaluations = evaluate_transition(&decoder, 0, state1, state2);
    assert_eq!(success_result, evaluations);

    // correct transition, push.9, step = 8 (extension = 8)
    let push_value = 9;
    let state1     = vec![1,  3, 5, 7, 9,  0, 0, 0,  1, 1, 1, 1, 1,  0, 0,  0,  11];
    let mut state2 = vec![2,  3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  0,  push_value];
    apply_hacc_round(&mut state2[1..5], UserOps::Push as u128, push_value, 8);
    let evaluations = evaluate_transition(&decoder, 8 * EXTENSION_FACTOR, state1, state2);
    assert_eq!(success_result, evaluations);

    // correct transition, add, step = 0
    let state1     = vec![1,  3, 5, 7, 9,  0, 0, 0,  0, 0, 0, 1, 0,  1, 1,  0,  0];
    let mut state2 = vec![2,  3, 5, 7, 9,  0, 0, 0,  1, 1, 1, 1, 1,  1, 1,  0,  0];
    apply_hacc_round(&mut state2[1..5], UserOps::Add as u128, 0, 0);
    let evaluations = evaluate_transition(&decoder, 0, state1, state2);
    assert_eq!(success_result, evaluations);

    // incorrect transition (wrong stack value), push.9, step = 0
    let push_value = 9;
    let state1     = vec![1,  3, 5, 7, 9,  0, 0, 0,  1, 1, 1, 1, 1,  0, 0,  0,  11];
    let mut state2 = vec![2,  3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  0,  11];
    apply_hacc_round(&mut state2[1..5], UserOps::Push as u128, push_value, 0);
    let evaluations = evaluate_transition(&decoder, 0, state1, state2);
    assert_ne!(success_result, evaluations);

    // incorrect transition (wrong opcode), push.9, step = 0
    let push_value = 9;
    let state1     = vec![1,  3, 5, 7, 9,  0, 0, 0,  1, 1, 1, 1, 1,  1, 1,  0,  11];
    let mut state2 = vec![2,  3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  0,   9];
    apply_hacc_round(&mut state2[1..5], UserOps::Push as u128, push_value, 0);
    let evaluations = evaluate_transition(&decoder, 0, state1, state2);
    assert_ne!(success_result, evaluations);

    // incorrect transition (stack value added to sponge), add, step = 0
    let state1     = vec![1,  3, 5, 7, 9,  0, 0, 0,  0, 0, 0, 1, 0,  1, 1,  0,  9];
    let mut state2 = vec![2,  3, 5, 7, 9,  0, 0, 0,  1, 1, 1, 1, 1,  1, 1,  0,  0];
    apply_hacc_round(&mut state2[1..5], UserOps::Add as u128, 9, 0);
    let evaluations = evaluate_transition(&decoder, 0, state1, state2);
    assert_ne!(success_result, evaluations);
}

// HELPER FUNCTIONS
// ================================================================================================
fn new_decoder(ctx_depth: usize, loop_depth: usize) -> Decoder {
    return Decoder::new(TRACE_LENGTH, EXTENSION_FACTOR, ctx_depth, loop_depth);
}

fn evaluate_transition(decoder: &Decoder, step: usize, state1: Vec<u128>, state2: Vec<u128>) -> Vec<u128>
{
    let state1 = TraceState::from_vec(decoder.ctx_depth(), decoder.loop_depth(), 1, &state1);
    let state2 = TraceState::from_vec(decoder.ctx_depth(), decoder.loop_depth(), 1, &state2);

    let mut evaluations = vec![0; decoder.constraint_degrees().len()];
    decoder.evaluate(&state1, &state2, step, &mut evaluations);
    return evaluations
}