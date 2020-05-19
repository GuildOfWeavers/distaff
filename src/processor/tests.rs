#[cfg(test)]
use crate::{ ProofOptions, opcodes, hash_acc, utils::CopyInto };

#[test]
fn execute_verify() {
    let program = [
        opcodes::DUP0, opcodes::PULL2, opcodes::ADD,
        opcodes::DUP0, opcodes::PULL2, opcodes::ADD,
        opcodes::DUP0, opcodes::PULL2, opcodes::ADD,
        opcodes::DUP0, opcodes::PULL2, opcodes::ADD,
        opcodes::DUP0, opcodes::PULL2, opcodes::ADD,
        opcodes::NOOP
    ].iter().map(|&op| op as u64).collect::<Vec<u64>>();
    let expected_hash = hash_acc::digest(&program[..(program.len() - 1)]).copy_into();

    let options = ProofOptions::default();
    let inputs = [1, 0];
    let num_outputs = 1;

    let (outputs, program_hash, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(outputs, [8]);
    assert_eq!(program_hash, expected_hash);

    let result = super::verify(&program_hash, &inputs, &outputs, &proof);
    assert_eq!(Ok(true), result);
}

#[test]
fn execute_verify_fail() {
    let program = [
        opcodes::DUP0, opcodes::PULL2, opcodes::ADD,
        opcodes::DUP0, opcodes::PULL2, opcodes::ADD,
        opcodes::DUP0, opcodes::PULL2, opcodes::ADD,
        opcodes::DUP0, opcodes::PULL2, opcodes::ADD,
        opcodes::DUP0, opcodes::PULL2, opcodes::ADD,
        opcodes::NOOP
    ].iter().map(|&op| op as u64).collect::<Vec<u64>>();
    let expected_hash = hash_acc::digest(&program[..(program.len() - 1)]).copy_into();

    let options = ProofOptions::default();
    let inputs = [1, 0];
    let num_outputs = 1;

    let (outputs, program_hash, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(outputs, [8]);
    assert_eq!(program_hash, expected_hash);

    // wrong inputs
    let result = super::verify(&program_hash, &[1, 1], &outputs, &proof);
    let err_msg = format!("verification of low-degree proof failed: evaluations did not match column value at depth 0");
    assert_eq!(Err(err_msg), result);

    // wrong outputs
    let result = super::verify(&program_hash, &inputs, &[13], &proof);
    let err_msg = format!("verification of low-degree proof failed: evaluations did not match column value at depth 0");
    assert_eq!(Err(err_msg), result);

    // wrong program hash
    let mut program_hash2 = program_hash.clone();
    program_hash2[0] = 1;
    let result = super::verify(&program_hash2, &inputs, &outputs, &proof);
    let err_msg = format!("verification of low-degree proof failed: evaluations did not match column value at depth 0");
    assert_eq!(Err(err_msg), result);
}

// TODO: add more tests