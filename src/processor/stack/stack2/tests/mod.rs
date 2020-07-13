use crate::math::{ field };
use crate::utils::{ hasher };
use super::{ Stack, super::ProgramInputs, OpHint, Opcode };
use crate::{ HASH_STATE_WIDTH };

mod comparisons;

const TRACE_LENGTH: usize = 16;

// FLOW CONTROL OPERATIONS
// ================================================================================================

#[test]
fn noop() {
    let mut stack = init_stack(&[1, 2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Noop, OpHint::None);
    assert_eq!(vec![1, 2, 3, 4, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(4, stack.depth);
    assert_eq!(4, stack.max_depth);
}

#[test]
fn assert() {
    let mut stack = init_stack(&[1, 2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Assert, OpHint::None);
    assert_eq!(vec![2, 3, 4, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(3, stack.depth);
    assert_eq!(4, stack.max_depth);
}

#[test]
#[should_panic(expected = "ASSERT failed at step 1")]
fn assert_fail() {
    let mut stack = init_stack(&[2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Assert, OpHint::None);
}

#[test]
fn asserteq() {
    let mut stack = init_stack(&[1, 1, 3, 4], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::AssertEq, OpHint::None);
    assert_eq!(vec![3, 4, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(2, stack.depth);
    assert_eq!(4, stack.max_depth);
}

#[test]
#[should_panic(expected = "ASSERTEQ failed at step 1")]
fn asserteq_fail() {
    let mut stack = init_stack(&[2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::AssertEq, OpHint::None);
}

// INPUT OPERATIONS
// ================================================================================================

#[test]
fn push() {
    let mut stack = init_stack(&[], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Push, OpHint::PushValue(3));
    assert_eq!(vec![3, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(1, stack.depth);
    assert_eq!(1, stack.max_depth);
}

#[test]
fn read() {
    let mut stack = init_stack(&[1], &[2, 3], &[], TRACE_LENGTH);

    stack.execute(Opcode::Read, OpHint::None);
    assert_eq!(vec![2, 1, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(2, stack.depth);
    assert_eq!(2, stack.max_depth);

    stack.execute(Opcode::Read, OpHint::None);
    assert_eq!(vec![3, 2, 1, 0, 0, 0, 0, 0], get_stack_state(&stack, 2));

    assert_eq!(3, stack.depth);
    assert_eq!(3, stack.max_depth);
}

#[test]
fn read2() {
    let mut stack = init_stack(&[1], &[2, 4], &[3, 5], TRACE_LENGTH);

    stack.execute(Opcode::Read2, OpHint::None);
    assert_eq!(vec![3, 2, 1, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(3, stack.depth);
    assert_eq!(3, stack.max_depth);

    stack.execute(Opcode::Read2, OpHint::None);
    assert_eq!(vec![5, 4, 3, 2, 1, 0, 0, 0], get_stack_state(&stack, 2));

    assert_eq!(5, stack.depth);
    assert_eq!(5, stack.max_depth);
}

// STACK MANIPULATION OPERATIONS
// ================================================================================================

#[test]
fn dup() {
    let mut stack = init_stack(&[1, 2], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Dup, OpHint::None);
    assert_eq!(vec![1, 1, 2, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(3, stack.depth);
    assert_eq!(3, stack.max_depth);
}

#[test]
fn dup2() {
    let mut stack = init_stack(&[1, 2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Dup2, OpHint::None);
    assert_eq!(vec![1, 2, 1, 2, 3, 4, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(6, stack.depth);
    assert_eq!(6, stack.max_depth);
}

#[test]
fn dup4() {
    let mut stack = init_stack(&[1, 2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Dup4, OpHint::None);
    assert_eq!(vec![1, 2, 3, 4, 1, 2, 3, 4], get_stack_state(&stack, 1));

    assert_eq!(8, stack.depth);
    assert_eq!(8, stack.max_depth);
}

#[test]
fn pad2() {
    let mut stack = init_stack(&[1, 2], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Pad2, OpHint::None);
    assert_eq!(vec![0, 0, 1, 2, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(4, stack.depth);
    assert_eq!(4, stack.max_depth);
}

#[test]
fn drop() {
    let mut stack = init_stack(&[1, 2], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Drop, OpHint::None);
    assert_eq!(vec![2, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(1, stack.depth);
    assert_eq!(2, stack.max_depth);
}

#[test]
fn drop4() {
    let mut stack = init_stack(&[1, 2, 3, 4, 5], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Drop4, OpHint::None);
    assert_eq!(vec![5, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(1, stack.depth);
    assert_eq!(5, stack.max_depth);
}

#[test]
fn swap() {
    let mut stack = init_stack(&[1, 2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Swap, OpHint::None);
    assert_eq!(vec![2, 1, 3, 4, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(4, stack.depth);
    assert_eq!(4, stack.max_depth);
}

#[test]
fn swap2() {
    let mut stack = init_stack(&[1, 2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Swap2, OpHint::None);
    assert_eq!(vec![3, 4, 1, 2, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(4, stack.depth);
    assert_eq!(4, stack.max_depth);
}

#[test]
fn swap4() {
    let mut stack = init_stack(&[1, 2, 3, 4, 5, 6, 7, 8], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Swap4, OpHint::None);
    assert_eq!(vec![5, 6, 7, 8, 1, 2, 3, 4], get_stack_state(&stack, 1));

    assert_eq!(8, stack.depth);
    assert_eq!(8, stack.max_depth);
}

#[test]
fn roll4() {
    let mut stack = init_stack(&[1, 2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Roll4, OpHint::None);
    assert_eq!(vec![4, 1, 2, 3, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(4, stack.depth);
    assert_eq!(4, stack.max_depth);
}

#[test]
fn roll8() {
    let mut stack = init_stack(&[1, 2, 3, 4, 5, 6, 7, 8], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Roll8, OpHint::None);
    assert_eq!(vec![8, 1, 2, 3, 4, 5, 6, 7], get_stack_state(&stack, 1));

    assert_eq!(8, stack.depth);
    assert_eq!(8, stack.max_depth);
}

// CONDITIONAL OPERATIONS
// ================================================================================================

#[test]
fn choose() {
    // choose on true
    let mut stack = init_stack(&[2, 3, 0], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Choose, OpHint::None);
    assert_eq!(vec![3, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(1, stack.depth);
    assert_eq!(3, stack.max_depth);

    let mut stack = init_stack(&[2, 3, 0, 4], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Choose, OpHint::None);
    assert_eq!(vec![3, 4, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(2, stack.depth);
    assert_eq!(4, stack.max_depth);

    // choose on false
    let mut stack = init_stack(&[2, 3, 1, 4], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Choose, OpHint::None);
    assert_eq!(vec![2, 4, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(2, stack.depth);
    assert_eq!(4, stack.max_depth);
}

#[test]
#[should_panic(expected = "CHOOSE on a non-binary condition at step 1")]
fn choose_fail() {
    let mut stack = init_stack(&[2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Choose, OpHint::None);
}

#[test]
fn choose2() {
    // choose on true
    let mut stack = init_stack(&[2, 3, 4, 5, 0, 6, 7], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Choose2, OpHint::None);
    assert_eq!(vec![4, 5, 7, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(3, stack.depth);
    assert_eq!(7, stack.max_depth);

    // choose on false
    let mut stack = init_stack(&[2, 3, 4, 5, 1, 6, 7], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Choose2, OpHint::None);
    assert_eq!(vec![2, 3, 7, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(3, stack.depth);
    assert_eq!(7, stack.max_depth);
}

#[test]
#[should_panic(expected = "CHOOSE2 on a non-binary condition at step 1")]
fn choose2_fail() {
    let mut stack = init_stack(&[2, 3, 4, 5, 6, 8, 8], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Choose2, OpHint::None);
}

// ARITHMETIC AND BOOLEAN OPERATIONS
// ================================================================================================

#[test]
fn add() {
    let mut stack = init_stack(&[1, 2], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Add, OpHint::None);
    assert_eq!(vec![3, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(1, stack.depth);
    assert_eq!(2, stack.max_depth);
}

#[test]
fn mul() {
    let mut stack = init_stack(&[2, 3], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Mul, OpHint::None);
    assert_eq!(vec![6, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(1, stack.depth);
    assert_eq!(2, stack.max_depth);
}

#[test]
fn inv() {
    let mut stack = init_stack(&[2, 3], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Inv, OpHint::None);
    assert_eq!(vec![field::inv(2), 3, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(2, stack.depth);
    assert_eq!(2, stack.max_depth);
}

#[test]
#[should_panic(expected = "cannot compute INV of 0 at step 1")]
fn inv_zero() {
    let mut stack = init_stack(&[0], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Inv, OpHint::None);
}

#[test]
fn neg() {
    let mut stack = init_stack(&[2, 3], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Neg, OpHint::None);
    assert_eq!(vec![field::neg(2), 3, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(2, stack.depth);
    assert_eq!(2, stack.max_depth);
}

#[test]
fn not() {
    let mut stack = init_stack(&[1, 2], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Not, OpHint::None);
    assert_eq!(vec![0, 2, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(2, stack.depth);
    assert_eq!(2, stack.max_depth);

    stack.execute(Opcode::Not, OpHint::None);
    assert_eq!(vec![1, 2, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 2));

    assert_eq!(2, stack.depth);
    assert_eq!(2, stack.max_depth);
}

#[test]
#[should_panic(expected = "cannot compute NOT of a non-binary value at step 1")]
fn not_fail() {
    let mut stack = init_stack(&[2, 3], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Not, OpHint::None);
}

#[test]
fn and() {
    let mut stack = init_stack(&[1, 1, 0], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::And, OpHint::None);
    assert_eq!(vec![1, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(2, stack.depth);
    assert_eq!(3, stack.max_depth);

    stack.execute(Opcode::And, OpHint::None);
    assert_eq!(vec![0, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 2));

    assert_eq!(1, stack.depth);
    assert_eq!(3, stack.max_depth);
}

#[test]
#[should_panic(expected = "cannot compute AND for a non-binary value at step 1")]
fn and_fail() {
    let mut stack = init_stack(&[1, 3], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::And, OpHint::None);
}

#[test]
fn or() {
    let mut stack = init_stack(&[0, 0, 1], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Or, OpHint::None);
    assert_eq!(vec![0, 1, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(2, stack.depth);
    assert_eq!(3, stack.max_depth);

    stack.execute(Opcode::Or, OpHint::None);
    assert_eq!(vec![1, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 2));

    assert_eq!(1, stack.depth);
    assert_eq!(3, stack.max_depth);
}

#[test]
#[should_panic(expected = "cannot compute OR for a non-binary value at step 1")]
fn and_or() {
    let mut stack = init_stack(&[1, 3], &[], &[], TRACE_LENGTH);
    stack.execute(Opcode::Or, OpHint::None);
}

// CRYPTOGRAPHIC OPERATIONS
// ================================================================================================

#[test]
fn rescr() {
    let mut stack = init_stack(&[0, 0, 1, 2, 3, 4], &[], &[], TRACE_LENGTH);
    let mut expected = vec![0, 0, 1, 2, 3, 4, 0, 0];

    stack.execute(Opcode::RescR, OpHint::None);
    hasher::apply_round(&mut expected[..HASH_STATE_WIDTH], 0);
    assert_eq!(expected, get_stack_state(&stack, 1));

    stack.execute(Opcode::RescR, OpHint::None);
    hasher::apply_round(&mut expected[..HASH_STATE_WIDTH], 1);
    assert_eq!(expected, get_stack_state(&stack, 2));

    assert_eq!(6, stack.depth);
    assert_eq!(6, stack.max_depth);
}

// HELPER FUNCTIONS
// ================================================================================================

fn init_stack(public_inputs: &[u128], secret_inputs_a: &[u128], secret_inputs_b: &[u128], trace_length: usize) -> Stack {
    let inputs = ProgramInputs::new(public_inputs, secret_inputs_a, secret_inputs_b);
    return Stack::new(&inputs, trace_length);
}

fn get_stack_state(stack: &Stack, step: usize) -> Vec<u128> {
    let mut state = Vec::with_capacity(stack.registers.len());
    for i in 0..stack.registers.len() {
        state.push(stack.registers[i][step]);
    }
    return state;
}