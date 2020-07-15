use crate::crypto::{ hash::blake3 };
use crate::utils::{ as_bytes };
use crate::processor::opcodes2::{ UserOps as Opcode };
use super::{ Program, ProgramBlock, Span, Group, Switch, Loop };

mod utils;
use utils::{ traverse, close_block };

// TESTS
// ================================================================================================

#[test]
fn single_block() {
    let block = Span::new_block(vec![Opcode::Noop; 15]);

    let program = Program::from_proc(vec![block]);
    let procedure = program.get_proc(0);
    let mut program_hash = [0, 0, 0, 0];
    let step = traverse(procedure.body(), &mut vec![], &mut program_hash, 0);
    let step = close_block(&mut program_hash, 0, 0, true, step);

    assert_eq!(*program.hash(), hash_to_bytes(&program_hash));
    assert_eq!(31, step);
}

#[test]
fn linear_blocks() {
    let block1 = Span::new_block(vec![Opcode::Noop; 15]);

    let inner_block1 = Span::new_block(vec![Opcode::Add; 15]);
    let block2 = Group::new_block(vec![inner_block1]);

    let inner_block2 = Span::new_block(vec![Opcode::Mul; 15]);
    let block3 = Group::new_block(vec![inner_block2]);

    // sequence of blocks ending with group block
    let program = Program::from_proc(vec![block1.clone(), block2.clone(), block3.clone()]);
    let procedure = program.get_proc(0);
    let mut program_hash = [0, 0, 0, 0];
    let step = traverse(procedure.body(), &mut vec![], &mut program_hash, 0);
    let step = close_block(&mut program_hash, 0, 0, true, step);

    assert_eq!(*program.hash(), hash_to_bytes(&program_hash));
    assert_eq!(95, step);

    // sequence of blocks ending with span block
    let block4 = Span::new_block(vec![Opcode::Inv; 15]);

    let program = Program::from_proc(vec![block1, block2, block3, block4]);
    let procedure = program.get_proc(0);
    let mut program_hash = [0, 0, 0, 0];
    let step = traverse(procedure.body(), &mut vec![], &mut program_hash, 0);
    let step = close_block(&mut program_hash, 0, 0, true, step);

    assert_eq!(*program.hash(), hash_to_bytes(&program_hash));
    assert_eq!(111, step);
}

#[test]
fn nested_blocks() {
    let block1 = Span::new_block(vec![Opcode::Noop; 15]);

    let inner_block1 = Span::new_block(vec![Opcode::Add; 15]);
    let block2 = Group::new_block(vec![inner_block1]);

    let inner_block2 = Span::new_block(vec![Opcode::Mul; 15]);
    let inner_inner_block1 = Span::new_block(vec![Opcode::Inv; 15]);
    let inner_block3 = Group::new_block(vec![inner_inner_block1]);
    let block3 = Group::new_block(vec![inner_block2, inner_block3]);

    // sequence of blocks ending with group block
    let program = Program::from_proc(vec![block1, block2, block3]);
    let procedure = program.get_proc(0);
    let mut program_hash = [0, 0, 0, 0];
    let step = traverse(procedure.body(), &mut vec![], &mut program_hash, 0);
    let step = close_block(&mut program_hash, 0, 0, true, step);

    assert_eq!(*program.hash(), hash_to_bytes(&program_hash));
    assert_eq!(127, step);
}

#[test]
fn conditional_program() {
    let block1 = Span::new_block(vec![Opcode::Noop; 15]);

    let t_branch = vec![Span::new_block(vec![
        Opcode::Assert, Opcode::Add, Opcode::Add, Opcode::Add,
        Opcode::Add,    Opcode::Add, Opcode::Add, Opcode::Add,
        Opcode::Add,    Opcode::Add, Opcode::Add, Opcode::Add,
        Opcode::Add,    Opcode::Add, Opcode::Add,
    ])];
    let f_branch = vec![Span::new_block(vec![
        Opcode::Not, Opcode::Assert, Opcode::Mul, Opcode::Mul,
        Opcode::Mul, Opcode::Mul,    Opcode::Mul, Opcode::Mul,
        Opcode::Mul, Opcode::Mul,    Opcode::Mul, Opcode::Mul,
        Opcode::Mul, Opcode::Mul,    Opcode::Mul,
    ])];
    let block2 = Switch::new_block(t_branch, f_branch);
    
    let program = Program::from_proc(vec![block1, block2]);
    let procedure = program.get_proc(0);

    // true branch execution
    let mut program_hash = [0, 0, 0, 0];
    let step = traverse(procedure.body(), &mut vec![1], &mut program_hash, 0);
    let step = close_block(&mut program_hash, 0, 0, true, step);
    assert_eq!(*program.hash(), hash_to_bytes(&program_hash));
    assert_eq!(63, step);

    // false branch execution
    let mut program_hash = [0, 0, 0, 0];
    let step = traverse(procedure.body(), &mut vec![0], &mut program_hash, 0);
    let step = close_block(&mut program_hash, 0, 0, true, step);
    assert_eq!(*program.hash(), hash_to_bytes(&program_hash));
    assert_eq!(63, step);
}

#[test]
fn simple_loop() {
    let block1 = Span::new_block(vec![Opcode::Noop; 15]);

    let loop_body = vec![Span::new_block(vec![
        Opcode::Assert, Opcode::Add, Opcode::Add, Opcode::Add,
        Opcode::Add,    Opcode::Add, Opcode::Add, Opcode::Add,
        Opcode::Add,    Opcode::Add, Opcode::Add, Opcode::Add,
        Opcode::Add,    Opcode::Add, Opcode::Add,
    ])];
    let block2 = Loop::new_block(loop_body);
    
    let program = Program::from_proc(vec![block1, block2]);
    let procedure = program.get_proc(0);

    // loop not entered
    let mut program_hash = [0, 0, 0, 0];
    let step = traverse(procedure.body(), &mut vec![0], &mut program_hash, 0);
    let step = close_block(&mut program_hash, 0, 0, true, step);
    assert_eq!(*program.hash(), hash_to_bytes(&program_hash));
    assert_eq!(63, step);

    // loop executed once
    let mut program_hash = [0, 0, 0, 0];
    let step = traverse(procedure.body(), &mut vec![0, 1], &mut program_hash, 0);
    let step = close_block(&mut program_hash, 0, 0, true, step);
    assert_eq!(*program.hash(), hash_to_bytes(&program_hash));
    assert_eq!(79, step);

    // loop executed 3 times
    let mut program_hash = [0, 0, 0, 0];
    let step = traverse(procedure.body(), &mut vec![0, 1, 1, 1], &mut program_hash, 0);
    let step = close_block(&mut program_hash, 0, 0, true, step);
    assert_eq!(*program.hash(), hash_to_bytes(&program_hash));
    assert_eq!(111, step);
}

#[test]
fn program_with_two_procedures() {
    
    let block1 = Group::new(vec![Span::new_block(vec![Opcode::Add; 15])]);
    let block2 = Group::new(vec![Span::new_block(vec![Opcode::Mul; 15])]);
    
    let program = Program::new(vec![block1.clone(), block2.clone()], blake3);
    
    let mut hash1 = [0, 0, 0, 0];
    let step = traverse(block1.body(), &mut vec![], &mut hash1, 0);
    close_block(&mut hash1, 0, 0, true, step);

    let mut hash2 = [0, 0, 0, 0];
    let step = traverse(block2.body(), &mut vec![], &mut hash2, 0);
    close_block(&mut hash2, 0, 0, true, step);
    
    let buf = [as_bytes(&hash1[..2]), as_bytes(&hash2[..2])].concat();
    let mut program_hash = [0u8; 32];
    blake3(&buf, &mut program_hash);

    assert_eq!(*program.hash(), program_hash);
}

#[test]
fn procedure_authentication() {
    
    let block1 = Group::new(vec![Span::new_block(vec![Opcode::Add; 15])]);
    let block2 = Group::new(vec![Span::new_block(vec![Opcode::Mul; 15])]);
    
    let program = Program::new(vec![block1.clone(), block2.clone()], blake3);
    
    let mut hash1 = [0, 0, 0, 0];
    let step = traverse(block1.body(), &mut vec![], &mut hash1, 0);
    close_block(&mut hash1, 0, 0, true, step);
    let hash1 = hash_to_bytes(&hash1);

    let mut hash2 = [0, 0, 0, 0];
    let step = traverse(block2.body(), &mut vec![], &mut hash2, 0);
    close_block(&mut hash2, 0, 0, true, step);
    let hash2 = hash_to_bytes(&hash2);
    
    let path1 = program.get_proc_path(0);
    assert_eq!(vec![hash1, hash2], path1);
    Program::verify_proc_path(program.hash(), 0, &path1, blake3);

    let path2 = program.get_proc_path(1);
    assert_eq!(vec![hash2, hash1], path2);
    Program::verify_proc_path(program.hash(), 1, &path2, blake3);
}

// HELPER FUNCTIONS
// ================================================================================================
fn hash_to_bytes(hash: &[u128; 4]) -> [u8; 32] {
    let mut hash_bytes = [0u8; 32];
    hash_bytes.copy_from_slice(&as_bytes(&hash[..2]));
    return hash_bytes;
}