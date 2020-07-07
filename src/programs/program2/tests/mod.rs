use crate::{ opcodes };
use super::{ Program, ProgramBlock, ExecutionHint, Span, Group, Switch, Loop };

mod utils;
use utils::{ traverse_true_branch };

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