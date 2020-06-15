use log::debug;
use std::time::Instant;

#[cfg(test)]
mod tests;

// RE-EXPORTS
// ================================================================================================
pub mod crypto;
pub mod math;
pub mod utils;

mod stark;
pub use stark::{ StarkProof, ProofOptions };

mod processor;
pub use processor::{ Program, ProgramInputs, opcodes, assembly };

// EXECUTOR
// ================================================================================================

/// Executes the specified `program` and returns the result together with program hash
/// and STARK-based proof of execution.
/// 
/// * `inputs` specifies the initial stack state and provides secret input tapes;
/// * `num_outputs` specifies the number of elements from the top of the stack to be returned;
pub fn execute(program: &Program, inputs: &ProgramInputs<u128>, num_outputs: usize, options: &ProofOptions) -> (Vec<u128>, [u8; 32], StarkProof<u128>)
{
    assert!(num_outputs <= stark::MAX_OUTPUTS, 
        "cannot produce more than {} outputs, but requested {}", stark::MAX_OUTPUTS, num_outputs);

    // execute the program to create an execution trace
    let now = Instant::now();
    let register_traces = processor::execute(program, inputs);
    let mut trace = stark::TraceTable::new2(register_traces, options.extension_factor());
    debug!("Generated execution trace of {} registers and {} steps in {} ms",
        trace.register_count(),
        trace.unextended_length(),
        now.elapsed().as_millis());

    // copy the user stack state the the last step to return as output
    let last_state = trace.get_state(trace.unextended_length() - 1);
    let outputs = last_state.get_user_stack()[0..num_outputs].to_vec();

    // construct program hash
    let mut program_hash = [0u8; 32];
    program_hash.copy_from_slice(utils::as_bytes(&trace.get_program_hash()));

    // generate STARK proof
    let proof = stark::prove(&mut trace, inputs.get_public_inputs(), &outputs, options);
    return (outputs, program_hash, proof);
}

// VERIFIER
// ================================================================================================

/// Verifies that if a program with the specified `program_hash` is executed with the 
/// provided `public_inputs` and some secret inputs, the result is equal to the `outputs`.
pub fn verify(program_hash: &[u8; 32], public_inputs: &[u128], outputs: &[u128], proof: &StarkProof<u128>) -> Result<bool, String>
{
    return stark::verify(program_hash, public_inputs, outputs, proof);
}