use log::debug;
use std::{ cmp, time::Instant };
use crate::math::{ F128, FiniteField };
use crate::stark::{ self, ProofOptions, StarkProof, ProgramInputs, MAX_OUTPUTS, MIN_TRACE_LENGTH };
use crate::utils::{ as_bytes };

pub mod opcodes;
pub mod program;
pub mod assembly;

#[cfg(test)]
mod tests;

// TODO: transforming execute() into a fully generic version results in about 10% - 15% runtime
// penalty (mostly in running FFT). So, keeping it non-generic for now.

/// Executes the specified `program` and returns the result together with program hash
/// and STARK-based proof of execution.
/// 
/// * `inputs` specify the initial stack state the with inputs[0] being the top of the stack;
/// * `num_outputs` specifies the number of elements from the top of the stack to be returned;
pub fn execute(program: &[F128], inputs: &ProgramInputs<F128>, num_outputs: usize, options: &ProofOptions) -> (Vec<F128>, [u8; 32], StarkProof<F128>)
{
    assert!(program.len() > 1,
        "expected a program with at last two operations, but received {}", program.len());
    assert!(program[0] == F128::from(opcodes::BEGIN),
        "a program must start with BEGIN operation");
    assert!(num_outputs <= MAX_OUTPUTS, 
        "cannot produce more than {} outputs, but requested {}", MAX_OUTPUTS, num_outputs);

    // pad the program with the appropriate number of NOOPs
    let mut program = program.to_vec();
    pad_program(&mut program);

    // execute the program to create an execution trace
    let now = Instant::now();
    let mut trace = stark::TraceTable::new(&program, inputs, options.extension_factor());
    debug!("Generated execution trace of {} registers and {} steps in {} ms",
        trace.register_count(),
        trace.unextended_length(),
        now.elapsed().as_millis());

    // copy the user stack state the the last step to return as output
    let last_state = trace.get_state(trace.unextended_length() - 1);
    let outputs = last_state.get_user_stack()[0..num_outputs].to_vec();

    // construct program hash
    let mut program_hash = [0u8; 32];
    program_hash.copy_from_slice(as_bytes(&trace.get_program_hash()));

    // generate STARK proof
    let proof = stark::prove(&mut trace, inputs.get_public_inputs(), &outputs, options);
    return (outputs, program_hash, proof);
}

/// Verifies that if a program with the specified `program_hash` is executed with the 
/// provided `public_inputs` and some secret inputs, the result is equal to the `outputs`.
pub fn verify(program_hash: &[u8; 32], public_inputs: &[F128], outputs: &[F128], proof: &StarkProof<F128>) -> Result<bool, String>
{
    return stark::verify(program_hash, public_inputs, outputs, proof);
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

/// Returns a hash value of the program.
pub fn hash_program<T: stark::Accumulator>(program: &[T]) -> [u8; 32] {
    assert!(program.len().is_power_of_two(), "program length must be a power of 2");
    assert!(program.len() >= MIN_TRACE_LENGTH, "program must consist of at least {} operations", MIN_TRACE_LENGTH);
    assert!(program[0] == T::from(opcodes::BEGIN), "program must start with BEGIN operation");
    assert!(program[program.len() - 1] == T::from(opcodes::NOOP), "program must end with NOOP operation");
    return T::digest(&program[..(program.len() - 1)]);
}
