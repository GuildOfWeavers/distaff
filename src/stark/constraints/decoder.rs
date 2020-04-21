use crate::math::{ field::{ self, sub, add, mul }, polys, fft };
use crate::processor::{ opcodes };
use crate::utils::zero_filled_vector;
use crate::stark::{ TraceState };
use crate::stark::utils::hash_acc::{
    apply_mds, apply_sbox, apply_inv_mds, STATE_WIDTH, NUM_ROUNDS, ARK
};

// CONSTANTS
// ================================================================================================

const NUM_CONSTRAINTS: usize = STATE_WIDTH + 9;

/// Degree of operation decoder constraints.
pub const CONSTRAINT_DEGREES: [usize; NUM_CONSTRAINTS] = [
    2, 2, 2, 2, 2,      // op_bits are binary
    2,                  // push_flag is binary
    6,                  // push_flag is set after a PUSH operation
    2,                  // push_flag gets reset on the next step
    2,                  // when push_flag = 0, op_bits are a binary decomposition of op_code
    6, 6, 6, 6, 6, 6,   // op_code hash accumulator constraints
    6, 6, 6, 6, 6, 6
];

// TYPES AND INTERFACES
// ================================================================================================
pub struct Decoder {
    rescue_ark  : Vec<[u64; 2 * STATE_WIDTH]>,
    hash_cycle  : usize,
}

// DECODER CONSTRAINT EVALUATOR IMPLEMENTATION
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

    pub fn constraint_count(&self) -> usize {
        return NUM_CONSTRAINTS;
    }

    // EVALUATOR FUNCTION
    // --------------------------------------------------------------------------------------------
    pub fn evaluate(&self, current: &TraceState, next: &TraceState, step: usize, result: &mut [u64]) {

        // 5 constraints, degree 2: op_bits must be binary
        let op_bits = current.get_op_bits();
        for i in 0..5 {
            result[i] = is_binary(op_bits[i]);
        }

        // 1 constraint, degree 2: push_flag must be binary
        result[5] = is_binary(current.get_push_flag());

        // 1 constraint, degree 5: push_flag must be set to 1 after a PUSH operation
        let op_flags = current.get_op_flags();
        result[6] = sub(op_flags[opcodes::PUSH as usize], next.get_push_flag());

        // 1 constraint, degree 2: push_flag cannot be 1 for two consecutive operations
        result[7] = mul(current.get_push_flag(), next.get_push_flag());

        // 1 constraint, degree 2: when push_flag = 0, op_bits must be a binary decomposition
        // of op_code, otherwise all op_bits must be 0 (NOOP)
        let op_bits_value = current.get_op_bits_value();
        let op_code = mul(current.get_op_code(), binary_not(current.get_push_flag()));
        result[8] = sub(op_code, op_bits_value);

        // 12 constraints, degree 6 to hash current op_code
        self.hash_opcode(current, next, step, &mut result[9..]);
    }

    // OP CODE HASHING
    // --------------------------------------------------------------------------------------------

    fn hash_opcode(&self, current: &TraceState, next: &TraceState, step: usize, result: &mut [u64]) {

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
            result[i] = sub(next_acc[i], current_acc[i]);
        }
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

fn is_binary(v: u64) -> u64 {
    return sub(mul(v, v), v);
}

fn binary_not(v: u64) -> u64 {
    return sub(field::ONE, v);
}