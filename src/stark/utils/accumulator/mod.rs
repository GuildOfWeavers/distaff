use crate::math::{ FiniteField, FieldElement, polynom };

mod acc64;
mod acc128;

// TYPES AND INTERFACES
// ================================================================================================
pub trait AccumulatorBuilder<T>
    where T: FieldElement + FiniteField<T>
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
    ark         : Vec<T>,
    ark_polys   : Vec<Vec<T>>,
    hash_cycle  : usize,
}

// ACCUMULATOR IMPLEMENTATION
// ================================================================================================
impl <T> Accumulator<T> 
    where T: FieldElement + FiniteField<T> + AccumulatorBuilder<T>
{
    pub fn digest(&self, values: &[T]) -> Vec<T> {
        let mut state = vec![T::ZERO; T::ACC_STATE_WIDTH];
        for i in 0..values.len() {
            self.apply_round(&mut state, values[i], i);
        }
        return state[0..T::ACC_DIGEST_SIZE].to_vec();
    }

    pub fn apply_round(&self, state: &mut [T], value: T, step: usize) {
        
        // inject value into the state
        state[0] = T::add(state[0], value);
        state[1] = T::mul(state[1], value);

        // apply Rescue round
        self.add_constants(state, step, 0);
        self.apply_sbox(state);
        self.apply_mds(state);

        self.add_constants(state, step, T::ACC_STATE_WIDTH);
        self.apply_inv_sbox(state);
        self.apply_mds(state);
    }

    pub fn add_constants(&self, state: &mut [T], step: usize, offset: usize) {
        let step = step % self.hash_cycle;
        let start = step * T::ACC_STATE_WIDTH * 2 + offset;
        let ark = &self.ark[start..(start + T::ACC_STATE_WIDTH)];
        for i in 0..T::ACC_STATE_WIDTH {
            state[i] = T::add(state[i], ark[i]);
        }
    }

    pub fn get_constants_at(&self, step: usize) -> &[T] {
        let step = step % self.hash_cycle;
        let start = step * T::ACC_STATE_WIDTH * 2;
        return &self.ark[start..(start + 2 * T::ACC_STATE_WIDTH)];
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