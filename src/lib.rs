use log::debug;
use std::ops::Range;
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
    assert!(num_outputs <= MAX_OUTPUTS, 
        "cannot produce more than {} outputs, but requested {}", MAX_OUTPUTS, num_outputs);

    // execute the program to create an execution trace
    let now = Instant::now();
    let register_traces = processor::execute(program, inputs);
    let mut trace = stark::TraceTable::new(register_traces, options.extension_factor());
    debug!("Generated execution trace of {} registers and {} steps in {} ms",
        trace.register_count(),
        trace.unextended_length(),
        now.elapsed().as_millis());

    // copy the user stack state the the last step to return as output
    let last_state = trace.get_state(trace.unextended_length() - 1);
    let outputs = last_state.get_user_stack()[..num_outputs].to_vec();

    // construct program hash
    let mut program_hash = [0u8; 32];
    program_hash.copy_from_slice(utils::as_bytes(&trace.get_program_hash()));

    // generate STARK proof
    let mut proof = stark::prove(&mut trace, inputs.get_public_inputs(), &outputs, options);

    // build Merkle authentication path for the program execution path and attach it to the proof
    let mut execution_path_hash = [0u128; ACC_STATE_RATE];
    execution_path_hash.copy_from_slice(&trace.get_program_hash());
    let (auth_path_index, auth_path) = program.get_auth_path(&execution_path_hash);
    proof.set_auth_path(auth_path, auth_path_index);

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

// GLOBAL CONSTANTS
// ================================================================================================

const MIN_TRACE_LENGTH      : usize = 16;
const MAX_REGISTER_COUNT    : usize = 128;
const MIN_EXTENSION_FACTOR  : usize = 16;

// HASH OPERATION
// ------------------------------------------------------------------------------------------------
const HASH_STATE_RATE       : usize = 4;
const HASH_STATE_CAPACITY   : usize = 2;
const HASH_STATE_WIDTH      : usize = HASH_STATE_RATE + HASH_STATE_CAPACITY;
const HASH_CYCLE_LENGTH     : usize = 16;

// HASH ACCUMULATOR
// ------------------------------------------------------------------------------------------------
const ACC_STATE_RATE        : usize = 2;
const ACC_STATE_CAPACITY    : usize = 2;
const ACC_STATE_WIDTH       : usize = ACC_STATE_RATE + ACC_STATE_CAPACITY;
const ACC_CYCLE_LENGTH      : usize = 16;

// DECODER LAYOUT
// ------------------------------------------------------------------------------------------------
//
//   op  ╒═════════ op_bits ═══════════╕╒══════ op_acc ════════╕
//    0      1    2     3     4     5     6     7     8     9
// ├─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┤

const NUM_OP_BITS           : usize = 5;
const NUM_LD_OPS            : usize = 32;

const DECODER_WIDTH         : usize = 1 + NUM_OP_BITS + ACC_STATE_WIDTH;

const OP_CODE_INDEX         : usize = 0;
const OP_BITS_RANGE         : Range<usize> = Range { start: 1, end: 6 };
const OP_ACC_RANGE          : Range<usize> = Range { start: 6, end: 6 + ACC_STATE_WIDTH };
const PROG_HASH_RANGE       : Range<usize> = Range { start: 6, end: 6 + ACC_STATE_RATE  };

// STACK LAYOUT
// ------------------------------------------------------------------------------------------------
//
//   aux ╒════════════════ user registers ═════════════════════╕
//    0      1    2    .................................    31
// ├─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┤

pub const MAX_PUBLIC_INPUTS : usize = 8;
pub const MAX_OUTPUTS       : usize = 8;
const MIN_STACK_DEPTH       : usize = 9;
const MAX_STACK_DEPTH       : usize = 32;