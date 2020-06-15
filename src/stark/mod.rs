use std::ops::Range;

mod trace;
mod constraints;
mod options;
mod prover;
mod verifier;
mod proof;
mod fri;
mod utils;

pub use trace::{ TraceTable, TraceState };

pub use constraints::{
    ConstraintEvaluator,
    ConstraintTable,
    ConstraintPoly,
    MAX_CONSTRAINT_DEGREE,
    MAX_TRANSITION_CONSTRAINTS };

pub use utils::{
    ConstraintCoefficients,
    CompositionCoefficients };

pub use options::ProofOptions;
pub use proof::{ StarkProof, DeepValues };
pub use prover::{ prove };
pub use verifier::{ verify };

// GENERAL CONSTANTS
// ------------------------------------------------------------------------------------------------
pub const MIN_TRACE_LENGTH  : usize = 16;
pub const MAX_REGISTER_COUNT: usize = 128;

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

// DECODER TRACE
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

// STACK TRACE
// ------------------------------------------------------------------------------------------------
//
//   aux ╒════════════════ user registers ═════════════════════╕
//    0      1    2    .................................    31
// ├─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┤

pub const MAX_PUBLIC_INPUTS : usize = 8;
pub const MAX_OUTPUTS       : usize = 8;
const MIN_STACK_DEPTH       : usize = 9;
const MAX_STACK_DEPTH       : usize = 32;