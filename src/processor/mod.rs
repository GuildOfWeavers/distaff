use std::cmp;
use crate::math::{ F128, FiniteField };
use crate::stark::{ MIN_TRACE_LENGTH };

mod decoder;
use decoder::Decoder;

mod stack;
use stack::Stack;

mod program;
pub use program::{ Program, ProgramInputs };

pub mod opcodes;
pub mod assembly;

/// Executes the specified `program` and returns the result together with program hash
/// and STARK-based proof of execution.
/// 
/// * `inputs` specify the initial stack state the with inputs[0] being the top of the stack;
/// * `num_outputs` specifies the number of elements from the top of the stack to be returned;
pub fn execute(program: &Program, inputs: &ProgramInputs<F128>) -> Vec<Vec<u128>>
{
    // TODO: use the entire program to build the trace table
    let mut program = program.execution_graph().operations().to_vec();
    pad_program(&mut program);

    // TODO: clean up
    // execute the program
    let mut decoder = Decoder::new(program.len());
    let mut stack = Stack::new(inputs, program.len());

    let mut step = 0;
    while step < program.len() - 1 {
        
        decoder.decode(program[step], false, step);
        stack.execute(program[step], program[step + 1], step);

        if program[step] == opcodes::f128::PUSH {
            step += 1;
            decoder.decode(program[step], true, step);
            stack.execute(opcodes::f128::NOOP, 0, step);
        }

        step += 1;
    }

    let mut register_traces = decoder.into_register_trace();
    register_traces.append(&mut stack.into_register_traces());

    return register_traces;
}

/// Pads the program with the appropriate number of NOOPs to ensure that:
/// 1. The length of the program is at least 16;
/// 2. The length of the program is a power of 2;
/// 3. The program terminates with a NOOP.
pub fn pad_program<T: FiniteField>(program: &mut Vec<T>) {
    
    let trace_length = if program.len() == program.len().next_power_of_two() {
        if program[program.len() - 1] == T::from(opcodes::NOOP) {
            program.len()
        }
        else {
            program.len().next_power_of_two() * 2
        }
    }
    else {
        program.len().next_power_of_two()
    };
    program.resize(cmp::max(trace_length, MIN_TRACE_LENGTH), T::from(opcodes::NOOP));
}