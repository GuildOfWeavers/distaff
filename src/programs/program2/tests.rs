use crate::{ opcodes };
use super::{ Program, ProgramBlock, ExecutionHint, Span, Group, hash_op };
use super::hashing::{ ACC_NUM_ROUNDS, CYCLE_LENGTH, STATE_WIDTH };
use crate::utils::accumulator::{ add_constants, apply_sbox, apply_mds, apply_inv_sbox };

// TESTS
// ================================================================================================

#[test]
fn traverse_linear_path() {
    let block = Span::from_instructions(vec![
        opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
        opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
        opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
        opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
    ]);

    let program = Program::new(vec![ProgramBlock::Span(block)]);

    let (step, hash) = traverse_true_branch(program.body(), 0, 0, 0);

    assert_eq!(program.hash(), hash);
}

#[test]
fn traverse_linear_blocks() {
    let block1 = ProgramBlock::Span(Span::from_instructions(vec![
        opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
        opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
        opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
        opcodes::NOOP, opcodes::NOOP, opcodes::NOOP
    ]));

    let block2 = ProgramBlock::Span(Span::from_instructions(vec![
        opcodes::ADD,  opcodes::ADD,  opcodes::ADD,  opcodes::ADD,
        opcodes::ADD,  opcodes::ADD,  opcodes::ADD,  opcodes::ADD,
        opcodes::ADD,  opcodes::ADD,  opcodes::ADD,  opcodes::ADD,
        opcodes::ADD,  opcodes::ADD,  opcodes::ADD,  opcodes::ADD,
    ]));

    let block3 = ProgramBlock::Group(Group::new(vec![block2]));

    let program = Program::new(vec![block1, block3]);

    let (step, hash) = traverse_true_branch(program.body(), 0, 0, 0);

    assert_eq!(program.hash(), hash);
    assert_eq!(0, 1);
}

// HELPER FUNCTIONS
// ================================================================================================
fn traverse(block: &ProgramBlock, hash: &mut [u128; 4], mut step: usize) -> usize {

    match block {
        ProgramBlock::Span(block)   => {
            for i in 0..block.length() {
                let (op_code, op_hint) = block.get_op(i);
                let op_value = match op_hint {
                    ExecutionHint::PushValue(value) => value,
                    _ => 0,
                };
                println!("{}: {} \t {:?}", step, op_code, hash);
                hash_op(hash, op_code, op_value, step);
                step += 1;
            }
        },
        ProgramBlock::Group(block)  => {
            println!("{}: BEGIN {:?}", step, hash);
            step += 1; // BEGIN
            let (new_step, state) = traverse_true_branch(block.blocks(), hash[0], 0, step);
            hash.copy_from_slice(&state);
            step = new_step;
        },
        ProgramBlock::Switch(block) => {
            
        },
        ProgramBlock::Loop(block)   => {
            
        },
    }

    return step;
}

fn op_hacc(state: &mut [u128; 4], step: usize) {

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

fn traverse_true_branch(blocks: &[ProgramBlock], parent_hash: u128, sibling_hash: u128, mut step: usize) -> (usize, [u128; 4]) {
    
    let mut state = [0, 0, 0, 0];
    for i in 0..blocks.len() {
        step = traverse(&blocks[i], &mut state, step);
    }

    if !blocks.last().unwrap().is_span() {
        println!("{}: {} \t {:?}", step, opcodes::NOOP, state);
        hash_op(&mut state, opcodes::NOOP, 0, step);
        step += 1;
    }

    println!("{}: TEND {:?}", step, state);
    step += 1; // TEND

    state = [parent_hash, state[0], sibling_hash, 0];
    for _ in 0..ACC_NUM_ROUNDS {
        println!("{}: HACC {:?}", step, state);
        op_hacc(&mut state, step);
        step += 1;
    }

    return (step, state);
}