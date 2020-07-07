use super::{ ProgramBlock, ExecutionHint };
use super::super::hashing::{ hash_op, acc_hash_round, ACC_NUM_ROUNDS };

pub fn traverse(block: &ProgramBlock, stack: &mut Vec<u128>, hash: &mut [u128; 4], mut step: usize) -> usize {

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
            let (new_step, state) = traverse_true_branch(block.body(), stack, hash[0], 0, step);
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

pub fn traverse_true_branch(blocks: &[ProgramBlock], stack: &mut Vec<u128>, parent_hash: u128, sibling_hash: u128, mut step: usize) -> (usize, [u128; 4]) {
    
    let mut state = [0, 0, 0, 0];
    for i in 0..blocks.len() {
        if i != 0 && blocks[i].is_span() {
            println!("{}: HACC {:?}", step, state);
            acc_hash_round(&mut state, step);
            step += 1;
        }
        step = traverse(&blocks[i], stack, &mut state, step);
    }

    println!("{}: HACC {:?}", step, state);
    acc_hash_round(&mut state, step);
    step += 1;

    println!("{}: TEND {:?}", step, state);
    step += 1; // TEND

    state = [parent_hash, state[0], sibling_hash, 0];
    for _ in 0..ACC_NUM_ROUNDS {
        println!("{}: HACC {:?}", step, state);
        acc_hash_round(&mut state, step);
        step += 1;
    }

    return (step, state);
}

fn traverse_false_branch(blocks: &[ProgramBlock], stack: &mut Vec<u128>, parent_hash: u128, sibling_hash: u128, mut step: usize) -> (usize, [u128; 4]) {
    
    let mut state = [0, 0, 0, 0];
    for i in 0..blocks.len() {
        if i != 0 && blocks[i].is_span() {
            println!("{}: HACC {:?}", step, state);
            acc_hash_round(&mut state, step);
            step += 1;
        }
        step = traverse(&blocks[i], stack, &mut state, step);
    }

    println!("{}: HACC {:?}", step, state);
    acc_hash_round(&mut state, step);
    step += 1;

    println!("{}: FEND {:?}", step, state);
    step += 1; // FEND

    state = [parent_hash, sibling_hash, state[0], 0];
    for _ in 0..ACC_NUM_ROUNDS {
        println!("{}: HACC {:?}", step, state);
        acc_hash_round(&mut state, step);
        step += 1;
    }

    return (step, state);
}

fn traverse_loop_body(blocks: &[ProgramBlock], stack: &mut Vec<u128>, parent_hash: u128, body_hash: u128, skip_hash: u128, mut step: usize) -> (usize, [u128; 4]) {
    
    let mut state = [0, 0, 0, 0];
    loop {

        for i in 0..blocks.len() {
            if i != 0 && blocks[i].is_span() {
                println!("{}: HACC {:?}", step, state);
                acc_hash_round(&mut state, step);
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
        acc_hash_round(&mut state, step);
        step += 1;
    }

    return (step, state);
}