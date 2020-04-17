use serde::{ Serialize, Deserialize };
use crate::crypto::BatchMerkleProof;
use crate::stark::fri::FriProof;

// TYPES AND INTERFACES
// ================================================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarkProof {
    trace_root  : [u64; 4],
    trace_proof : BatchMerkleProof,
    ld_proof    : FriProof,
}

// STARK PROOF IMPLEMENTATION
// ================================================================================================
impl StarkProof {

    pub fn new(trace_root: &[u64; 4], trace_proof: BatchMerkleProof, ld_proof: FriProof) -> StarkProof {
        return StarkProof {
            trace_root  : *trace_root,
            trace_proof : trace_proof,
            ld_proof    : ld_proof
        };
    }

}