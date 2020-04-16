use crate::crypto::BatchMerkleProof;
use crate::stark::{ ProofOptions, fri::FriProof };

pub struct StarkProof {
    trace_root  : [u64; 4],
    trace_proof : BatchMerkleProof,
    ld_proof    : FriProof,
    options     : ProofOptions,
}