use crate::processor::opcodes;
use crate::utils::zero_filled_vector;
use super::{ NUM_OP_BITS };

// TRACE BUILDER
// ================================================================================================
pub fn process(program: &[u64], extension_factor: usize) -> (Vec<u64>, Vec<u64>, [Vec<u64>; NUM_OP_BITS]) {

    let trace_length = program.len();
    let domain_size = trace_length * extension_factor;

    assert!(trace_length.is_power_of_two(), "trace length must be a power of 2");
    assert!(extension_factor.is_power_of_two(), "trace extension factor must be a power of 2");
    assert!(program[trace_length - 1] == opcodes::NOOP, "last operation of a program must be NOOP");

    // create op_code register and copy program into it
    let mut op_code = zero_filled_vector(trace_length, domain_size);
    op_code.copy_from_slice(program);

    // initialize push_flags and op_bits registers
    let mut push_flag = zero_filled_vector(trace_length, domain_size);
    let mut op_bits: [Vec<u64>; NUM_OP_BITS] = [
        zero_filled_vector(trace_length, domain_size),
        zero_filled_vector(trace_length, domain_size),
        zero_filled_vector(trace_length, domain_size),
        zero_filled_vector(trace_length, domain_size),
        zero_filled_vector(trace_length, domain_size),
    ];

    // populate push_flags and op_bits registers
    let mut i = 0;
    while i < trace_length {
        set_op_bits(&mut op_bits, op_code[i], i);
        push_flag[i] = 0;

        // if the current operation is PUSH, the next operation is a constant to be pushed onto
        // the stack; so, set the push_flag for the next operation to 1 and op_bits to NOOP
        if op_code[i] == opcodes::PUSH {
            i += 1;
            set_op_bits(&mut op_bits, opcodes::NOOP, i);
            push_flag[i] = 1;
        }

        i += 1;
    }

    return (op_code, push_flag, op_bits);
}

// HELPER FUNCTIONS
// ================================================================================================

/// Sets the op_bits registers at the specified `step` to the binary decomposition
/// of the `op_code` parameter.
fn set_op_bits(op_bits: &mut [Vec<u64>; NUM_OP_BITS], op_code: u64, step: usize) {
    for i in 0..op_bits.len() {
        op_bits[i][step] = (op_code >> i) & 1;
    }
}