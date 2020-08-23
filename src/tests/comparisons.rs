use crate::{ ProofOptions, math::field };
use super::{
    build_program, OpCode,
    super::{ execute, verify, ProgramInputs }
};

#[test]
fn eq_operations() {
    let program = build_program(vec![
        OpCode::Begin, OpCode::Read, OpCode::Eq,   OpCode::Swap2,
        OpCode::Read,  OpCode::Eq,   OpCode::Noop, OpCode::Noop,
        OpCode::Noop,  OpCode::Noop, OpCode::Noop, OpCode::Noop,
        OpCode::Noop,  OpCode::Noop, OpCode::Noop,
    ], &[]);

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
    let mut instructions = vec![
        OpCode::Begin, OpCode::Pad2, OpCode::Noop, OpCode::Noop,
        OpCode::Noop,  OpCode::Noop, OpCode::Noop, OpCode::Noop,
        OpCode::Push,
    ];
    for _ in 0..128 { instructions.push(OpCode::Cmp);  }
    instructions.push(OpCode::Drop4);
    while instructions.len() < 255 { instructions.push(OpCode::Noop); }

    let program = build_program(instructions, &[p127]);

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

    // build inputs
    let mut inputs_a = Vec::new();
    for i in 0..128 { inputs_a.push((a >> (127 - i)) & 1); }
    inputs_a.reverse();

    // build the program
    let mut instructions = vec![OpCode::Begin];
    for _ in 0..128 { instructions.push(OpCode::BinAcc); }
    instructions.push(OpCode::Drop);
    instructions.push(OpCode::Drop);
    instructions.push(OpCode::Drop);
    while instructions.len() < 255 { instructions.push(OpCode::Noop); }

    let program = build_program(instructions, &[]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::new(
        &[0, 0, 1, 0, a],
        &inputs_a,
        &[]);
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