use rand::prelude::*;
use rand::distributions::Uniform;
use crate::utils::{ CopyInto };
use super::{ ProofOptions };

pub mod hash_acc;

mod prng_coefficients;
pub use prng_coefficients::{ ConstraintCoefficients, CompositionCoefficients };

pub fn compute_query_positions(seed: &[u64; 4], domain_size: usize, options: &ProofOptions) -> Vec<usize> {
    let range = Uniform::from(0..domain_size);
    let mut index_iter = StdRng::from_seed(seed.copy_into()).sample_iter(range);
    let num_queries = options.num_queries();

    let mut result = Vec::new();
    for _ in 0..1000 {
        let value = index_iter.next().unwrap();
        if value % options.extension_factor() == 0 { continue; }
        if result.contains(&value) { continue; }
        result.push(value);
        if result.len() >= num_queries { break; }
    }

    if result.len() < num_queries {
        panic!("needed to generate {} query positions, but generated only {}", num_queries, result.len());
    }

    return result;
}