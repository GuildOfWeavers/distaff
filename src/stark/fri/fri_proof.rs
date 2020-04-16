use crate::crypto::{ BatchMerkleProof };

pub struct FriProof {
    pub ev_root     : [u64; 4],
    pub ev_proof    : BatchMerkleProof,
    pub layers      : Vec<FriLayer>,
    pub remainder   : Vec<u64>,
}

pub struct FriLayer {
    pub column_root : [u64; 4],
    pub column_proof: BatchMerkleProof,
    pub poly_proof  : BatchMerkleProof,
}

impl FriProof {

    pub fn new(ev_root: &[u64; 4], ev_proof: BatchMerkleProof) -> FriProof {
        return FriProof {
            ev_root     : *ev_root,
            ev_proof    : ev_proof,
            layers      : Vec::new(),
            remainder   : Vec::new(),
        };
    }

}