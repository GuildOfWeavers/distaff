mod trace;
mod constraints;
mod options;
mod prover;
mod verifier;
mod proof;
mod fri;
mod utils;

pub use trace::{
    TraceTable, TraceState,
    MAX_REGISTER_COUNT,
    MIN_STACK_DEPTH,
    MAX_STACK_DEPTH,
    MAX_INPUTS,
    MAX_OUTPUTS,
    MIN_TRACE_LENGTH };

pub use constraints::{
    ConstraintEvaluator,
    ConstraintTable,
    ConstraintPoly,
    MAX_CONSTRAINT_DEGREE,
    MAX_TRANSITION_CONSTRAINTS };

pub use utils::{
    Hasher,
    Accumulator,
    ConstraintCoefficients,
    CompositionCoefficients };

pub use options::ProofOptions;
pub use proof::{ StarkProof, DeepValues };
pub use prover::{ prove };
pub use verifier::{ verify };


const NUM_LD_OPS        : usize = 32;
const STACK_HEAD_SIZE   : usize = 2;

// HASH OPERATION CONSTANTS
// ------------------------------------------------------------------------------------------------
const HASH_STATE_RATE       : usize = 4;
const HASH_STATE_CAPACITY   : usize = 2;
const HASH_STATE_WIDTH      : usize = HASH_STATE_RATE + HASH_STATE_CAPACITY;
const HASH_CYCLE_LENGTH     : usize = 16;

// HASH ACCUMULATOR CONSTANTS
// ------------------------------------------------------------------------------------------------
const ACC_STATE_RATE        : usize = 2;
const ACC_STATE_CAPACITY    : usize = 2;
const ACC_STATE_WIDTH       : usize = ACC_STATE_RATE + ACC_STATE_CAPACITY;
const ACC_CYCLE_LENGTH      : usize = 16;