use std::mem;
use std::slice;
use crate::math::{ FiniteField, polynom };

mod acc64;
mod acc128;

// TYPES AND INTERFACES
// ================================================================================================
pub trait AccumulatorBuilder<T>
    where T: FiniteField
{
    const ACC_NUM_ROUNDS    : usize;
    const ACC_STATE_WIDTH   : usize;
    const ACC_DIGEST_SIZE   : usize;

    fn get_accumulator(extension_factor: usize) -> Accumulator<T>;
}

pub struct Accumulator<T> {
    alpha       : T,
    inv_alpha   : T,
    mds         : Vec<T>,
    inv_mds     : Vec<T>,
    ark         : Vec<Vec<T>>,
    ark_polys   : Vec<Vec<T>>,
}

// ACCUMULATOR IMPLEMENTATION
// ================================================================================================
impl <T> Accumulator<T> 
    where T: FiniteField + AccumulatorBuilder<T>
{
    /// Hashes a list of u64 values into a single 32-byte value using a modified version of 
    /// [Rescue](https://eprint.iacr.org/2019/426) hash function. The modifications are:
    /// 
    /// 1. First and last steps of the permutation are removed to make the permutation fully-foldable.
    /// 2. A single round is executed for each value in the list with constants cycling every 16 
    /// rounds. This means that a new value is injected into the state at the beginning of each round.
    /// 
    /// The last modification differs significantly form how the function was originally designed,
    /// and likely compromises security.
    pub fn digest(&self, values: &[T]) -> [u8; 32] {
        let mut state = vec![T::ZERO; T::ACC_STATE_WIDTH];
        for i in 0..values.len() {
            self.apply_round(&mut state, values[i], i);
        }

        let element_size = mem::size_of::<T>();
        assert!(element_size * T::ACC_DIGEST_SIZE == 32, "digest size must be 32 bytes");
        let state_slice = &state[0..T::ACC_DIGEST_SIZE];
        let state_slice = unsafe {  slice::from_raw_parts(state_slice.as_ptr() as *const u8, 32) };

        let mut result = [0u8; 32];
        result.copy_from_slice(state_slice);

        return result;
    }

    pub fn apply_round(&self, state: &mut [T], value: T, step: usize) {
        
        // inject value into the state
        state[0] = T::add(state[0], value);
        state[1] = T::mul(state[1], value);

        let ark = self.get_constants_at(step);

        // apply Rescue round
        for i in 0..T::ACC_STATE_WIDTH {
            state[i] = T::add(state[i], ark[i]);
        }
        self.apply_sbox(state);
        self.apply_mds(state);

        for i in 0..T::ACC_STATE_WIDTH {
            state[i] = T::add(state[i], ark[i + T::ACC_STATE_WIDTH]);
        }
        self.apply_inv_sbox(state);
        self.apply_mds(state);
    }

    pub fn get_constants_at(&self, step: usize) -> &Vec<T> {
        return &self.ark[step % self.ark.len()];
    }

    pub fn evaluate_constants_at(&self, x: T) -> Vec<T> {
        let mut result = Vec::with_capacity(self.ark_polys.len());
        for i in 0..self.ark_polys.len() {
            result.push(polynom::eval(&self.ark_polys[i], x));
        }
        return result;
    }

    pub fn apply_sbox(&self, state: &mut [T]) {
        for i in 0..T::ACC_STATE_WIDTH {
            state[i] = T::exp(state[i], self.alpha);
        }
    }

    pub fn apply_inv_sbox(&self, state: &mut[T]) {
        // TODO: optimize
        for i in 0..T::ACC_STATE_WIDTH {
            state[i] = T::exp(state[i], self.inv_alpha);
        }
    }

    pub fn apply_mds(&self, state: &mut[T]) {
        let mut result = vec![T::ZERO; T::ACC_STATE_WIDTH];
        let mut temp = vec![T::ZERO; T::ACC_STATE_WIDTH];
        for i in 0..T::ACC_STATE_WIDTH {
            for j in 0..T::ACC_STATE_WIDTH {
                temp[j] = T::mul(self.mds[i * T::ACC_STATE_WIDTH + j], state[j]);
            }
    
            for j in 0..T::ACC_STATE_WIDTH {
                result[i] = T::add(result[i], temp[j]);
            }
        }
        state.copy_from_slice(&result);
    }

    pub fn apply_inv_mds(&self, state: &mut[T]) {
        let mut result = vec![T::ZERO; T::ACC_STATE_WIDTH];
        let mut temp = vec![T::ZERO; T::ACC_STATE_WIDTH];
        for i in 0..T::ACC_STATE_WIDTH {
            for j in 0..T::ACC_STATE_WIDTH {
                temp[j] = T::mul(self.inv_mds[i * T::ACC_STATE_WIDTH + j], state[j]);
            }
    
            for j in 0..T::ACC_STATE_WIDTH {
                result[i] = T::add(result[i], temp[j]);
            }
        }
        state.copy_from_slice(&result);
    }
}