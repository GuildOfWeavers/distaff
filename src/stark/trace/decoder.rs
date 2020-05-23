use std::ops::Range;
use crate::processor::opcodes;
use crate::math::{ FiniteField };
use crate::stark::utils::{ Accumulator };
use crate::utils::filled_vector;
use super::{ NUM_OP_BITS };

// REGISTER POSITION INFO
// ================================================================================================
pub const OP_CODE_IDX     : usize = 0;
pub const PUSH_FLAG_IDX   : usize = 1;
pub const OP_BITS_RANGE   : Range<usize> = Range { start: 2, end: 2 + NUM_OP_BITS };

// TODO: these should be constant functions, but currently not supported

pub fn num_registers<T: Accumulator>() -> usize {
    return 2 + NUM_OP_BITS + T::STATE_WIDTH;
}

pub fn op_acc_range<T: Accumulator>() -> Range<usize> {
    let start = 2 + NUM_OP_BITS;
    let end = start + T::STATE_WIDTH;
    return Range { start, end };
}

pub fn prog_hash_range<T: Accumulator>() -> Range<usize> {
    let start = 2 + NUM_OP_BITS;
    let end = start + T::DIGEST_SIZE;
    return Range { start, end };
}

// TRACE BUILDER
// ================================================================================================

/// Builds decoder execution trace; the trace consists of the following registers:
/// op_code: 1 register
/// push_flag: 1 register
/// op_bits: 5 registers
/// op_acc: 12 registers
pub fn process<T>(program: &[T], extension_factor: usize) -> Vec<Vec<T>>
    where T: FiniteField + Accumulator
{
    let trace_length = program.len();
    let domain_size = trace_length * extension_factor;

    assert!(trace_length.is_power_of_two(), "trace length must be a power of 2");
    assert!(extension_factor.is_power_of_two(), "trace extension factor must be a power of 2");
    assert!(program[trace_length - 1] == T::from(opcodes::NOOP), "last operation of a program must be NOOP");

    // create op_code register and copy program into it
    let mut op_code = filled_vector(trace_length, domain_size, T::ZERO);
    op_code.copy_from_slice(program);

    // initialize push_flags and op_bits registers
    let mut push_flag = filled_vector(trace_length, domain_size, T::ZERO);
    let mut op_bits = vec![
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
    ];

    // populate push_flags and op_bits registers
    let mut i = 0;
    while i < trace_length {
        set_op_bits(&mut op_bits, op_code[i].as_u8(), i);
        push_flag[i] = T::ZERO;

        // if the current operation is PUSH, the next operation is a constant to be pushed onto
        // the stack; so, set the push_flag for the next operation to 1 and op_bits to NOOP
        if op_code[i] == T::from(opcodes::PUSH) {
            i += 1;
            set_op_bits(&mut op_bits, opcodes::NOOP, i);
            push_flag[i] = T::ONE;
        }

        i += 1;
    }

    // create op_acc register traces
    let op_acc = hash_program(&op_code, domain_size);

    // move all registers into a single vector
    let mut registers = vec![op_code, push_flag];
    for register in op_bits.into_iter() { registers.push(register); }
    for register in op_acc.into_iter() { registers.push(register); }

    return registers;
}

// HELPER FUNCTIONS
// ================================================================================================

/// Uses a modified version of Rescue hash function to reduce all op_codes into a single hash value
fn hash_program<T>(op_codes: &[T], domain_size: usize) -> Vec<Vec<T>>
    where T: FiniteField + Accumulator
{
    let trace_length = op_codes.len();

    // allocate space for the registers
    let mut registers = Vec::with_capacity(T::STATE_WIDTH);
    for _ in 0..T::STATE_WIDTH {
        registers.push(filled_vector(trace_length, domain_size, T::ZERO));
    }

    let mut state = vec![T::ZERO; T::STATE_WIDTH];
    for i in 0..(op_codes.len() - 1) {

        // add op_code into the accumulator
        T::apply_round(&mut state, op_codes[i], i);

        // copy updated state into registers for the next step
        for j in 0..T::STATE_WIDTH {
            registers[j][i + 1] = state[j];
        }
    }

    return registers;
}

/// Sets the op_bits registers at the specified `step` to the binary decomposition
/// of the `op_code` parameter.
fn set_op_bits<T>(op_bits: &mut Vec<Vec<T>>, op_code: u8, step: usize)
    where T: FiniteField
{
    for i in 0..op_bits.len() {
        op_bits[i][step] = T::from((op_code >> i) & 1);
    }
}