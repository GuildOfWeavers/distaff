mod trace;
pub use trace::{ TraceTable, TraceState, MIN_STACK_DEPTH };

mod constraints;
pub use constraints::{ ConstraintTable };

mod prover;
pub use prover::{ prove };