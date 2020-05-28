use crate::math::{ FiniteField };

mod hash128;

// TYPES AND INTERFACES
// ================================================================================================
pub trait Hasher: FiniteField {

    const NUM_ROUNDS    : usize;
    const STATE_WIDTH   : usize;

    fn apply_round(state: &mut [Self], step: usize) {
        
        let ark_idx = step % Self::NUM_ROUNDS;

        // apply Rescue round
        Self::add_constants(state, ark_idx, 0);
        Self::apply_sbox(state);
        Self::apply_mds(state);

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