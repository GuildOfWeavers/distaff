// CONSTANTS
// ================================================================================================
const DEFAULT_EXTENSION_FACTOR: usize  = 32;
const DEFAULT_TRACE_QUERY_COUNT: usize = 32;
const DEFAULT_FRI_QUERY_COUNT: usize   = 48;

// TYPES AND INTERFACES
// ================================================================================================
pub struct ProofOptions {
    extension_factor    : usize,
    trace_query_count   : usize,
    fri_query_count     : usize,
}

// PROOF OPTIONS IMPLEMENTATION
// ================================================================================================
impl ProofOptions {

    pub fn new(extension_factor: usize, trace_query_count: usize, fri_query_count: usize) -> ProofOptions {

        assert!(extension_factor.is_power_of_two(), "extension_factor must be a power of 2");
        assert!(extension_factor >= 16, "extension_factor cannot be smaller than 16");
        assert!(extension_factor <= 128, "extension_factor cannot be greater than 128");

        assert!(trace_query_count > 0, "trace_query_count must be greater than 0");
        assert!(trace_query_count <= 128, "trace_query_count cannot be greater than 128");

        assert!(fri_query_count > 0, "fri_query_count must be greater than 0");
        assert!(fri_query_count <= 128, "fri_query_count cannot be greater than 128");

        return ProofOptions { extension_factor, trace_query_count, fri_query_count };
    }

    pub fn default() -> ProofOptions {
        return ProofOptions {
            extension_factor    : DEFAULT_EXTENSION_FACTOR,
            trace_query_count   : DEFAULT_TRACE_QUERY_COUNT,
            fri_query_count     : DEFAULT_FRI_QUERY_COUNT
        };
    }

    pub fn extension_factor(&self) -> usize {
        return self.extension_factor;
    }

    pub fn trace_query_count(&self) -> usize {
        return self.trace_query_count;
    }

    pub fn fri_query_count(&self) -> usize {
        return self.fri_query_count;
    }
}