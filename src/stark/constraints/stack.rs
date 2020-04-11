use std::cmp;
use crate::math::field::{ add, sub, mul };
use crate::stark::{ TraceState, MIN_STACK_DEPTH };
use crate::processor::{ opcodes };

// EVALUATOR FUNCTION
// ================================================================================================
pub fn evaluate(current: &TraceState, next: &TraceState, stack_depth: usize) -> Vec<u64> {

    let op_flags = current.get_op_flags();
    let current_stack = current.get_stack();
    let mut expected_stack = vec![0u64; cmp::max(stack_depth, MIN_STACK_DEPTH)];

    mul_acc(&mut expected_stack,  current_stack, op_flags[opcodes::NOOP as usize]);

    op_pull1(&mut expected_stack, current_stack, op_flags[opcodes::PULL1 as usize]);
    op_pull2(&mut expected_stack, current_stack, op_flags[opcodes::PULL2 as usize]);

    op_push(&mut expected_stack,  current_stack, next.get_op_code(), op_flags[opcodes::PUSH as usize]);
    op_dup0(&mut expected_stack,  current_stack, op_flags[opcodes::DUP0 as usize]);
    op_dup1(&mut expected_stack,  current_stack, op_flags[opcodes::DUP1 as usize]);

    op_drop(&mut expected_stack,  current_stack, op_flags[opcodes::DROP as usize]);
    op_add(&mut expected_stack,   current_stack, op_flags[opcodes::ADD as usize]);
    op_sub(&mut expected_stack,   current_stack, op_flags[opcodes::SUB as usize]);
    op_mul(&mut expected_stack,   current_stack, op_flags[opcodes::MUL as usize]);

    let next_stack = next.get_stack();
    expected_stack.truncate(stack_depth);
    for i in 0..stack_depth {
        expected_stack[i] = sub(next_stack[i], expected_stack[i]);
    }
    return expected_stack;
}

// OPERATIONS
// ================================================================================================
fn op_pull1(next: &mut [u64], current: &[u64], op_flag: u64) {
    next[0] = add(next[0], mul(current[1], op_flag));
    next[1] = add(next[1], mul(current[0], op_flag));
    mul_acc(&mut next[2..], &current[2..], op_flag);
}

fn op_pull2(next: &mut [u64], current: &[u64], op_flag: u64) {
    next[0] = add(next[0], mul(current[2], op_flag));
    next[1] = add(next[1], mul(current[0], op_flag));
    next[2] = add(next[2], mul(current[1], op_flag));
    mul_acc(&mut next[3..], &current[3..], op_flag);
}

fn op_push(next: &mut [u64], current: &[u64], op_code: u64, op_flag: u64) {
    next[0] = add(next[0], mul(op_code, op_flag));
    mul_acc(&mut next[1..], &current[0..], op_flag);
}

fn op_dup0(next: &mut [u64], current: &[u64], op_flag: u64) {
    next[0] = add(next[0], mul(current[0], op_flag));
    mul_acc(&mut next[1..], &current[0..], op_flag);
}

fn op_dup1(next: &mut [u64], current: &[u64], op_flag: u64) {
    next[0] = add(next[0], mul(current[1], op_flag));
    mul_acc(&mut next[1..], &current[0..], op_flag);
}

fn op_drop(next: &mut [u64], current: &[u64], op_flag: u64) {
    let n = next.len() - 1;
    mul_acc(&mut next[0..n], &current[1..], op_flag);
}

fn op_add(next: &mut [u64], current: &[u64], op_flag: u64) {
    let n = next.len() - 1;
    let op_result = add(current[0], current[1]);
    next[0] = add(next[0], mul(op_result, op_flag));
    mul_acc(&mut next[1..n], &current[2..], op_flag);
}

fn op_sub(next: &mut [u64], current: &[u64], op_flag: u64) {
    let n = next.len() - 1;
    let op_result = sub(current[1], current[0]);
    next[0] = add(next[0], mul(op_result, op_flag));
    mul_acc(&mut next[1..n], &current[2..], op_flag);
}

fn op_mul(next: &mut [u64], current: &[u64], op_flag: u64) {
    let n = next.len() - 1;
    let op_result = mul(current[1], current[0]);
    next[0] = add(next[0], mul(op_result, op_flag));
    mul_acc(&mut next[1..n], &current[2..], op_flag);
}

// HELPER FUNCTIONS
// ================================================================================================
fn mul_acc(a: &mut [u64], b: &[u64], c: u64) {
    for i in 0..a.len() {
        a[i] = add(a[i], mul(b[i], c));
    }
}