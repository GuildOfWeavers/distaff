use rand::prelude::*;
use rand::distributions::Uniform;
use crate::stark::ProofOptions;

// TYPES AND INTERFACES
// ================================================================================================
pub struct QueryIndexGenerator {
    extension_factor    : usize,
    trace_query_count   : usize,
    fri_query_count     : usize,
}

// QUERY INDEX GENERATOR IMPLEMENTATION
// ================================================================================================
impl QueryIndexGenerator {

    pub fn new(options: &ProofOptions) -> QueryIndexGenerator {
        return QueryIndexGenerator {
            extension_factor    : options.extension_factor(),
            trace_query_count   : options.trace_query_count(),
            fri_query_count     : options.fri_query_count(),
        };
    }

    pub fn get_trace_indexes(&self, seed: &[u64; 4], max_index: usize) -> Vec<usize> {
        return self.generate_indexes(seed, self.trace_query_count, max_index);
    }

    pub fn get_fri_indexes(&self, seed: &[u64; 4], max_index: usize) -> Vec<usize> {
        return self.generate_indexes(seed, self.trace_query_count, max_index);
    }

    fn generate_indexes(&self, seed: &[u64; 4], num_indexes: usize, max_index: usize) -> Vec<usize> {

        let seed = unsafe { *(seed as *const _ as *const [u8; 32]) };
        let range = Uniform::from(0..max_index);
        let mut index_iter = StdRng::from_seed(seed).sample_iter(range);

        let mut result = Vec::new();
        for _ in 0..1000 {
            let value = index_iter.next().unwrap();
            if value % self.extension_factor != 0 { continue; }
            if result.contains(&value) { continue; }
            result.push(value);
            if result.len() >= num_indexes { break; }
        }
        return result;
    }
}