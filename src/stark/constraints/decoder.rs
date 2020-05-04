use crate::math::{ field::{ self, sub, add, mul }, polynom, fft };
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
const CONSTRAINT_DEGREES: [usize; NUM_CONSTRAINTS] = [
    2, 2, 2, 2, 2,      // op_bits are binary
    2,                  // push_flag is binary
    5,                  // push_flag is set after a PUSH operation
    2,                  // push_flag gets reset on the next step
    2,                  // when push_flag = 0, op_bits are a binary decomposition of op_code
    6, 6, 6, 6, 6, 6,   // op_code hash accumulator constraints
    6, 6, 6, 6, 6, 6
];

// TYPES AND INTERFACES
// ================================================================================================
pub struct Decoder {
    rescue_ark  : Vec<[u64; 2 * STATE_WIDTH]>,
    rescue_polys: Vec<Vec<u64>>,
    hash_cycle  : usize,
}

// DECODER CONSTRAINT EVALUATOR IMPLEMENTATION
// ================================================================================================
impl Decoder {

    pub fn new(extension_factor: usize) -> Decoder {
        let (rescue_polys, ark_evaluations) = extend_constants(extension_factor);
        
        let hash_cycle = NUM_ROUNDS * extension_factor;
        let mut rescue_ark = Vec::with_capacity(hash_cycle);
        for i in 0..(NUM_ROUNDS * extension_factor) {
            rescue_ark.push([0; 2 * STATE_WIDTH]);
            for j in 0..(2 * STATE_WIDTH) {
                rescue_ark[i][j] = ark_evaluations[j][i];
            }
        }

        return Decoder { rescue_ark, rescue_polys, hash_cycle };
    }

    pub fn constraint_count(&self) -> usize {
        return NUM_CONSTRAINTS;
    }

    pub fn constraint_degrees() -> &'static [usize] {
        return &CONSTRAINT_DEGREES;
    }

    // EVALUATOR FUNCTION
    // --------------------------------------------------------------------------------------------
    pub fn evaluate(&self, current: &TraceState, next: &TraceState, step: usize, result: &mut [u64]) {

        // 9 constraints to decode op_code
        self.decode_opcode(current, next, result);

        // 12 constraints to hash op_code
        self.hash_opcode(current, next, &self.rescue_ark[step % self.hash_cycle], &mut result[9..]);
    }

    pub fn evaluate_at(&self, current: &TraceState, next: &TraceState, x: u64, result: &mut [u64]) {
        // 9 constraints to decode op_code
        self.decode_opcode(current, next, result);

        // 12 constraints to hash op_code
        let mut rescue_ark = [0u64; 2 * STATE_WIDTH];
        for i in 0..rescue_ark.len() {
            rescue_ark[i] = polynom::eval(&self.rescue_polys[i], x);
        }
        self.hash_opcode(current, next, &rescue_ark, &mut result[9..]);
    }

    // EVALUATION HELPERS
    // --------------------------------------------------------------------------------------------
    fn decode_opcode(&self, current: &TraceState, next: &TraceState, result: &mut [u64]) {
        // TODO: degree of expanded op_bits is assumed to be 5, but in reality can be less than 5
        // if opcodes used in the program don't touch some op_bits. Thus, all degrees that assume
        // op_flag values to have degree 5, may be off.

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
    }

    fn hash_opcode(&self, current: &TraceState, next: &TraceState, ark: &[u64; 2 * STATE_WIDTH], result: &mut [u64]) {
        let op_code = current.get_op_code();

        let mut current_acc = [0; STATE_WIDTH];
        current_acc.copy_from_slice(current.get_op_acc());
        let mut next_acc = [0; STATE_WIDTH];
        next_acc.copy_from_slice(next.get_op_acc());

        current_acc[0] = add(current_acc[0], op_code);
        current_acc[1] = mul(current_acc[1], op_code);
        for i in 0..STATE_WIDTH {
            current_acc[i] = add(current_acc[i], ark[i]);
        }
        apply_sbox(&mut current_acc);
        apply_mds(&mut current_acc);
    
        apply_inv_mds(&mut next_acc);
        apply_sbox(&mut next_acc);
        for i in 0..STATE_WIDTH {
            next_acc[i] = sub(next_acc[i], ark[STATE_WIDTH + i]);
        }

        for i in 0..STATE_WIDTH {
            result[i] = sub(next_acc[i], current_acc[i]);
        }
    }
}

// HELPER FUNCTIONS
// ================================================================================================
pub fn extend_constants(extension_factor: usize) -> (Vec<Vec<u64>>, Vec<Vec<u64>>) {
    
    let root = field::get_root_of_unity(NUM_ROUNDS as u64);
    let inv_twiddles = fft::get_inv_twiddles(root, NUM_ROUNDS);

    let domain_size = NUM_ROUNDS * extension_factor;
    let domain_root = field::get_root_of_unity(domain_size as u64);
    let twiddles = fft::get_twiddles(domain_root, domain_size);

    let mut polys = Vec::with_capacity(ARK.len());
    let mut evaluations = Vec::with_capacity(ARK.len());

    for constant in ARK.iter() {
        let mut extended_constant = zero_filled_vector(NUM_ROUNDS, domain_size);
        extended_constant.copy_from_slice(constant);

        polynom::interpolate_fft_twiddles(&mut extended_constant, &inv_twiddles, true);
        polys.push(extended_constant.clone());

        unsafe { extended_constant.set_len(extended_constant.capacity()); }
        polynom::eval_fft_twiddles(&mut extended_constant, &twiddles, true);

        evaluations.push(extended_constant);
    }

    return (polys, evaluations);
}

fn is_binary(v: u64) -> u64 {
    return sub(mul(v, v), v);
}

fn binary_not(v: u64) -> u64 {
    return sub(field::ONE, v);
}