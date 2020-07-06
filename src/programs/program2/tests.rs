use crate::{ opcodes };
use super::{ Program, ProgramBlock, ExecutionHint, Span, Group, hash_op };
use super::hashing::{ ACC_NUM_ROUNDS, CYCLE_LENGTH, STATE_WIDTH };
use crate::utils::accumulator::{ add_constants, apply_sbox, apply_mds, apply_inv_sbox };

// TESTS
// ================================================================================================

#[test]
fn traverse_linear_path() {
    let block = Span::new_block(vec![opcodes::NOOP; 16]);

    let program = Program::new(vec![block]);
    let (step, hash) = traverse_true_branch(program.body(), 0, 0, 0);

    assert_eq!(program.hash(), hash);
    assert_eq!(31, step);
}

#[test]
fn traverse_linear_blocks() {
    let block1 = Span::new_block(vec![opcodes::NOOP; 15]);

    let inner_block1 = Span::new_block(vec![opcodes::ADD; 16]);
    let block2 = Group::new_block(vec![inner_block1]);

    let inner_block2 = Span::new_block(vec![opcodes::MUL; 16]);
    let block3 = Group::new_block(vec![inner_block2]);

    // sequence of blocks ending with group block
    let program = Program::new(vec![block1.clone(), block2.clone(), block3.clone()]);
    let (step, hash) = traverse_true_branch(program.body(), 0, 0, 0);

    assert_eq!(program.hash(), hash);
    assert_eq!(95, step);

    // sequence of blocks ending with span block
    let block4 = Span::new_block(vec![opcodes::INV; 16]);

    let program = Program::new(vec![block1, block2, block3, block4]);
    let (step, hash) = traverse_true_branch(program.body(), 0, 0, 0);

    assert_eq!(program.hash(), hash);
    assert_eq!(111, step);
}

#[test]
fn traverse_nested_blocks() {
    let block1 = Span::new_block(vec![opcodes::NOOP; 15]);

    let inner_block1 = Span::new_block(vec![opcodes::ADD; 16]);
    let block2 = Group::new_block(vec![inner_block1]);

    let inner_block2 = Span::new_block(vec![opcodes::MUL; 15]);
    let inner_inner_block1 = Span::new_block(vec![opcodes::INV; 16]);
    let inner_block3 = Group::new_block(vec![inner_inner_block1]);
    let block3 = Group::new_block(vec![inner_block2, inner_block3]);

    // sequence of blocks ending with group block
    let program = Program::new(vec![block1, block2, block3]);
    let (step, hash) = traverse_true_branch(program.body(), 0, 0, 0);

    assert_eq!(program.hash(), hash);
    assert_eq!(127, step);
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
            println!("{}: BEGIN {:?}", step, hash);
            step += 1; // BEGIN
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
        if i != 0 && blocks[i].is_span() {
            println!("{}: {}!\t {:?}", step, opcodes::NOOP, state);
            step += 1;
        }
        step = traverse(&blocks[i], &mut state, step);
    }

    if !blocks.last().unwrap().is_span() {
        println!("{}: {}!!\t {:?}", step, opcodes::NOOP, state);
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

fn traverse_false_branch(blocks: &[ProgramBlock], parent_hash: u128, sibling_hash: u128, mut step: usize) -> (usize, [u128; 4]) {
    
    let mut state = [0, 0, 0, 0];
    for i in 0..blocks.len() {
        step = traverse(&blocks[i], &mut state, step);
    }

    if !blocks.last().unwrap().is_span() {
        println!("{}: {} \t {:?}", step, opcodes::NOOP, state);
        hash_op(&mut state, opcodes::NOOP, 0, step);
        step += 1;
    }

    println!("{}: FEND {:?}", step, state);
    step += 1; // TEND

    state = [parent_hash, state[0], sibling_hash, 0];
    for _ in 0..ACC_NUM_ROUNDS {
        println!("{}: HACC {:?}", step, state);
        op_hacc(&mut state, step);
        step += 1;
    }

    return (step, state);
}