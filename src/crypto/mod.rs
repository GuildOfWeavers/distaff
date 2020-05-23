pub mod hash;

mod merkle;
pub use merkle::{ MerkleTree, BatchMerkleProof };

pub type HashFunction = fn(&[u8], &mut [u8]);