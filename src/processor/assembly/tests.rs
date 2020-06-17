use crate::crypto::hash::blake3;
use super::{ Program, ExecutionGraph, opcodes };

#[test]
fn linear_assembly() {
    let source = "push.1 push.2 add";
    let program = super::compile(source).unwrap();

    let expected_program = Program::from_path(vec![
        opcodes::BEGIN, opcodes::PUSH, 1, opcodes::PUSH, 2, opcodes::ADD
    ]);

    assert_eq!(expected_program.hash(), program.hash());
}

#[test]
fn branching_assembly() {
    let source = "
        push.3
        push.5
        read
        if.true
            add
        else
            mul
        endif";
    let program = super::compile(source).unwrap();

    let mut root = ExecutionGraph::new(vec![opcodes::BEGIN, opcodes::PUSH, 3, opcodes::PUSH, 5, opcodes::READ]);
    let tb = ExecutionGraph::new(vec![opcodes::ASSERT, opcodes::ADD]);
    let fb = ExecutionGraph::new(vec![opcodes::NOT, opcodes::ASSERT, opcodes::MUL]);
    root.set_next(tb, fb);

    let expected_program = Program::new(root, blake3);
    assert_eq!(expected_program.hash(), program.hash());
}

#[test]
fn nested_branching_assembly() {
    let source = "
        push.3
        push.5
        read
        if.true
            add
            push.7
            read
            if.true
                mul
            else
                add
            endif
        else
            mul
        endif";
    let program = super::compile(source).unwrap();

    let mut root = ExecutionGraph::new(vec![opcodes::BEGIN, opcodes::PUSH, 3, opcodes::PUSH, 5, opcodes::READ]);
    let mut t0 = ExecutionGraph::new(vec![opcodes::ASSERT, opcodes::ADD, opcodes::PUSH, 7, opcodes::READ]);
    let t1 = ExecutionGraph::new(vec![opcodes::ASSERT, opcodes::MUL]);
    let f1 = ExecutionGraph::new(vec![opcodes::NOT, opcodes::ASSERT, opcodes::ADD]);
    t0.set_next(t1, f1);
    let f0 = ExecutionGraph::new(vec![opcodes::NOT, opcodes::ASSERT, opcodes::MUL]);
    root.set_next(t0, f0);

    let expected_program = Program::new(root, blake3);
    assert_eq!(expected_program.hash(), program.hash());
}

#[test]
fn sequential_branching_assembly() {
    let source = "
        push.3
        push.5
        read
        if.true
            add
        else
            mul
        endif
        push.7
        read
        if.true
            mul
        else
            add
        endif";
    let program = super::compile(source).unwrap();

    let mut root = ExecutionGraph::new(vec![opcodes::BEGIN, opcodes::PUSH, 3, opcodes::PUSH, 5, opcodes::READ]);

    let mut t0 = ExecutionGraph::new(vec![opcodes::ASSERT, opcodes::ADD, opcodes::PUSH, 7, opcodes::READ]);
    let t00 = ExecutionGraph::new(vec![opcodes::ASSERT, opcodes::MUL]);
    let f00 = ExecutionGraph::new(vec![opcodes::NOT, opcodes::ASSERT, opcodes::ADD]);
    t0.set_next(t00, f00);

    let mut f0 = ExecutionGraph::new(vec![opcodes::NOT, opcodes::ASSERT, opcodes::MUL, opcodes::PUSH, 7, opcodes::READ]);
    let t10 = ExecutionGraph::new(vec![opcodes::ASSERT, opcodes::MUL]);
    let f10 = ExecutionGraph::new(vec![opcodes::NOT, opcodes::ASSERT, opcodes::ADD]);
    f0.set_next(t10, f10);

    root.set_next(t0, f0);

    let expected_program = Program::new(root, blake3);
    assert_eq!(expected_program.hash(), program.hash());
}

#[test]
fn sequential_nested_branching_assembly() {
    let source = "
        push.3
        push.5
        read
        if.true
            add
            push.7
            read
            if.true
                mul
            else
                add
            endif
        else
            mul
        endif
        push.9
        read
        if.true
            mul
        else
            add
        endif";
    let program = super::compile(source).unwrap();

    let mut root = ExecutionGraph::new(vec![opcodes::BEGIN, opcodes::PUSH, 3, opcodes::PUSH, 5, opcodes::READ]);

    let mut b0 = ExecutionGraph::new(vec![opcodes::ASSERT, opcodes::ADD, opcodes::PUSH, 7, opcodes::READ]);
    let mut b00 = ExecutionGraph::new(vec![opcodes::ASSERT, opcodes::MUL, opcodes::PUSH, 9, opcodes::READ]);
    let b000 = ExecutionGraph::new(vec![opcodes::ASSERT, opcodes::MUL]);
    let b001 = ExecutionGraph::new(vec![opcodes::NOT, opcodes::ASSERT, opcodes::ADD]);
    b00.set_next(b000, b001);

    let mut b01 = ExecutionGraph::new(vec![opcodes::NOT, opcodes::ASSERT, opcodes::ADD, opcodes::PUSH, 9, opcodes::READ]);
    let b010 = ExecutionGraph::new(vec![opcodes::ASSERT, opcodes::MUL]);
    let b011 = ExecutionGraph::new(vec![opcodes::NOT, opcodes::ASSERT, opcodes::ADD]);
    b01.set_next(b010, b011);
    b0.set_next(b00, b01);

    let mut b1 = ExecutionGraph::new(vec![opcodes::NOT, opcodes::ASSERT, opcodes::MUL, opcodes::PUSH, 9, opcodes::READ]);
    let b10 = ExecutionGraph::new(vec![opcodes::ASSERT, opcodes::MUL]);
    let b11 = ExecutionGraph::new(vec![opcodes::NOT, opcodes::ASSERT, opcodes::ADD]);
    b1.set_next(b10, b11);

    root.set_next(b0, b1);

    let expected_program = Program::new(root, blake3);
    assert_eq!(expected_program.hash(), program.hash());
}