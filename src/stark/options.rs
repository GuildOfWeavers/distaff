use serde::{ Serializer, Deserializer, Serialize, Deserialize, ser, de };
use crate::crypto::{ HashFunction, hash };

// CONSTANTS
// ================================================================================================
const DEFAULT_EXTENSION_FACTOR: u8  = 32;
const DEFAULT_TRACE_QUERY_COUNT: u8 = 48;
const DEFAULT_FRI_QUERY_COUNT: u8   = 32;

// TYPES AND INTERFACES
// ================================================================================================
#[derive(Serialize, Deserialize)]
pub struct ProofOptions {
    extension_factor    : u8,
    trace_query_count   : u8,
    fri_query_count     : u8,

    #[serde(serialize_with = "hash_function_serialize")]
    #[serde(deserialize_with = "hash_function_deserialize")]
    hash_function: HashFunction,
}

// PROOF OPTIONS IMPLEMENTATION
// ================================================================================================
impl ProofOptions {

    pub fn new(
        extension_factor : usize,
        trace_query_count: usize,
        fri_query_count  : usize,
        hash_function    : HashFunction) -> ProofOptions
    {

        assert!(extension_factor.is_power_of_two(), "extension_factor must be a power of 2");
        assert!(extension_factor >= 16, "extension_factor cannot be smaller than 16");
        assert!(extension_factor <= 128, "extension_factor cannot be greater than 128");

        assert!(trace_query_count > 0, "trace_query_count must be greater than 0");
        assert!(trace_query_count <= 128, "trace_query_count cannot be greater than 128");

        assert!(fri_query_count > 0, "fri_query_count must be greater than 0");
        assert!(fri_query_count <= 128, "fri_query_count cannot be greater than 128");

        return ProofOptions {
            extension_factor    : extension_factor as u8,
            trace_query_count   : trace_query_count as u8,
            fri_query_count     : fri_query_count as u8,
            hash_function
        };
    }

    pub fn extension_factor(&self) -> usize {
        return self.extension_factor as usize;
    }

    pub fn trace_query_count(&self) -> usize {
        return self.trace_query_count as usize;
    }

    pub fn fri_query_count(&self) -> usize {
        return self.fri_query_count as usize;
    }

    pub fn hash_function(&self) -> HashFunction {
        return self.hash_function;
    }
}

impl Default for ProofOptions {

    fn default() -> ProofOptions {
        return ProofOptions {
            extension_factor    : DEFAULT_EXTENSION_FACTOR,
            trace_query_count   : DEFAULT_TRACE_QUERY_COUNT,
            fri_query_count     : DEFAULT_FRI_QUERY_COUNT,
            hash_function       : hash::blake3
        };
    }

}

// HELPER FUNCTIONS
// ================================================================================================
fn hash_function_serialize<S>(hf: &HashFunction, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer
{
    match *hf as usize {
        f if f == hash::blake3 as usize => s.serialize_u8(0),
        _ => Err(ser::Error::custom("unsupported hash function"))?
    }
}

fn hash_function_deserialize<'de, D>(deserializer: D) -> Result<HashFunction, D::Error>
where
    D: Deserializer<'de>
{
    //let hf_value: u8 = Deserialize::deserialize(deserializer)?;
    match Deserialize::deserialize(deserializer)? {
        0u8 => Ok(hash::blake3),
        _ => Err(de::Error::custom("unsupported hash function"))
    }
}