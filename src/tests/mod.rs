use std::collections::HashMap;
use crate::{
    ProofOptions, Program, ProgramInputs, OpCode, OpHint,
    blocks::{ ProgramBlock, Span, Group },
    math::field, utils::hasher
};

mod branches;
mod comparisons;

#[test]
fn execute_verify() {
    let program = build_program(vec![
        OpCode::Begin, OpCode::Swap, OpCode::Dup2, OpCode::Drop,
        OpCode::Add,   OpCode::Swap, OpCode::Dup2, OpCode::Drop,
        OpCode::Add,   OpCode::Swap, OpCode::Dup2, OpCode::Drop,
        OpCode::Add,   OpCode::Noop, OpCode::Noop,
    ], &[]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[1, 0]);
    let num_outputs = 1;

    let (outputs, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(outputs, [3]);

    let result = super::verify(program.hash(), inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

#[test]
fn execute_verify_fail() {
    let program = build_program(vec![
        OpCode::Begin, OpCode::Swap, OpCode::Dup2, OpCode::Drop,
        OpCode::Add,   OpCode::Swap, OpCode::Dup2, OpCode::Drop,
        OpCode::Add,   OpCode::Swap, OpCode::Dup2, OpCode::Drop,
        OpCode::Add,   OpCode::Noop, OpCode::Noop,
    ], &[]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[1, 0]);
    let num_outputs = 1;

    let (outputs, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(outputs, [3]);

    // wrong inputs
    let result = super::verify(program.hash(), &[1, 1], &outputs, &proof);
    let err_msg = format!("verification of low-degree proof failed: evaluations did not match column value at depth 0");
    assert_eq!(Err(err_msg), result);

    // wrong outputs
    let result = super::verify(program.hash(), inputs.get_public_inputs(), &[5], &proof);
    let err_msg = format!("verification of low-degree proof failed: evaluations did not match column value at depth 0");
    assert_eq!(Err(err_msg), result);

    // wrong program hash
    let mut program_hash2 = program.hash().clone();
    program_hash2[0] = 1;
    let result = super::verify(&program_hash2, inputs.get_public_inputs(), &outputs, &proof);
    let err_msg = format!("verification of low-degree proof failed: evaluations did not match column value at depth 0");
    assert_eq!(Err(err_msg), result);
}

#[test]
fn stack_manipulation() {
    let program = build_program(vec![
        OpCode::Begin, OpCode::Noop,  OpCode::Noop,  OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,  OpCode::Noop,  OpCode::Noop,
        OpCode::Swap,  OpCode::Swap2, OpCode::Swap4, OpCode::Roll4,
        OpCode::Roll8, OpCode::Dup,   OpCode::Add,   OpCode::Pad2,
        OpCode::Push,  OpCode::Swap4, OpCode::Drop4, OpCode::Dup2,
        OpCode::Swap4, OpCode::Add,   OpCode::Add,   OpCode::Dup4,
        OpCode::Push,  OpCode::Add,   OpCode::Add,   OpCode::Add,
        OpCode::Add,   OpCode::Noop,  OpCode::Noop,
    ], &[11, 12]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[7, 6, 5, 4, 3, 2, 1, 0]);
    let num_outputs = 8;

    let (outputs, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(outputs, [46, 19, 4, 11, 0, 11, 0, 6]);

    let result = super::verify(program.hash(), inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

#[test]
fn conditional_operations() {
    // CHOOSE
    let program = build_program(vec![
        OpCode::Begin, OpCode::Choose,  OpCode::Choose, OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,    OpCode::Noop, OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,    OpCode::Noop, OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,    OpCode::Noop,
    ], &[]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[3, 4, 1, 5, 0, 6, 7, 8]);
    let num_outputs = 8;

    let (outputs, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(outputs, [5, 6, 7, 8, 0, 0, 0, 0]);

    let result = super::verify(program.hash(), inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);

    // CHOOSE2
    let program = build_program(vec![
        OpCode::Begin, OpCode::Noop,    OpCode::Noop,    OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,    OpCode::Noop,    OpCode::Noop,
        OpCode::Push,  OpCode::Noop,    OpCode::Noop,    OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,    OpCode::Noop,    OpCode::Noop,
        OpCode::Push,  OpCode::Choose2, OpCode::Choose2, OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,    OpCode::Noop,    OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,    OpCode::Noop,    OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,    OpCode::Noop,
    ], &[3, 4]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[5, 6, 1, 0, 7, 8, 0, 0]);
    let num_outputs = 8;

    let (outputs, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(outputs, [7, 8, 0, 0, 0, 0, 0, 0]);

    let result = super::verify(program.hash(), inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);

    // CSWAP2
    let program = build_program(vec![
        OpCode::Begin,  OpCode::CSwap2,  OpCode::Pad2,   OpCode::Swap4,
        OpCode::CSwap2, OpCode::Noop,    OpCode::Noop,   OpCode::Noop,
        OpCode::Noop,   OpCode::Noop,    OpCode::Noop,   OpCode::Noop,
        OpCode::Noop,   OpCode::Noop,    OpCode::Noop,
    ], &[]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[3, 4, 1, 2, 1, 0, 5, 6]);
    let num_outputs = 8;

    let (outputs, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(outputs, [3, 4, 5, 6, 1, 2, 0, 0]);

    let result = super::verify(program.hash(), inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

#[test]
#[should_panic(expected = "CHOOSE on a non-binary condition at step 2")]
fn selection_operations_panic() {
    let program = build_program(vec![
        OpCode::Begin, OpCode::Choose, OpCode::Choose, OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,   OpCode::Noop,   OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,   OpCode::Noop,   OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,   OpCode::Noop,
    ], &[]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[3, 4, 2, 5, 0, 6, 7, 8]);
    let num_outputs = 8;

    super::execute(&program, &inputs, num_outputs, &options);
}

#[test]
fn math_operations() {
    let program = build_program(vec![
        OpCode::Begin, OpCode::Add,  OpCode::Mul,  OpCode::Inv,
        OpCode::Neg,   OpCode::Swap, OpCode::Not,  OpCode::Noop, 
        OpCode::Noop,  OpCode::Noop, OpCode::Noop, OpCode::Noop, 
        OpCode::Noop,  OpCode::Noop, OpCode::Noop, 
    ], &[]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[7, 6, 5, 0, 2, 3]);
    let num_outputs = 2;

    let expected_result = vec![field::ONE, field::neg(field::inv(65))];

    let (outputs, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(expected_result, outputs);

    let result = super::verify(program.hash(), inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

#[test]
fn bool_operations() {
    let program = build_program(vec![
        OpCode::Begin, OpCode::Not,  OpCode::Or,   OpCode::Or,
        OpCode::And,   OpCode::And,  OpCode::Not,  OpCode::Noop,
        OpCode::Noop,  OpCode::Noop, OpCode::Noop, OpCode::Noop,
        OpCode::Noop,  OpCode::Noop, OpCode::Noop,
    ], &[]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[1, 0, 1, 1, 0]);
    let num_outputs = 1;

    let expected_result = vec![field::ONE];

    let (outputs, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(expected_result, outputs);

    let result = super::verify(program.hash(), inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

#[test]
fn hash_operations() {
    // single hash
    let program = build_program(vec![
        OpCode::Begin, OpCode::Noop,  OpCode::Noop,  OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,  OpCode::Noop,  OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,  OpCode::Noop,  OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,  OpCode::Noop,  OpCode::Noop,
        OpCode::RescR, OpCode::RescR, OpCode::RescR, OpCode::RescR,
        OpCode::RescR, OpCode::RescR, OpCode::RescR, OpCode::RescR,
        OpCode::RescR, OpCode::RescR, OpCode::Drop,  OpCode::Drop,
        OpCode::Drop,  OpCode::Drop,  OpCode::Noop
    ], &[]);

    let value = [1, 2, 3, 4];
    let mut expected_hash = hasher::digest(&value);
    expected_hash.reverse();

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[0, 0, 4, 3, 2, 1]);
    let num_outputs = 2;

    let (outputs, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(expected_hash, outputs);

    let result = super::verify(program.hash(), inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);

    // double hash
    let program = build_program(vec![
        OpCode::Begin, OpCode::Noop,  OpCode::Noop,  OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,  OpCode::Noop,  OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,  OpCode::Noop,  OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,  OpCode::Noop,  OpCode::Noop,
        OpCode::RescR, OpCode::RescR, OpCode::RescR, OpCode::RescR,
        OpCode::RescR, OpCode::RescR, OpCode::RescR, OpCode::RescR,
        OpCode::RescR, OpCode::RescR, OpCode::Drop4, OpCode::Noop,
        OpCode::Pad2,  OpCode::Dup2,  OpCode::Noop,  OpCode::Noop,
        OpCode::RescR, OpCode::RescR, OpCode::RescR, OpCode::RescR,
        OpCode::RescR, OpCode::RescR, OpCode::RescR, OpCode::RescR,
        OpCode::RescR, OpCode::RescR, OpCode::Drop4, OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,  OpCode::Noop
    ], &[]);

    let value = [1, 2, 3, 4];
    let mut expected_hash = hasher::digest(&value);
    expected_hash = hasher::digest(&expected_hash);
    expected_hash.reverse();

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[0, 0, 4, 3, 2, 1]);
    let num_outputs = 2;

    let (outputs, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(expected_hash, outputs);

    let result = super::verify(program.hash(), inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

#[test]
fn read_operations() {
    let program = build_program(vec![
        OpCode::Begin, OpCode::Read, OpCode::Read2, OpCode::Noop,
        OpCode::Noop, OpCode::Noop,  OpCode::Noop,  OpCode::Noop,
        OpCode::Push, OpCode::Noop,  OpCode::Noop,  OpCode::Noop,
        OpCode::Noop, OpCode::Noop,  OpCode::Noop,
    ], &[5]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::new(&[1], &[2, 3], &[4]);
    let num_outputs = 5;

    let (outputs, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(vec![5, 4, 3, 2, 1], outputs);

    let result = super::verify(program.hash(), inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

#[test]
fn assert_operations() {
    let program = build_program(vec![
        OpCode::Begin, OpCode::Assert, OpCode::Noop, OpCode::AssertEq,
        OpCode::Noop,  OpCode::Noop,   OpCode::Noop, OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,   OpCode::Noop, OpCode::Noop,
        OpCode::Noop,  OpCode::Noop,   OpCode::Noop,
    ], &[]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[1, 3, 3]);
    let num_outputs = 2;

    let expected_result = vec![0, 0];

    let (outputs, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(expected_result, outputs);

    let result = super::verify(program.hash(), inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

// TODO: add more tests

// HELPER FUNCTIONS
// ================================================================================================
fn build_program(instructions: Vec<OpCode>, push_values: &[u128]) -> Program {

    // build hint map for PUSh operations
    let mut j = 0;
    let mut hints = HashMap::new();
    for i in 0..instructions.len() {
        match instructions[i] {
            OpCode::Push => {
                assert!(j < push_values.len(), "not enough push values");
                hints.insert(i, OpHint::PushValue(push_values[j]));
                j += 1;
            },
            _ => ()
        }
    }
    assert!(j == push_values.len(), "too many push values");

    let root = vec![ProgramBlock::Span(Span::new(instructions, hints))];
    return Program::new(Group::new(root));
}