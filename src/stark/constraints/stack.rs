use std::{ cmp, marker::PhantomData };
use crate::math::{ FiniteField };
use crate::stark::{ TraceState, Accumulator, MIN_STACK_DEPTH, MAX_STACK_DEPTH };
use crate::processor::{ opcodes };

// CONSTANTS
// ================================================================================================
// TODO: set correct degrees for all stack registers
const CONSTRAINT_DEGREES: [usize; MAX_STACK_DEPTH] = [7; MAX_STACK_DEPTH];

// TYPES AND INTERFACES
// ================================================================================================
pub struct Stack<T> {
    stack_depth : usize,
    phantom     : PhantomData<T>
}

// STACK CONSTRAINT EVALUATOR IMPLEMENTATION
// ================================================================================================
impl <T> Stack<T>
    where T: FiniteField + Accumulator
{
    pub fn new(stack_depth: usize) -> Stack<T> {
        return Stack { stack_depth, phantom: PhantomData };
    }

    pub fn constraint_degrees(&self) -> &[usize] {
        return &CONSTRAINT_DEGREES[..self.stack_depth];
    }

    // EVALUATOR FUNCTION
    // --------------------------------------------------------------------------------------------
    pub fn evaluate(&self, current: &TraceState<T>, next: &TraceState<T>, result: &mut [T]) {

        let op_flags = current.get_op_flags();
        let current_stack = current.get_stack();
        let mut expected_stack = vec![T::ZERO; cmp::max(self.stack_depth, MIN_STACK_DEPTH)];
    
        T::mul_acc(&mut expected_stack,  current_stack, op_flags[opcodes::NOOP as usize]);
    
        Self::op_swap(&mut expected_stack,  current_stack, op_flags[opcodes::SWAP as usize]);
        Self::op_swap2(&mut expected_stack, current_stack, op_flags[opcodes::SWAP2 as usize]);
        Self::op_swap4(&mut expected_stack, current_stack, op_flags[opcodes::SWAP4 as usize]);
    
        Self::op_push(&mut expected_stack,  current_stack, next.get_op_code(), op_flags[opcodes::PUSH as usize]);
        Self::op_dup(&mut expected_stack,   current_stack, op_flags[opcodes::DUP as usize]);
        Self::op_dup2(&mut expected_stack,  current_stack, op_flags[opcodes::DUP2 as usize]);
        Self::op_dup4(&mut expected_stack,  current_stack, op_flags[opcodes::DUP4 as usize]);
    
        Self::op_drop(&mut expected_stack,  current_stack, op_flags[opcodes::DROP as usize]);
        Self::op_add(&mut expected_stack,   current_stack, op_flags[opcodes::ADD as usize]);
        Self::op_sub(&mut expected_stack,   current_stack, op_flags[opcodes::SUB as usize]);
        Self::op_mul(&mut expected_stack,   current_stack, op_flags[opcodes::MUL as usize]);
    
        let next_stack = next.get_stack();
        for i in 0..self.stack_depth {
            result[i] = T::sub(next_stack[i], expected_stack[i]);
        }
    }

    // OPERATIONS
    // --------------------------------------------------------------------------------------------
    fn op_swap(next: &mut [T], current: &[T], op_flag: T) {
        next[0] = T::add(next[0], T::mul(current[1], op_flag));
        next[1] = T::add(next[1], T::mul(current[0], op_flag));
        T::mul_acc(&mut next[2..], &current[2..], op_flag);
    }
    
    fn op_swap2(next: &mut [T], current: &[T], op_flag: T) {
        next[0] = T::add(next[0], T::mul(current[2], op_flag));
        next[1] = T::add(next[1], T::mul(current[3], op_flag));
        next[2] = T::add(next[2], T::mul(current[0], op_flag));
        next[3] = T::add(next[3], T::mul(current[1], op_flag));
        T::mul_acc(&mut next[4..], &current[3..], op_flag);
    }
    
    fn op_swap4(next: &mut [T], current: &[T], op_flag: T) {
        next[0] = T::add(next[0], T::mul(current[4], op_flag));
        next[1] = T::add(next[1], T::mul(current[5], op_flag));
        next[2] = T::add(next[2], T::mul(current[6], op_flag));
        next[3] = T::add(next[3], T::mul(current[7], op_flag));
        next[4] = T::add(next[4], T::mul(current[0], op_flag));
        next[5] = T::add(next[5], T::mul(current[1], op_flag));
        next[6] = T::add(next[6], T::mul(current[2], op_flag));
        next[7] = T::add(next[7], T::mul(current[3], op_flag));
        T::mul_acc(&mut next[8..], &current[3..], op_flag);
    }

    fn op_push(next: &mut [T], current: &[T], op_code: T, op_flag: T) {
        next[0] = T::add(next[0], T::mul(op_code, op_flag));
        T::mul_acc(&mut next[1..], &current[0..], op_flag);
    }
    
    fn op_dup(next: &mut [T], current: &[T], op_flag: T) {
        next[0] = T::add(next[0], T::mul(current[0], op_flag));
        T::mul_acc(&mut next[1..], &current[0..], op_flag);
    }
    
    fn op_dup2(next: &mut [T], current: &[T], op_flag: T) {
        next[0] = T::add(next[0], T::mul(current[0], op_flag));
        next[1] = T::add(next[1], T::mul(current[1], op_flag));
        T::mul_acc(&mut next[2..], &current[0..], op_flag);
    }
    
    fn op_dup4(next: &mut [T], current: &[T], op_flag: T) {
        next[0] = T::add(next[0], T::mul(current[0], op_flag));
        next[1] = T::add(next[1], T::mul(current[1], op_flag));
        next[2] = T::add(next[2], T::mul(current[2], op_flag));
        next[3] = T::add(next[3], T::mul(current[3], op_flag));
        T::mul_acc(&mut next[4..], &current[0..], op_flag);
    }

    fn op_drop(next: &mut [T], current: &[T], op_flag: T) {
        let n = next.len() - 1;
        T::mul_acc(&mut next[0..n], &current[1..], op_flag);
    }
    
    fn op_add(next: &mut [T], current: &[T], op_flag: T) {
        let n = next.len() - 1;
        let op_result = T::add(current[0], current[1]);
        next[0] = T::add(next[0], T::mul(op_result, op_flag));
        T::mul_acc(&mut next[1..n], &current[2..], op_flag);
    }
    
    fn op_sub(next: &mut [T], current: &[T], op_flag: T) {
        let n = next.len() - 1;
        let op_result = T::sub(current[1], current[0]);
        next[0] = T::add(next[0], T::mul(op_result, op_flag));
        T::mul_acc(&mut next[1..n], &current[2..], op_flag);
    }
    
    fn op_mul(next: &mut [T], current: &[T], op_flag: T) {
        let n = next.len() - 1;
        let op_result = T::mul(current[1], current[0]);
        next[0] = T::add(next[0], T::mul(op_result, op_flag));
        T::mul_acc(&mut next[1..n], &current[2..], op_flag);
    }

}