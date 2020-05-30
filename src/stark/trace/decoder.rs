use crate::processor::opcodes;
use crate::math::{ FiniteField };
use crate::stark::{ NUM_OP_BITS, ACC_STATE_WIDTH, DECODER_WIDTH, utils::Accumulator };
use crate::utils::filled_vector;

// TRACE BUILDER
// ================================================================================================

/// Builds decoder execution trace
pub fn process<T>(program: &[T], extension_factor: usize) -> Vec<Vec<T>>
    where T: FiniteField + Accumulator
{
    let trace_length = program.len();
    let domain_size = trace_length * extension_factor;

    assert!(program.len() > 1, "program length must be greater than 1");
    assert!(program.len().is_power_of_two(), "program length must be a power of 2");
    assert!(program[0] == T::from(opcodes::BEGIN), "first operation of a program must be BEGIN");
    assert!(program[program.len() - 1] == T::from(opcodes::NOOP), "last operation of a program must be NOOP");
    assert!(extension_factor.is_power_of_two(), "trace extension factor must be a power of 2");

    // create op_code register and copy program into it
    let mut op_code = filled_vector(trace_length, domain_size, T::ZERO);
    op_code.copy_from_slice(program);

    // initialize op_bits registers
    let mut op_bits = Vec::with_capacity(NUM_OP_BITS);
    for _ in 0..NUM_OP_BITS {
        op_bits.push(filled_vector(trace_length, domain_size, T::ZERO));
    }

    // populate op_bits registers
    let mut i = 0;
    while i < trace_length {
        set_op_bits(&mut op_bits, op_code[i].as_u8(), i);

        // if the current operation is PUSH, the next operation is a constant to be pushed onto
        // the stack; so, set op_bits for the next operation to NOOP and skip over it
        if op_code[i] == T::from(opcodes::PUSH) {
            i += 1;
            set_op_bits(&mut op_bits, opcodes::NOOP, i);
        }

        i += 1;
    }

    // create op_acc register traces
    let op_acc = hash_program(&op_code, domain_size);

    // move all registers into a single vector
    let mut registers = vec![op_code];
    for register in op_bits.into_iter() { registers.push(register); }
    for register in op_acc.into_iter() { registers.push(register); }

    assert!(registers.len() == DECODER_WIDTH,
        "invalid decoder width: expected: {}, actual: {}", DECODER_WIDTH, registers.len());

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
    let mut registers = Vec::with_capacity(ACC_STATE_WIDTH);
    for _ in 0..ACC_STATE_WIDTH {
        registers.push(filled_vector(trace_length, domain_size, T::ZERO));
    }

    let mut state = vec![T::ZERO; ACC_STATE_WIDTH];
    for i in 0..(op_codes.len() - 1) {

        // add op_code into the accumulator
        T::apply_round(&mut state, op_codes[i], i);

        // copy updated state into registers for the next step
        for j in 0..ACC_STATE_WIDTH {
            registers[j][i + 1] = state[j];
        }
    }

    return registers;
}

/// Sets the op_bits registers at the specified `step` to the binary decomposition
/// of the `op_code` parameter.
fn set_op_bits<T: FiniteField>(op_bits: &mut Vec<Vec<T>>, op_code: u8, step: usize) {
    for i in 0..op_bits.len() {
        op_bits[i][step] = T::from((op_code >> i) & 1);
    }
}