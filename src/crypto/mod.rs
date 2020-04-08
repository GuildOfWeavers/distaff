pub mod hash;
pub use hash::*;

pub mod merkle;
pub use merkle::*;

pub type HashFunction = fn(&[u64], &mut [u64]);