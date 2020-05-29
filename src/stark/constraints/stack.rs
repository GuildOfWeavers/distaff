use std::cmp;
use crate::math::{ FiniteField, polynom };
use crate::stark::{ TraceState, Accumulator, Hasher, MAX_STACK_DEPTH };
use crate::processor::{ opcodes };

// CONSTANTS
// ================================================================================================
// TODO: set correct degrees for all stack registers
const CONSTRAINT_DEGREES: [usize; MAX_STACK_DEPTH] = [8; MAX_STACK_DEPTH];

// TYPES AND INTERFACES
// ================================================================================================
pub struct Stack<T: FiniteField> {
    stack_depth     : usize,
    hash_evaluator  : HashEvaluator<T>,
}

// STACK CONSTRAINT EVALUATOR IMPLEMENTATION
// ================================================================================================
impl <T> Stack<T>
    where T: FiniteField + Accumulator + Hasher
{
    pub fn new(trace_length: usize, extension_factor: usize, stack_depth: usize) -> Stack<T> {

        let hash_evaluator = HashEvaluator::new(trace_length, extension_factor);

        return Stack { stack_depth, hash_evaluator };
    }

    pub fn constraint_degrees(&self) -> &[usize] {
        return &CONSTRAINT_DEGREES[..self.stack_depth];
    }

    // EVALUATOR FUNCTIONS
    // --------------------------------------------------------------------------------------------
    pub fn evaluate(&self, current: &TraceState<T>, next: &TraceState<T>, step: usize, result: &mut [T]) {

        let op_flags = current.get_op_flags();
        let current_stack = current.get_stack();
        let next_stack = next.get_stack();

        // evaluate constraints for simple operations
        let next_op = next.get_op_code();
        self.simple_transitions(current_stack, next_stack, op_flags, next_op, result);

        // evaluate constraints for hash operation
        let hash_flag = op_flags[opcodes::HASH as usize];
        self.hash_evaluator.evaluate(current_stack, next_stack, step, hash_flag, result);
    }

    pub fn evaluate_at(&self, current: &TraceState<T>, next: &TraceState<T>, x: T, result: &mut [T]) {
        let op_flags = current.get_op_flags();
        let current_stack = current.get_stack();
        let next_stack = next.get_stack();

        // evaluate constraints for simple operations
        let next_op = next.get_op_code();
        self.simple_transitions(current_stack, next_stack, op_flags, next_op, result);

        // evaluate constraints for hash operation
        let hash_flag = op_flags[opcodes::HASH as usize];
        self.hash_evaluator.evaluate_at(current_stack, next_stack, x, hash_flag, result);
    }

    // TODO: use NUM_LD_OPS
    fn simple_transitions(&self, current: &[T], next: &[T], op_flags: [T; 32], next_op: T, result: &mut [T]) {
        
        // TODO: use AUX_WIDTH
        let current = &current[2..];
        let next = &next[2..];
        let result = &mut result[2..];

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

// HASH EVALUATOR
// ================================================================================================
struct HashEvaluator<T: FiniteField> {
    trace_length    : usize,
    cycle_length    : usize,
    ark_values      : Vec<Vec<T>>,
    ark_polys       : Vec<Vec<T>>,
}

impl<T> HashEvaluator <T>
    where T: FiniteField + Hasher
{
    pub fn new(trace_length: usize, extension_factor: usize) -> HashEvaluator<T> {
        let (ark_polys, ark_evaluations) = T::get_extended_constants(extension_factor);

        let cycle_length = T::NUM_ROUNDS * extension_factor;
        let mut ark_values = Vec::with_capacity(cycle_length);
        for i in 0..(T::NUM_ROUNDS * extension_factor) {
            ark_values.push(vec![T::ZERO; 2 * T::STATE_WIDTH]);
            for j in 0..(2 * T::STATE_WIDTH) {
                ark_values[i][j] = ark_evaluations[j][i];
            }
        }

        return HashEvaluator { trace_length, cycle_length, ark_values, ark_polys };
    }

    pub fn evaluate(&self, current: &[T], next: &[T], step: usize, op_flag: T, result: &mut [T]) {
        let ark = &self.ark_values[step % self.cycle_length];
        self.eval_hash(current, next, ark, op_flag, result);
        self.eval_rest(current, next, op_flag, result);
    }

    pub fn evaluate_at(&self, current: &[T], next: &[T], x: T, op_flag: T, result: &mut [T]) {
        let num_cycles = T::from_usize(self.trace_length / T::NUM_ROUNDS);
        let x = T::exp(x, num_cycles);

        let mut ark = vec![T::ZERO; 2 * T::STATE_WIDTH];
        for i in 0..ark.len() {
            ark[i] = polynom::eval(&self.ark_polys[i], x);
        }
        self.eval_hash(current, next, &ark, op_flag, result);
        self.eval_rest(current, next, op_flag, result);
    }

    fn eval_hash(&self, current: &[T], next: &[T], ark: &[T], op_flag: T, result: &mut [T]) {

        let mut state_part1 = vec![T::ZERO; T::STATE_WIDTH];    // TODO: convert to array
        state_part1.copy_from_slice(&current[..6]);
        let mut state_part2 = vec![T::ZERO; T::STATE_WIDTH];    // TODO: convert to array
        state_part2.copy_from_slice(&next[..6]);

        for i in 0..T::STATE_WIDTH {
            state_part1[i] = T::add(state_part1[i], ark[i]);
        }
        T::apply_sbox(&mut state_part1);
        T::apply_mds(&mut state_part1);
    
        T::apply_inv_mds(&mut state_part2);
        T::apply_sbox(&mut state_part2);
        for i in 0..T::STATE_WIDTH {
            state_part2[i] = T::sub(state_part2[i], ark[T::STATE_WIDTH + i]);
        }

        for i in 0..cmp::min(result.len(), 6) {
            result[i] = T::mul(T::sub(state_part2[i], state_part1[i]), op_flag);
        }

    }

    fn eval_rest(&self, current: &[T], next: &[T], op_flag: T, result: &mut [T]) {
        for i in 6..result.len() {
            result[i] = T::add(result[i], T::sub(next[i], T::mul(current[i], op_flag)));
        }
    }
}