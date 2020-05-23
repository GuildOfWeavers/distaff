use std::mem;
use rand::prelude::*;
use rand::distributions::Uniform;
use crate::math::{ FiniteField };
use super::{ ProofOptions, MAX_CONSTRAINT_DEGREE };

// RE-EXPORTS
// ================================================================================================
mod accumulator;
pub use accumulator::{ Accumulator };

mod coefficients;
pub use coefficients::{ ConstraintCoefficients, CompositionCoefficients };

mod proof_of_work;
pub use proof_of_work::{ find_pow_nonce, verify_pow_nonce };

pub fn get_composition_degree(trace_length: usize) -> usize {
    return (MAX_CONSTRAINT_DEGREE - 1) * trace_length - 1;
}

// PUBLIC FUNCTIONS
// ================================================================================================

pub fn get_incremental_trace_degree(trace_length: usize) -> usize {
    let composition_degree = get_composition_degree(trace_length);
    return composition_degree - (trace_length - 2);
}

pub fn compute_query_positions(seed: &[u8; 32], domain_size: usize, options: &ProofOptions) -> Vec<usize> {
    let range = Uniform::from(0..domain_size);
    let mut index_iter = StdRng::from_seed(*seed).sample_iter(range);
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

pub fn map_trace_to_constraint_positions<T: FiniteField>(positions: &[usize]) -> Vec<usize> {
    let element_size = mem::size_of::<T>();
    let elements_per_leaf = 32 / element_size;
    let mut result = Vec::with_capacity(positions.len());
    for &position in positions.iter() {
        let cp = position / elements_per_leaf;
        if !result.contains(&cp) { result.push(cp); }
    }
    return result;
}