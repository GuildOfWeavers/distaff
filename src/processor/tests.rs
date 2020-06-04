#[cfg(test)]
use crate::{ ProofOptions, ProgramInputs, opcodes::f128 as opcodes, F128, FiniteField, Accumulator, Hasher };

#[test]
fn execute_verify() {
    let program = [
        opcodes::BEGIN, opcodes::SWAP, opcodes::DUP2, opcodes::DROP,
        opcodes::ADD,   opcodes::SWAP, opcodes::DUP2, opcodes::DROP,
        opcodes::ADD,   opcodes::SWAP, opcodes::DUP2, opcodes::DROP,
        opcodes::ADD,   opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
    ];
    let expected_hash = <F128 as Accumulator>::digest(&program[..(program.len() - 1)]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[1, 0]);
    let num_outputs = 1;

    let (outputs, program_hash, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(outputs, [3]);
    assert_eq!(program_hash, expected_hash);

    let result = super::verify(&program_hash, inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

#[test]
fn execute_verify_fail() {
    let program = [
        opcodes::BEGIN, opcodes::SWAP, opcodes::DUP2, opcodes::DROP,
        opcodes::ADD,   opcodes::SWAP, opcodes::DUP2, opcodes::DROP,
        opcodes::ADD,   opcodes::SWAP, opcodes::DUP2, opcodes::DROP,
        opcodes::ADD,   opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
    ];
    let expected_hash = <F128 as Accumulator>::digest(&program[..(program.len() - 1)]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[1, 0]);
    let num_outputs = 1;

    let (outputs, program_hash, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(outputs, [3]);
    assert_eq!(program_hash, expected_hash);

    // wrong inputs
    let result = super::verify(&program_hash, &[1, 1], &outputs, &proof);
    let err_msg = format!("verification of low-degree proof failed: evaluations did not match column value at depth 0");
    assert_eq!(Err(err_msg), result);

    // wrong outputs
    let result = super::verify(&program_hash, inputs.get_public_inputs(), &[5], &proof);
    let err_msg = format!("verification of low-degree proof failed: evaluations did not match column value at depth 0");
    assert_eq!(Err(err_msg), result);

    // wrong program hash
    let mut program_hash2 = program_hash.clone();
    program_hash2[0] = 1;
    let result = super::verify(&program_hash2, inputs.get_public_inputs(), &outputs, &proof);
    let err_msg = format!("verification of low-degree proof failed: evaluations did not match column value at depth 0");
    assert_eq!(Err(err_msg), result);
}

#[test]
fn stack_operations() {
    let program = [
        opcodes::BEGIN,  opcodes::SWAP,    opcodes::SWAP2, opcodes::SWAP4,
        opcodes::CHOOSE, opcodes::PUSH,    11,             opcodes::ROLL4, 
        opcodes::DUP,    opcodes::CHOOSE2, opcodes::DUP4,  opcodes::ROLL8,
        opcodes::DROP,   opcodes::DROP,    opcodes::DUP2,  opcodes::NOOP
    ];
    let expected_hash = <F128 as Accumulator>::digest(&program[..(program.len() - 1)]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[7, 6, 5, 4, 3, 2, 1, 0]);
    let num_outputs = 8;

    let (outputs, program_hash, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(outputs, [3, 6, 3, 6, 7, 11, 3, 6]);
    assert_eq!(program_hash, expected_hash);

    let result = super::verify(&program_hash, inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

#[test]
fn logic_operations() {
    // CHOOSE
    let program = [
        opcodes::BEGIN,  opcodes::CHOOSE,  opcodes::CHOOSE, opcodes::NOOP,
        opcodes::NOOP,   opcodes::NOOP,    opcodes::NOOP,   opcodes::NOOP,
        opcodes::NOOP,   opcodes::NOOP,    opcodes::NOOP,   opcodes::NOOP,
        opcodes::NOOP,   opcodes::NOOP,    opcodes::NOOP,   opcodes::NOOP,
    ];
    let expected_hash = <F128 as Accumulator>::digest(&program[..(program.len() - 1)]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[3, 4, 1, 5, 0, 6, 7, 8]);
    let num_outputs = 8;

    let (outputs, program_hash, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(outputs, [5, 6, 7, 8, 0, 0, 0, 0]);
    assert_eq!(program_hash, expected_hash);

    let result = super::verify(&program_hash, inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);

    // CHOOSE2
    let program = [
        opcodes::BEGIN, opcodes::PUSH,    3,                opcodes::PUSH,
        4,              opcodes::CHOOSE2, opcodes::CHOOSE2, opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,    opcodes::NOOP,    opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,    opcodes::NOOP,    opcodes::NOOP,
    ];
    let expected_hash = <F128 as Accumulator>::digest(&program[..(program.len() - 1)]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[5, 6, 1, 0, 7, 8, 0, 0]);
    let num_outputs = 8;

    let (outputs, program_hash, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(outputs, [7, 8, 0, 0, 0, 0, 0, 0]);
    assert_eq!(program_hash, expected_hash);

    let result = super::verify(&program_hash, inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

#[test]
#[should_panic]
fn logic_operations_panic() {
    let program = [
        opcodes::BEGIN,  opcodes::CHOOSE,  opcodes::CHOOSE, opcodes::NOOP,
        opcodes::NOOP,   opcodes::NOOP,    opcodes::NOOP,   opcodes::NOOP,
        opcodes::NOOP,   opcodes::NOOP,    opcodes::NOOP,   opcodes::NOOP,
        opcodes::NOOP,   opcodes::NOOP,    opcodes::NOOP,   opcodes::NOOP,
    ];

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[3, 4, 2, 5, 0, 6, 7, 8]);
    let num_outputs = 8;

    super::execute(&program, &inputs, num_outputs, &options);
}

#[test]
fn math_operations() {
    let program = [
        opcodes::BEGIN, opcodes::ADD,  opcodes::MUL,  opcodes::INV,
        opcodes::NEG,   opcodes::SWAP, opcodes::NOT,  opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
    ];
    let expected_hash = <F128 as Accumulator>::digest(&program[..(program.len() - 1)]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[7, 6, 5, 0, 2, 3]);
    let num_outputs = 2;

    let expected_result = vec![F128::ONE, F128::neg(F128::inv(65))];

    let (outputs, program_hash, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(expected_result, outputs);
    assert_eq!(program_hash, expected_hash);

    let result = super::verify(&program_hash, inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

#[test]
fn hash_operations() {
    // single hash
    let program = [
        opcodes::BEGIN, opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
        opcodes::HASHR, opcodes::HASHR, opcodes::HASHR, opcodes::HASHR,
        opcodes::HASHR, opcodes::HASHR, opcodes::HASHR, opcodes::HASHR,
        opcodes::HASHR, opcodes::HASHR, opcodes::DROP,  opcodes::DROP,
        opcodes::DROP,  opcodes::DROP,  opcodes::NOOP,  opcodes::NOOP,
    ];

    let value = [1, 2, 3, 4];
    let mut expected_hash = <F128 as Hasher>::digest(&value);
    expected_hash.reverse();

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[0, 0, 4, 3, 2, 1]);
    let num_outputs = 2;

    let (outputs, program_hash, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(expected_hash, outputs);

    let result = super::verify(&program_hash, inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);

    // double hash
    let program = [
        opcodes::BEGIN, opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
        opcodes::HASHR, opcodes::HASHR, opcodes::HASHR, opcodes::HASHR,
        opcodes::HASHR, opcodes::HASHR, opcodes::HASHR, opcodes::HASHR,
        opcodes::HASHR, opcodes::HASHR, opcodes::DROP4, opcodes::NOOP,
        opcodes::PAD2,  opcodes::DUP2,  opcodes::NOOP,  opcodes::NOOP,
        opcodes::HASHR, opcodes::HASHR, opcodes::HASHR, opcodes::HASHR,
        opcodes::HASHR, opcodes::HASHR, opcodes::HASHR, opcodes::HASHR,
        opcodes::HASHR, opcodes::HASHR, opcodes::DROP4, opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
    ];

    let value = [1, 2, 3, 4];
    let mut expected_hash = <F128 as Hasher>::digest(&value);
    expected_hash = <F128 as Hasher>::digest(&expected_hash);
    expected_hash.reverse();

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[0, 0, 4, 3, 2, 1]);
    let num_outputs = 2;

    let (outputs, program_hash, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(expected_hash, outputs);

    let result = super::verify(&program_hash, inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

#[test]
fn read_operations() {
    let program = [
        opcodes::BEGIN, opcodes::READ,  opcodes::READ2, opcodes::NOOP,
        opcodes::PUSH,      5,          opcodes::NOOP,  opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
    ];

    let options = ProofOptions::default();
    let inputs = ProgramInputs::new(&[1], &[2, 3], &[4]);
    let num_outputs = 5;

    let (outputs, program_hash, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(vec![5, 4, 3, 2, 1], outputs);

    let result = super::verify(&program_hash, inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

#[test]
fn comparison_operations() {
    let program = [
        opcodes::BEGIN, opcodes::EQ,     opcodes::SWAP2, opcodes::EQ,
        opcodes::NOOP,  opcodes::NOOP,   opcodes::NOOP,  opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,   opcodes::NOOP,  opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,   opcodes::NOOP,  opcodes::NOOP,
    ];
    let expected_hash = <F128 as Accumulator>::digest(&program[..(program.len() - 1)]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[1, 2, 3, 4, 4]);
    let num_outputs = 3;

    let expected_result = vec![1, 0, 3];

    let (outputs, program_hash, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(expected_result, outputs);
    assert_eq!(program_hash, expected_hash);

    let result = super::verify(&program_hash, inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

#[test]
fn assert_operation() {
    let program = [
        opcodes::BEGIN, opcodes::ASSERT, opcodes::NOOP, opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,   opcodes::NOOP, opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,   opcodes::NOOP, opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,   opcodes::NOOP, opcodes::NOOP,
    ];
    let expected_hash = <F128 as Accumulator>::digest(&program[..(program.len() - 1)]);

    let options = ProofOptions::default();
    let inputs = ProgramInputs::from_public(&[1, 2, 3]);
    let num_outputs = 2;

    let expected_result = vec![2, 3];

    let (outputs, program_hash, proof) = super::execute(&program, &inputs, num_outputs, &options);
    assert_eq!(expected_result, outputs);
    assert_eq!(program_hash, expected_hash);

    let result = super::verify(&program_hash, inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}

// TODO: add more tests