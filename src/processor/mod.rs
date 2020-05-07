use log::debug;
use std::cmp;
use std::time::Instant;
use crate::stark::{ self, ProofOptions, StarkProof, MAX_INPUTS, MAX_OUTPUTS, MIN_TRACE_LENGTH };
use crate::utils::CopyInto;

mod tests;
pub mod opcodes;

/// Executes the specified `program` and returns the result together with program hash
/// and STARK-based proof of execution.
/// 
/// * `inputs` specify the initial stack state the with inputs[0] being the top of the stack;
/// * `num_outputs` specifies the number of elements from the top of the stack to be returned;
pub fn execute(program: &[u64], inputs: &[u64], num_outputs: usize, options: &ProofOptions) -> (Vec<u64>, [u8; 32], StarkProof) {

    assert!(inputs.len() <= MAX_INPUTS,
        "Expected no more than {} inputs, but received {}", MAX_INPUTS, inputs.len());
    assert!(num_outputs <= MAX_OUTPUTS, 
        "Cannot produce more than {} outputs, but requested {}", MAX_OUTPUTS, num_outputs);

    // pad the program with the appropriate number of NOOPs
    let program = pad_program(program);

    // execute the program to create an execution trace
    let now = Instant::now();
    let mut trace = stark::TraceTable::new(&program, inputs, options.extension_factor());
    debug!("Generated execution trace of {} registers and {} steps in {} ms",
        trace.register_count(),
        trace.unextended_length(),
        now.elapsed().as_millis());

    // copy the stack state the the last step to return as output
    let last_state = trace.get_state(trace.unextended_length() - 1);
    let outputs = last_state.get_stack()[0..num_outputs].to_vec();
    
    // generate STARK proof
    let proof = stark::prove(&mut trace, inputs, &outputs, options);
    return (outputs, trace.get_program_hash().copy_into(), proof);
}

/// Verifies that if a program with the specified `program_hash` is executed with the 
/// provided `inputs`, the result is equal to the `outputs`.
pub fn verify(program_hash: &[u8; 32], inputs: &[u64], outputs: &[u64], proof: &StarkProof) -> Result<bool, String> {

    return stark::verify(program_hash, inputs, outputs, proof);
}

/// Pads the program with the appropriate number of NOOPs to ensure that:
/// 1. The length of the program is at least 16;
/// 2. The length of the program is a power of 2;
/// 3. The program terminates with a NOOP.
pub fn pad_program(program: &[u64]) -> Vec<u64> {
    let mut program = program.to_vec();
    let trace_length = if program.len() == program.len().next_power_of_two() {
        if program[program.len() - 1] == opcodes::NOOP {
            program.len()
        }
        else {
            program.len().next_power_of_two() * 2
        }
    }
    else {
        program.len().next_power_of_two()
    };
    program.resize(cmp::max(trace_length, MIN_TRACE_LENGTH), opcodes::NOOP);
    return program;
}