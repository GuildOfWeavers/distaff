use std::cmp;
use crate::math::{ FiniteField };
use crate::processor::{ ProgramInputs, opcodes};
use crate::stark::{ utils::Hasher };
use crate::stark::{ MIN_STACK_DEPTH, MAX_STACK_DEPTH };
use crate::utils::{ filled_vector };

mod stack_trace;
use stack_trace::StackTrace;

// CONSTANTS
// ================================================================================================
const MIN_USER_STACK_DEPTH: usize = MIN_STACK_DEPTH - 1;
const MAX_USER_STACK_DEPTH: usize = MAX_STACK_DEPTH - 1;

// TRACE BUILDER
// ================================================================================================
pub fn execute<T>(program: &[T], inputs: &ProgramInputs<T>, extension_factor: usize) -> Vec<Vec<T>>
    where T: FiniteField + Hasher
{
    let trace_length = program.len();
    let domain_size = trace_length * extension_factor;

    assert!(program.len() > 1, "program length must be greater than 1");
    assert!(program.len().is_power_of_two(), "program length must be a power of 2");
    assert!(program[0] == T::from(opcodes::BEGIN), "first operation of a program must be BEGIN");
    assert!(program[program.len() - 1] == T::from(opcodes::NOOP), "last operation of a program must be NOOP");
    assert!(extension_factor.is_power_of_two(), "trace extension factor must be a power of 2");

    // allocate space for stack registers and populate the first state with public inputs
    let public_inputs = inputs.get_public_inputs();
    let init_stack_depth = cmp::max(public_inputs.len(), MIN_USER_STACK_DEPTH);
    let mut user_registers: Vec<Vec<T>> = Vec::with_capacity(init_stack_depth);
    for i in 0..init_stack_depth {
        let mut register = filled_vector(trace_length, domain_size, T::ZERO);
        if i < public_inputs.len() { 
            register[0] = public_inputs[i];
        }
        user_registers.push(register);
    }

    let aux_register = filled_vector(trace_length, domain_size, T::ZERO);

    // reverse secret inputs so that they are consumed in FIFO order
    let [secret_inputs_a, secret_inputs_b] = inputs.get_secret_inputs();
    let mut secret_inputs_a = secret_inputs_a.clone();
    secret_inputs_a.reverse();
    let mut secret_inputs_b = secret_inputs_b.clone();
    secret_inputs_b.reverse();

    let mut stack = StackTrace {
        aux_register,
        user_registers,
        secret_inputs_a,
        secret_inputs_b,
        max_depth: public_inputs.len(),
        depth: public_inputs.len()
    };

    // execute the program capturing each successive stack state in the trace
    let mut i = 0; 
    while i < trace_length - 1 {
        // update stack state based on the current operation
        // TODO: make sure operation can be safely cast to u8
        match program[i].as_u8() {

            opcodes::BEGIN   => stack.noop(i),
            opcodes::NOOP    => stack.noop(i),
            opcodes::ASSERT  => stack.assert(i),

            opcodes::PUSH  => {
                // push the value of the next instruction onto the stack and skip a step
                // since next instruction is not an operation
                stack.push(i, program[i + 1]);
                i += 1;
                stack.noop(i);
            },

            opcodes::READ    => stack.read(i),
            opcodes::READ2   => stack.read2(i),

            opcodes::DUP     => stack.dup(i),
            opcodes::DUP2    => stack.dup2(i),
            opcodes::DUP4    => stack.dup4(i),
            opcodes::PAD2    => stack.pad2(i),

            opcodes::DROP    => stack.drop(i),
            opcodes::DROP4   => stack.drop4(i),

            opcodes::SWAP    => stack.swap(i),
            opcodes::SWAP2   => stack.swap2(i),
            opcodes::SWAP4   => stack.swap4(i),

            opcodes::ROLL4   => stack.roll4(i),
            opcodes::ROLL8   => stack.roll8(i),

            opcodes::CHOOSE  => stack.choose(i),
            opcodes::CHOOSE2 => stack.choose2(i),

            opcodes::ADD     => stack.add(i),
            opcodes::MUL     => stack.mul(i),
            opcodes::INV     => stack.inv(i),
            opcodes::NEG     => stack.neg(i),
            opcodes::NOT     => stack.not(i),

            opcodes::EQ      => stack.eq(i),
            opcodes::CMP     => stack.cmp(i),
            opcodes::BINACC  => stack.binacc(i),

            opcodes::HASHR   => stack.hashr(i),

            _ => panic!("operation {} is not supported", program[i])
        }
        i += 1;
    }

    // make sure all secret inputs have been consumed
    assert!(stack.secret_inputs_a.len() == 0 && stack.secret_inputs_b.len() == 0,
        "not all secret inputs have been consumed");

    // keep only the registers used during program execution
    stack.user_registers.truncate(stack.max_depth);
    let mut registers = Vec::with_capacity(stack.user_registers.len() + 1);
    registers.push(stack.aux_register);
    registers.append(&mut stack.user_registers);

    return registers;
}