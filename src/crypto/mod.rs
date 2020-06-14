pub mod hash;

mod merkle;
pub use merkle::{ MerkleTree, BatchMerkleProof, build_merkle_nodes };

pub type HashFunction = fn(&[u8], &mut [u8]);