use crate::{ opcodes };
use super::{ Program, ProgramBlock, ExecutionHint, Span, Group, Switch, Loop, hash_op };
use super::hashing::{ ACC_NUM_ROUNDS, CYCLE_LENGTH, STATE_WIDTH };
use crate::utils::accumulator::{ add_constants, apply_sbox, apply_mds, apply_inv_sbox };

// TESTS
// ================================================================================================

#[test]
fn single_block() {
    let block = Span::new_block(vec![opcodes::NOOP; 15]);

    let program = Program::new(vec![block]);
    let (step, hash) = traverse_true_branch(program.body(), &mut vec![], 0, 0, 0);

    assert_eq!(program.hash(), hash);
    assert_eq!(31, step);
}

#[test]
fn linear_blocks() {
    let block1 = Span::new_block(vec![opcodes::NOOP; 15]);

    let inner_block1 = Span::new_block(vec![opcodes::ADD; 15]);
    let block2 = Group::new_block(vec![inner_block1]);

    let inner_block2 = Span::new_block(vec![opcodes::MUL; 15]);
    let block3 = Group::new_block(vec![inner_block2]);

    // sequence of blocks ending with group block
    let program = Program::new(vec![block1.clone(), block2.clone(), block3.clone()]);
    let (step, hash) = traverse_true_branch(program.body(), &mut vec![], 0, 0, 0);

    assert_eq!(program.hash(), hash);
    assert_eq!(95, step);

    // sequence of blocks ending with span block
    let block4 = Span::new_block(vec![opcodes::INV; 15]);

    let program = Program::new(vec![block1, block2, block3, block4]);
    let (step, hash) = traverse_true_branch(program.body(), &mut vec![], 0, 0, 0);

    assert_eq!(program.hash(), hash);
    assert_eq!(111, step);
}

#[test]
fn nested_blocks() {
    let block1 = Span::new_block(vec![opcodes::NOOP; 15]);

    let inner_block1 = Span::new_block(vec![opcodes::ADD; 15]);
    let block2 = Group::new_block(vec![inner_block1]);

    let inner_block2 = Span::new_block(vec![opcodes::MUL; 15]);
    let inner_inner_block1 = Span::new_block(vec![opcodes::INV; 15]);
    let inner_block3 = Group::new_block(vec![inner_inner_block1]);
    let block3 = Group::new_block(vec![inner_block2, inner_block3]);

    // sequence of blocks ending with group block
    let program = Program::new(vec![block1, block2, block3]);
    let (step, hash) = traverse_true_branch(program.body(), &mut vec![], 0, 0, 0);

    assert_eq!(program.hash(), hash);
    assert_eq!(127, step);
}

#[test]
fn conditional_program() {
    let block1 = Span::new_block(vec![opcodes::NOOP; 15]);

    let t_branch = vec![Span::new_block(vec![
        opcodes::ASSERT, opcodes::ADD, opcodes::ADD, opcodes::ADD,
        opcodes::ADD,    opcodes::ADD, opcodes::ADD, opcodes::ADD,
        opcodes::ADD,    opcodes::ADD, opcodes::ADD, opcodes::ADD,
        opcodes::ADD,    opcodes::ADD, opcodes::ADD,
    ])];
    let f_branch = vec![Span::new_block(vec![
        opcodes::NOT, opcodes::ASSERT, opcodes::MUL, opcodes::MUL,
        opcodes::MUL, opcodes::MUL,    opcodes::MUL, opcodes::MUL,
        opcodes::MUL, opcodes::MUL,    opcodes::MUL, opcodes::MUL,
        opcodes::MUL, opcodes::MUL,    opcodes::MUL,
    ])];
    let block2 = Switch::new_block(t_branch, f_branch);
    
    let program = Program::new(vec![block1, block2]);

    // true branch execution
    let (step, hash) = traverse_true_branch(program.body(), &mut vec![1], 0, 0, 0);
    assert_eq!(program.hash(), hash);
    assert_eq!(63, step);

    // false branch execution
    let (step, hash) = traverse_true_branch(program.body(), &mut vec![0], 0, 0, 0);
    assert_eq!(program.hash(), hash);
    assert_eq!(63, step);
}

#[test]
fn simple_loop() {
    let block1 = Span::new_block(vec![opcodes::NOOP; 15]);

    let loop_body = vec![Span::new_block(vec![
        opcodes::ASSERT, opcodes::ADD, opcodes::ADD, opcodes::ADD,
        opcodes::ADD,    opcodes::ADD, opcodes::ADD, opcodes::ADD,
        opcodes::ADD,    opcodes::ADD, opcodes::ADD, opcodes::ADD,
        opcodes::ADD,    opcodes::ADD, opcodes::ADD,
    ])];
    let block2 = Loop::new_block(loop_body);
    
    let program = Program::new(vec![block1, block2]);

    // loop not entered
    let (step, hash) = traverse_true_branch(program.body(), &mut vec![0], 0, 0, 0);
    assert_eq!(program.hash(), hash);
    assert_eq!(63, step);

    // loop executed once
    let (step, hash) = traverse_true_branch(program.body(), &mut vec![0, 1], 0, 0, 0);
    assert_eq!(program.hash(), hash);
    assert_eq!(63, step);

    // loop executed 3 times
    let (step, hash) = traverse_true_branch(program.body(), &mut vec![0, 1, 1, 1], 0, 0, 0);
    assert_eq!(program.hash(), hash);
    assert_eq!(95, step);
}

// HELPER FUNCTIONS
// ================================================================================================
fn traverse(block: &ProgramBlock, stack: &mut Vec<u128>, hash: &mut [u128; 4], mut step: usize) -> usize {

    match block {
        ProgramBlock::Span(block) => {
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
        ProgramBlock::Group(block) => {
            println!("{}: BEGIN {:?}", step, hash);
            step += 1; // BEGIN
            let (new_step, state) = traverse_true_branch(block.blocks(), stack, hash[0], 0, step);
            hash.copy_from_slice(&state);
            step = new_step;
        },
        ProgramBlock::Switch(block) => {
            println!("{}: BEGIN {:?}", step, hash);
            step += 1; // BEGIN

            let condition = stack.pop().unwrap();
            match condition {
                0 => {
                    let blocks = block.false_branch();
                    let sibling_hash = block.true_branch_hash();
                    let (new_step, state) = traverse_false_branch(blocks, stack, hash[0], sibling_hash, step);
                    hash.copy_from_slice(&state);
                    step = new_step;
                },
                1 => {
                    let blocks = block.true_branch();
                    let sibling_hash = block.false_branch_hash();
                    let (new_step, state) = traverse_true_branch(blocks, stack, hash[0], sibling_hash, step);
                    hash.copy_from_slice(&state);
                    step = new_step;
                },
                _ => panic!("cannot select a branch based on a non-binary condition {}", condition)
            };
        },
        ProgramBlock::Loop(block) => {
            let condition = stack.pop().unwrap();
            match condition {
                0 => {
                    println!("{}: BEGIN {:?}", step, hash);
                    step += 1; // BEGIN

                    let blocks = block.skip();
                    let body_hash = block.body_hash();
                    let (new_step, state) = traverse_false_branch(blocks, stack, hash[0], body_hash, step);
                    hash.copy_from_slice(&state);
                    step = new_step;
                },
                1 => {
                    println!("{}: LOOP {:?}", step, hash);
                    step += 1; // LOOP

                    let body = block.body();
                    let body_hash = block.body_hash();
                    let skip_hash = block.skip_hash();
                    let (new_step, state) = traverse_loop_body(body, stack, hash[0], body_hash, skip_hash, step);
                    hash.copy_from_slice(&state);
                    step = new_step;
                },
                _ => panic!("cannot enter loop based on a non-binary condition {}", condition)
            };
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

fn traverse_true_branch(blocks: &[ProgramBlock], stack: &mut Vec<u128>, parent_hash: u128, sibling_hash: u128, mut step: usize) -> (usize, [u128; 4]) {
    
    let mut state = [0, 0, 0, 0];
    for i in 0..blocks.len() {
        if i != 0 && blocks[i].is_span() {
            println!("{}: SKIP {:?}", step, state);
            step += 1;
        }
        step = traverse(&blocks[i], stack, &mut state, step);
    }

    println!("{}: SKIP {:?}", step, state);
    step += 1;

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

fn traverse_false_branch(blocks: &[ProgramBlock], stack: &mut Vec<u128>, parent_hash: u128, sibling_hash: u128, mut step: usize) -> (usize, [u128; 4]) {
    
    let mut state = [0, 0, 0, 0];
    for i in 0..blocks.len() {
        if i != 0 && blocks[i].is_span() {
            println!("{}: {}!\t {:?}", step, opcodes::NOOP, state);
            step += 1;
        }
        step = traverse(&blocks[i], stack, &mut state, step);
    }

    println!("{}: SKIP {:?}", step, state);
    step += 1;

    println!("{}: FEND {:?}", step, state);
    step += 1; // FEND

    state = [parent_hash, sibling_hash, state[0], 0];
    for _ in 0..ACC_NUM_ROUNDS {
        println!("{}: HACC {:?}", step, state);
        op_hacc(&mut state, step);
        step += 1;
    }

    return (step, state);
}

fn traverse_loop_body(blocks: &[ProgramBlock], stack: &mut Vec<u128>, parent_hash: u128, body_hash: u128, skip_hash: u128, mut step: usize) -> (usize, [u128; 4]) {
    
    let mut state = [0, 0, 0, 0];
    loop {

        for i in 0..blocks.len() {
            if i != 0 && blocks[i].is_span() {
                println!("{}: {}!\t {:?}", step, opcodes::NOOP, state);
                step += 1;
            }
            step = traverse(&blocks[i], stack, &mut state, step);
        }

        assert!(state[0] == body_hash, "loop image didn't match loop body hash");

        println!("{}: WRAP {:?}", step, state);
        step += 1; // WRAP

        let condition = stack.pop().unwrap();
        match condition {
            0 => break,
            1 => state = [0, 0, 0, 0],
            _ => panic!("cannot exit loop based on a non-binary condition {}", condition)
        };
    }

    println!("{}: TEND {:?}", step, state);
    step += 1; // TEND

    state = [parent_hash, body_hash, skip_hash, 0];
    for _ in 0..ACC_NUM_ROUNDS {
        println!("{}: HACC {:?}", step, state);
        op_hacc(&mut state, step);
        step += 1;
    }

    return (step, state);
}