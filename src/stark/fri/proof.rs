use serde::{ Serialize, Deserialize };
use crate::crypto::{ BatchMerkleProof };

// TYPES AND INTERFACES
// ================================================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriProof {
    pub ev_root     : [u64; 4],
    pub ev_proof    : BatchMerkleProof,
    pub layers      : Vec<FriLayer>,
    pub remainder   : Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriLayer {
    pub column_root : [u64; 4],
    pub column_proof: BatchMerkleProof,
    pub poly_proof  : BatchMerkleProof,
}

// FRI PROOF IMPLEMENTATION
// ================================================================================================
impl FriProof {

    pub fn new(ev_root: &[u64; 4], ev_proof: BatchMerkleProof) -> FriProof {
        return FriProof {
            ev_root     : *ev_root,
            ev_proof    : ev_proof,
            layers      : Vec::new(),
            remainder   : Vec::new(),
        };
    }

    pub fn ev_root(&self) -> &[u64; 4] {
        return &self.ev_root;
    }
}