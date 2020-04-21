use crate::math::{ field::{ self, sub, add, mul }, polys, fft };
use crate::utils::zero_filled_vector;
use crate::stark::{ TraceState };
use crate::stark::utils::hash_acc::{
    apply_mds, apply_sbox, apply_inv_mds, STATE_WIDTH, NUM_ROUNDS, ARK
};

/// Degree of hash accumulator constraints.
pub const CONSTRAINT_DEGREES: [usize; STATE_WIDTH] = [6; STATE_WIDTH];

// TYPES AND INTERFACES
// ================================================================================================
pub struct Decoder {
    rescue_ark  : Vec<[u64; 2 * STATE_WIDTH]>,
    hash_cycle  : usize,
}

// DECODER IMPLEMENTATION
// ================================================================================================
impl Decoder {

    pub fn new(extension_factor: usize) -> Decoder {
        let extended_constants = extend_constants(extension_factor);
        
        let hash_cycle = NUM_ROUNDS * extension_factor;
        let mut rescue_ark = Vec::with_capacity(hash_cycle);
        for i in 0..(NUM_ROUNDS * extension_factor) {
            rescue_ark.push([0; 2 * STATE_WIDTH]);
            for j in 0..(2 * STATE_WIDTH) {
                rescue_ark[i][j] = extended_constants[j][i];
            }
        }

        return Decoder { rescue_ark, hash_cycle };
    }

    // EVALUATOR FUNCTION
    // --------------------------------------------------------------------------------------------
    pub fn evaluate(&self, current: &TraceState, next: &TraceState, step: usize) -> [u64; STATE_WIDTH] {

        let op_code = current.get_op_code();
        let mut current_acc = [0; STATE_WIDTH];
        current_acc.copy_from_slice(current.get_op_acc());
        let mut next_acc = [0; STATE_WIDTH];
        next_acc.copy_from_slice(next.get_op_acc());
    
        let step = step % self.hash_cycle;

        current_acc[0] = add(current_acc[0], op_code);
        current_acc[1] = mul(current_acc[1], op_code);
        self.add_constants(&mut current_acc, step, 0);
        apply_sbox(&mut current_acc);
        apply_mds(&mut current_acc);
    
        apply_inv_mds(&mut next_acc);
        apply_sbox(&mut next_acc);
        self.sub_constants(&mut next_acc, step, STATE_WIDTH);
    
        for i in 0..STATE_WIDTH {
            next_acc[i] = sub(next_acc[i], current_acc[i]);
        }
    
        return next_acc;
    }

    fn add_constants(&self, state: &mut[u64; STATE_WIDTH], step: usize, offset: usize) {
        for i in 0..STATE_WIDTH {
            state[i] = add(state[i], self.rescue_ark[step][offset + i]);
        }
    }
    
    fn sub_constants(&self, state: &mut[u64; STATE_WIDTH], step: usize, offset: usize) {
        for i in 0..STATE_WIDTH {
            state[i] = sub(state[i], self.rescue_ark[step][offset + i]);
        }
    }
}

// HELPER FUNCTIONS
// ================================================================================================
pub fn extend_constants(extension_factor: usize) -> Vec<Vec<u64>> {
    
    let root = field::get_root_of_unity(NUM_ROUNDS as u64);
    let inv_twiddles = fft::get_inv_twiddles(root, NUM_ROUNDS);

    let domain_size = NUM_ROUNDS * extension_factor;
    let domain_root = field::get_root_of_unity(domain_size as u64);
    let twiddles = fft::get_twiddles(domain_root, domain_size);

    let mut result = Vec::with_capacity(ARK.len());
    for constant in ARK.iter() {
        let mut extended_constant = zero_filled_vector(NUM_ROUNDS, domain_size);
        extended_constant.copy_from_slice(constant);

        polys::interpolate_fft_twiddles(&mut extended_constant, &inv_twiddles, true);
        unsafe { extended_constant.set_len(extended_constant.capacity()); }
        polys::eval_fft_twiddles(&mut extended_constant, &twiddles, true);

        result.push(extended_constant);
    }

    return result;
}