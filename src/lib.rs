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
pub use processor::{ OpCode, OpHint };

mod programs;
pub use programs::{ Program, ProgramInputs, assembly, blocks };

// EXECUTOR
// ================================================================================================

/// Executes the specified `program` and returns the result together with program hash
/// and STARK-based proof of execution.
/// 
/// * `inputs` specifies the initial stack state and provides secret input tapes;
/// * `num_outputs` specifies the number of elements from the top of the stack to be returned;
pub fn execute(program: &Program, inputs: &ProgramInputs, num_outputs: usize, options: &ProofOptions) -> (Vec<u128>, StarkProof)
{
    assert!(num_outputs <= MAX_OUTPUTS, 
        "cannot produce more than {} outputs, but requested {}", MAX_OUTPUTS, num_outputs);

    let proc_index = 0; // TODO

    // execute the program to create an execution trace
    let now = Instant::now();
    let (trace, ctx_depth, loop_depth) = processor::execute(program, proc_index, inputs);
    let mut trace = stark::TraceTable::new(trace, ctx_depth, loop_depth, options.extension_factor());
    debug!("Generated execution trace of {} registers and {} steps in {} ms",
        trace.register_count(),
        trace.unextended_length(),
        now.elapsed().as_millis());

    // copy the user stack state the the last step to return as output
    let last_state = trace.get_last_state();
    let outputs = last_state.user_stack()[..num_outputs].to_vec();

    // make sure number of executed operations was sufficient
    assert!(last_state.op_counter() as usize >= MIN_TRACE_LENGTH,
        "a program must consist of at least {} operation, but only {} were executed",
        MIN_TRACE_LENGTH,
        last_state.op_counter());

    // generate STARK proof
    let mut proof = stark::prove(&mut trace, inputs.get_public_inputs(), &outputs, options);

    // build Merkle authentication path for procedure within the program and attach it to the proof
    let procedure_hash = utils::as_bytes(last_state.program_hash());
    let proc_path = program.get_proc_path(proc_index);
    assert!(proc_path[0] == procedure_hash,
        "expected procedure hash {} does not match trace hash {}",
        hex::encode(proc_path[0]),
        hex::encode(procedure_hash));
    proof.set_proc_path(proc_path, proc_index);

    return (outputs, proof);
}

// VERIFIER
// ================================================================================================

/// Verifies that if a program with the specified `program_hash` is executed with the 
/// provided `public_inputs` and some secret inputs, the result is equal to the `outputs`.
pub fn verify(program_hash: &[u8; 32], public_inputs: &[u128], outputs: &[u128], proof: &StarkProof) -> Result<bool, String>
{
    return stark::verify(program_hash, public_inputs, outputs, proof);
}

// GLOBAL CONSTANTS
// ================================================================================================

pub const MAX_CONTEXT_DEPTH : usize = 16;
pub const MAX_LOOP_DEPTH    : usize = 8;
const MIN_TRACE_LENGTH      : usize = 16;
const MAX_REGISTER_COUNT    : usize = 128;
const MIN_EXTENSION_FACTOR  : usize = 16;
const BASE_CYCLE_LENGTH     : usize = 16;

// PUSH OPERATION
// ------------------------------------------------------------------------------------------------
const PUSH_OP_ALIGNMENT     : usize = 8;

// HASH OPERATION
// ------------------------------------------------------------------------------------------------
const HASH_STATE_RATE       : usize = 4;
const HASH_STATE_CAPACITY   : usize = 2;
const HASH_STATE_WIDTH      : usize = HASH_STATE_RATE + HASH_STATE_CAPACITY;
const HASH_NUM_ROUNDS       : usize = 10;
const HASH_DIGEST_SIZE      : usize = 2;

// OPERATION SPONGE
// ------------------------------------------------------------------------------------------------
const SPONGE_WIDTH          : usize = 4;
const PROGRAM_DIGEST_SIZE   : usize = 2;
const HACC_NUM_ROUNDS       : usize = 14;

// DECODER LAYOUT
// ------------------------------------------------------------------------------------------------
//
//  ctr ╒═════ sponge ══════╕╒═══ cf_ops ══╕╒═══════ ld_ops ═══════╕╒═ hd_ops ╕╒═ ctx ══╕╒═ loop ═╕
//   0    1    2    3    4    5    6    7    8    9    10   11   12   13   14   15   ..   ..   ..
// ├────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┤

const NUM_CF_OP_BITS        : usize = 3;
const NUM_LD_OP_BITS        : usize = 5;
const NUM_HD_OP_BITS        : usize = 2;

const NUM_CF_OPS            : usize = 8;
const NUM_LD_OPS            : usize = 32;
const NUM_HD_OPS            : usize = 4;

const OP_COUNTER_IDX        : usize = 0;
const SPONGE_RANGE          : Range<usize> = Range { start:  1, end:  5 };
const CF_OP_BITS_RANGE      : Range<usize> = Range { start:  5, end:  8 };
const LD_OP_BITS_RANGE      : Range<usize> = Range { start:  8, end: 13 };
const HD_OP_BITS_RANGE      : Range<usize> = Range { start: 13, end: 15 };

// STACK LAYOUT
// ------------------------------------------------------------------------------------------------
//
// ╒═══════════════════ user registers ════════════════════════╕
//    0      1    2    .................................    31
// ├─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┤

pub const MAX_PUBLIC_INPUTS : usize = 8;
pub const MAX_OUTPUTS       : usize = 8;
pub const MAX_STACK_DEPTH   : usize = 32;
const MIN_STACK_DEPTH       : usize = 8;