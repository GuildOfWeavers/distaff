// RE-EXPORTS
// ================================================================================================
pub mod crypto;
pub mod math;
pub mod utils;

mod stark;
pub use stark::{ StarkProof, ProofOptions };

mod processor;
pub use processor::{ Program, ProgramInputs, opcodes, assembly, execute, verify };