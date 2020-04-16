mod trace;
mod constraints;
mod prover;
mod proof;
pub mod fri;
mod utils;

pub use trace::{ TraceTable, TraceState, MIN_STACK_DEPTH, MAX_STACK_DEPTH };
pub use constraints::{ ConstraintTable, MAX_CONSTRAINT_DEGREE };
pub use prover::{ prove };
pub use proof::{ StarkProof };
pub use utils::{ ProofOptions, hash_acc::digest as hash_acc };
pub use utils::QueryIndexGenerator; // TODO: remove