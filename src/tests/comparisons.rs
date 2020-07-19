/*
TODO: re-enable
use crate::{ ProofOptions, opcodes::f128 as opcodes, math::field };
use super::super::{ execute, verify, Program, ProgramInputs };

#[test]
fn eq_operations() {
    let program = Program::from_path(vec![
        opcodes::BEGIN, opcodes::READ,  opcodes::EQ,    opcodes::SWAP2,
        opcodes::READ,  opcodes::EQ,    opcodes::NOOP,  opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
    ]);

    let options = ProofOptions::default();
    let diff_inv = field::inv(field::sub(1, 2));
    let inputs = ProgramInputs::new(&[1, 2, 3, 4, 4], &[diff_inv, 1], &[]);
    let num_outputs = 3;

    let expected_result = vec![1, 0, 3];

    let (outputs, proof) = execute(&program, &inputs, num_outputs, &options);
    assert_eq!(expected_result, outputs);

    let result = verify(program.hash(), inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

#[test]
fn cmp_operation() {

    let a: u128 = field::rand();
    let b: u128 = field::rand();
    let p127: u128 = field::exp(2, 127);

    // build inputs
    let (inputs_a, inputs_b) = build_inputs_for_cmp(a, b, 128);

    // build the program
    let mut program = vec![opcodes::BEGIN, opcodes::PAD2, opcodes::PUSH, p127];
    for _ in 0..128 { program.push(opcodes::CMP);  }
    program.push(opcodes::DROP4);
    while program.len() < 256 { program.push(opcodes::NOOP); }

    let program = Program::from_path(program);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::new(&[0, 0, 0, 0, 0, a, b], &inputs_a, &inputs_b);
    let num_outputs = 4;

    let lt = if a < b { field::ONE }  else { field::ZERO };
    let gt = if a < b { field::ZERO } else { field::ONE  };
    let expected_result = vec![gt, lt, b, a];

    // execute the program and make sure results are correct
    let (outputs, proof) = execute(&program, &inputs, num_outputs, &options);
    assert_eq!(expected_result, outputs);

    // verify execution proof
    let result = verify(program.hash(), inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

#[test]
fn binacc_operation() {

    let a: u128 = field::rand();
    let p127: u128 = field::exp(2, 127);

    // build inputs
    let mut inputs_a = Vec::new();
    for i in 0..128 { inputs_a.push((a >> i) & 1); }
    inputs_a.reverse();

    // build the program
    let mut program = vec![opcodes::BEGIN];
    for _ in 0..128 { program.push(opcodes::BINACC); }
    program.push(opcodes::DROP);
    program.push(opcodes::DROP);
    while program.len() < 256 { program.push(opcodes::NOOP); }

    let program = Program::from_path(program);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::new(&[p127, 0, 0, a], &inputs_a, &[]);
    let num_outputs = 2;

    let expected_result = vec![a, a];

    // execute the program and make sure results are correct
    let (outputs, proof) = execute(&program, &inputs, num_outputs, &options);
    assert_eq!(expected_result, outputs);

    // verify execution proof
    let result = verify(program.hash(), inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

// HELPER FUNCTIONS
// ================================================================================================
fn build_inputs_for_cmp(a: u128, b: u128, size: usize) -> (Vec<u128>, Vec<u128>) {

    let mut inputs_a = Vec::new();
    let mut inputs_b = Vec::new();
    for i in 0..size {
        inputs_a.push((a >> i) & 1);
        inputs_b.push((b >> i) & 1);
    }
    inputs_a.reverse();
    inputs_b.reverse();

    return (inputs_a, inputs_b);
}
*/