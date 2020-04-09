use crate::math::{ field };
use crate::utils::zero_filled_vector;
use crate::utils::acc_hash::{ add_constants, apply_mds, apply_sbox, apply_inv_sbox, NUM_ROUNDS };

pub use crate::utils::acc_hash::STATE_WIDTH;

// OPERATION ACCUMULATOR
// ================================================================================================
pub fn digest(op_codes: &[u64], extension_factor: usize) -> [Vec<u64>; STATE_WIDTH] {
    
    let trace_length = op_codes.len() + 1;
    let domain_size = trace_length * extension_factor;

    let mut registers = [
        zero_filled_vector(trace_length, domain_size),
        zero_filled_vector(trace_length, domain_size),
        zero_filled_vector(trace_length, domain_size),
        zero_filled_vector(trace_length, domain_size),
        zero_filled_vector(trace_length, domain_size),
        zero_filled_vector(trace_length, domain_size),
        zero_filled_vector(trace_length, domain_size),
        zero_filled_vector(trace_length, domain_size),
        zero_filled_vector(trace_length, domain_size),
        zero_filled_vector(trace_length, domain_size),
        zero_filled_vector(trace_length, domain_size),
        zero_filled_vector(trace_length, domain_size),
    ];

    let mut state = [0; STATE_WIDTH];
    for register in registers.iter_mut() {
        register[0] = 0;
    }

    for i in 0..op_codes.len() {
        // inject op_code into the state
        state[0] = field::add(state[0], op_codes[i]);
        state[1] = field::mul(state[1], op_codes[i]);

        // apply Rescue round
        add_constants(&mut state, i % NUM_ROUNDS, 0);
        apply_sbox(&mut state);
        apply_mds(&mut state);

        add_constants(&mut state, i % NUM_ROUNDS, STATE_WIDTH);
        apply_inv_sbox(&mut state);
        apply_mds(&mut state);

        // copy updated state into registers for the next step
        for j in 0..STATE_WIDTH {
            registers[j][i + 1] = state[j];
        }
    }

    return registers;
}