/*
TODO: re-enable
use crate::{ ProofOptions, opcodes::f128 as opcodes, ExecutionGraph };
use super::{ Program, ProgramInputs, super::{ execute, verify, } };

#[test]
fn simple_branching() {
    // build program execution graph
    let mut exe_graph = ExecutionGraph::new(vec![
        opcodes::BEGIN, opcodes::PUSH, 3, opcodes::PUSH, 5, opcodes::READ
    ]);
    let true_branch = ExecutionGraph::new(vec![
        opcodes::ASSERT, opcodes::ADD
    ]);
    let false_branch = ExecutionGraph::new(vec![
        opcodes::NOT, opcodes::ASSERT, opcodes::MUL
    ]);
    exe_graph.set_next(true_branch, false_branch);

    let options = ProofOptions::default();
    let program = Program::new(exe_graph, options.hash_fn());
    let num_outputs = 1;

    // test true branch
    let inputs = ProgramInputs::new(&[], &[1], &[]);
    let (outputs, proof) = execute(&program, &inputs, num_outputs, &options);
    assert_eq!(outputs, [8]);
    let result = verify(program.hash(), inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);

    // test false branch
    let inputs = ProgramInputs::new(&[], &[0], &[]);
    let (outputs, proof) = execute(&program, &inputs, num_outputs, &options);
    assert_eq!(outputs, [15]);
    let result = verify(program.hash(), inputs.get_public_inputs(), &outputs, &proof);
    assert_eq!(Ok(true), result);
}
*/