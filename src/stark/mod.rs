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
    ConstraintPoly };

pub use utils::{
    ConstraintCoefficients,
    CompositionCoefficients };

pub use options::ProofOptions;
pub use proof::{ StarkProof, DeepValues };
pub use prover::{ prove };
pub use verifier::{ verify };

const MAX_CONSTRAINT_DEGREE : usize = 8;