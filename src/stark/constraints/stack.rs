use std::{ marker::PhantomData };
use crate::math::{ FiniteField };
use crate::stark::{ TraceState, Accumulator, MAX_STACK_DEPTH };
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
        let next_stack = next.get_stack();
        let next_op = next.get_op_code();
        
        // TODO: use AUX_WIDTH
        let c_user_stack = &current_stack[2..];
        let n_user_stack = &next_stack[2..];

        self.simple_transitions(c_user_stack, n_user_stack, op_flags, next_op, &mut result[2..]);
    }

    // TODO: use NUM_LD_OPS
    fn simple_transitions(&self, current: &[T], next: &[T], op_flags: [T; 32], next_op: T, result: &mut [T]) {
        
        let mut expected = vec![T::ZERO; current.len()];

        T::mul_acc(&mut expected,       current, op_flags[opcodes::BEGIN as usize]);
        T::mul_acc(&mut expected,       current, op_flags[opcodes::NOOP as usize]);
    
        Self::op_swap(&mut expected,    current, op_flags[opcodes::SWAP as usize]);
        Self::op_swap2(&mut expected,   current, op_flags[opcodes::SWAP2 as usize]);
        Self::op_swap4(&mut expected,   current, op_flags[opcodes::SWAP4 as usize]);
    
        Self::op_roll4(&mut expected,   current, op_flags[opcodes::ROLL4 as usize]);
        Self::op_roll8(&mut expected,   current, op_flags[opcodes::ROLL8 as usize]);

        Self::op_choose(&mut expected,  current, op_flags[opcodes::CHOOSE as usize]);
        Self::op_choose2(&mut expected, current, op_flags[opcodes::CHOOSE2 as usize]);

        Self::op_push(&mut expected,    current, next_op, op_flags[opcodes::PUSH as usize]);
        Self::op_dup(&mut expected,     current, op_flags[opcodes::DUP as usize]);
        Self::op_dup2(&mut expected,    current, op_flags[opcodes::DUP2 as usize]);
        Self::op_dup4(&mut expected,    current, op_flags[opcodes::DUP4 as usize]);
    
        Self::op_drop(&mut expected,    current, op_flags[opcodes::DROP as usize]);
        Self::op_add(&mut expected,     current, op_flags[opcodes::ADD as usize]);
        Self::op_sub(&mut expected,     current, op_flags[opcodes::SUB as usize]);
        Self::op_mul(&mut expected,     current, op_flags[opcodes::MUL as usize]);
    
        for i in 0..result.len() {
            result[i] = T::sub(next[i], expected[i]);
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
        T::mul_acc(&mut next[4..], &current[4..], op_flag);
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
        T::mul_acc(&mut next[8..], &current[8..], op_flag);
    }

    fn op_roll4(next: &mut [T], current: &[T], op_flag: T) {
        next[0] = T::add(next[0], T::mul(current[3], op_flag));
        next[1] = T::add(next[1], T::mul(current[0], op_flag));
        next[2] = T::add(next[2], T::mul(current[1], op_flag));
        next[3] = T::add(next[3], T::mul(current[2], op_flag));
        T::mul_acc(&mut next[4..], &current[4..], op_flag);
    }
    
    fn op_roll8(next: &mut [T], current: &[T], op_flag: T) {
        next[0] = T::add(next[0], T::mul(current[7], op_flag));
        next[1] = T::add(next[1], T::mul(current[0], op_flag));
        next[2] = T::add(next[2], T::mul(current[1], op_flag));
        next[3] = T::add(next[3], T::mul(current[2], op_flag));
        next[4] = T::add(next[4], T::mul(current[3], op_flag));
        next[5] = T::add(next[5], T::mul(current[4], op_flag));
        next[6] = T::add(next[6], T::mul(current[5], op_flag));
        next[7] = T::add(next[7], T::mul(current[6], op_flag));
        T::mul_acc(&mut next[8..], &current[8..], op_flag);
    }

    fn op_choose(next: &mut [T], current: &[T], op_flag: T) {
        let n = next.len() - 2;
        let condition1 = current[2];
        let condition2 = T::sub(T::ONE, condition1);
        let value = T::add(T::mul(condition1, current[0]), T::mul(condition2, current[1]));
        next[0] = T::add(next[0], T::mul(value, op_flag));
        T::mul_acc(&mut next[1..n], &current[3..], op_flag);
    }

    fn op_choose2(next: &mut [T], current: &[T], op_flag: T) {
        let n = next.len() - 4;
        let condition1 = current[4];
        let condition2 = T::sub(T::ONE, condition1);
        let value1 = T::add(T::mul(condition1, current[0]), T::mul(condition2, current[2]));
        next[0] = T::add(next[0], T::mul(value1, op_flag));
        let value2 = T::add(T::mul(condition1, current[1]), T::mul(condition2, current[3]));
        next[1] = T::add(next[1], T::mul(value2, op_flag));
        T::mul_acc(&mut next[2..n], &current[6..], op_flag);
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