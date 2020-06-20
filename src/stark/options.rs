use serde::{ Serialize, Deserialize };
use crate::crypto::{ HashFunction, hash };
use super::MAX_CONSTRAINT_DEGREE;

// CONSTANTS
// ================================================================================================
const DEFAULT_EXTENSION_FACTOR: u8 = (MAX_CONSTRAINT_DEGREE * 4) as u8;
const DEFAULT_NUM_QUERIES     : u8 = 50;
const DEFAULT_GRINDING_FACTOR : u8 = 20;

// TYPES AND INTERFACES
// ================================================================================================

// TODO: validate field values on de-serialization
#[derive(Clone, Serialize, Deserialize)]
pub struct ProofOptions {
    extension_factor    : u8,   // stored as power of 2
    num_queries         : u8,
    grinding_factor     : u8,

    #[serde(with = "hash_fn_serialization")]
    hash_fn: HashFunction,
}

// PROOF OPTIONS IMPLEMENTATION
// ================================================================================================
impl ProofOptions {

    pub fn new(
        extension_factor : usize,
        num_queries      : usize,
        grinding_factor  : u32,
        hash_fn          : HashFunction) -> ProofOptions
    {
        assert!(extension_factor.is_power_of_two(), "extension_factor must be a power of 2");
        assert!(extension_factor >= 16, "extension_factor cannot be smaller than 16");
        assert!(extension_factor <= 256, "extension_factor cannot be greater than 256");

        assert!(num_queries > 0, "num_queries must be greater than 0");
        assert!(num_queries <= 128, "num_queries cannot be greater than 128");

        assert!(grinding_factor <= 32, "grinding factor cannot be greater than 32");

        return ProofOptions {
            extension_factor    : extension_factor.trailing_zeros() as u8,
            num_queries         : num_queries as u8,
            grinding_factor     : grinding_factor as u8,
            hash_fn
        };
    }

    pub fn extension_factor(&self) -> usize {
        return 1 << (self.extension_factor as usize)
    }

    pub fn num_queries(&self) -> usize {
        return self.num_queries as usize;
    }

    pub fn grinding_factor(&self) -> u32 {
        return self.grinding_factor as u32;
    }

    pub fn hash_fn(&self) -> HashFunction {
        return self.hash_fn;
    }

    pub fn security_level(&self, optimistic: bool) -> u32 {
        let one_over_rho = (self.extension_factor() / MAX_CONSTRAINT_DEGREE) as u32;
        let security_factor = 31 - one_over_rho.leading_zeros(); // same as log2(one_over_rho)
        let num_queries = if optimistic == true { self.num_queries } else { self.num_queries / 2 };

        let mut result = security_factor * num_queries as u32;
        if result >= 80 {
            result += self.grinding_factor as u32;
        }

        return result;
    }
}

impl Default for ProofOptions {

    fn default() -> ProofOptions {
        return ProofOptions {
            extension_factor: DEFAULT_EXTENSION_FACTOR.trailing_zeros() as u8,
            num_queries     : DEFAULT_NUM_QUERIES,
            grinding_factor : DEFAULT_GRINDING_FACTOR,
            hash_fn         : hash::blake3,
        };
    }

}

// HASH FUNCTION SERIALIZATION / DE-SERIALIZATION
// ================================================================================================
mod hash_fn_serialization {

    use serde::{ Serializer, Deserializer, Deserialize, ser, de };
    use crate::crypto::{ HashFunction, hash };

    pub fn serialize<S>(hf: &HashFunction, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        match *hf as usize {
            f if f == hash::blake3 as usize => s.serialize_u8(0),
            _ => Err(ser::Error::custom("unsupported hash function"))?
        }
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashFunction, D::Error>
    where
        D: Deserializer<'de>
    {
        match Deserialize::deserialize(deserializer)? {
            0u8 => Ok(hash::blake3),
            _ => Err(de::Error::custom("unsupported hash function"))
        }
    }
}