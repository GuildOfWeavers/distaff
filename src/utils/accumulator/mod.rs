use std::mem;
use std::slice;
use crate::math::{ FiniteField };

mod acc64;
mod acc128;

// TYPES AND INTERFACES
// ================================================================================================
pub trait Accumulator: FiniteField {

    const NUM_ROUNDS    : usize;
    const STATE_WIDTH   : usize;
    const DIGEST_SIZE   : usize;

    /// Hashes a list of u64 values into a single 32-byte value using a modified version of 
    /// [Rescue](https://eprint.iacr.org/2019/426) hash function. The modifications are:
    /// 
    /// 1. First and last steps of the permutation are removed to make the permutation fully-foldable.
    /// 2. A single round is executed for each value in the list with constants cycling every 16 
    /// rounds. This means that a new value is injected into the state at the beginning of each round.
    /// 
    /// The last modification differs significantly form how the function was originally designed,
    /// and likely compromises security.
    fn digest(values: &[Self]) -> [u8; 32] {
        let mut state = vec![Self::ZERO; Self::STATE_WIDTH];
        for i in 0..values.len() {
            Self::apply_round(&mut state, values[i], i);
        }

        let element_size = mem::size_of::<Self>();
        debug_assert!(element_size * Self::DIGEST_SIZE == 32, "digest size must be 32 bytes");
        let state_slice = &state[0..Self::DIGEST_SIZE];
        let state_slice = unsafe {  slice::from_raw_parts(state_slice.as_ptr() as *const u8, 32) };

        let mut result = [0u8; 32];
        result.copy_from_slice(state_slice);

        return result;
    }

    fn apply_round(state: &mut [Self], value: Self, step: usize) {
        
        let ark_idx = step % Self::NUM_ROUNDS;

        // apply first half of Rescue round
        Self::add_constants(state, ark_idx, 0);
        Self::apply_sbox(state);
        Self::apply_mds(state);

        // inject value into the state
        state[0] = Self::add(state[0], Self::mul(state[2], value));
        state[1] = Self::mul(state[1], Self::add(state[3], value));

        // apply second half of Rescue round
        Self::add_constants(state, ark_idx, Self::STATE_WIDTH);
        Self::apply_inv_sbox(state);
        Self::apply_mds(state);
    }

    fn add_constants(state: &mut[Self], idx: usize, offset: usize);

    fn apply_sbox(state: &mut [Self]);
    fn apply_inv_sbox(state: &mut[Self]);

    fn apply_mds(state: &mut[Self]);
    fn apply_inv_mds(state: &mut[Self]);

    fn get_extended_constants(extension_factor: usize) -> (Vec<Vec<Self>>, Vec<Vec<Self>>);
}