use std::cmp;
use crate::math::{ FiniteField, polynom };
use crate::stark::{ TraceState, Accumulator, Hasher };
use crate::stark::{ MAX_STACK_DEPTH, STACK_HEAD_SIZE, NUM_LD_OPS, HASH_STATE_WIDTH, HASH_CYCLE_LENGTH };
use crate::processor::{ opcodes };

// CONSTANTS
// ================================================================================================
// TODO: set correct degrees for stack constraints
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
        return Stack {
            stack_depth     : stack_depth,
            hash_evaluator  : HashEvaluator::new(trace_length, extension_factor)
        };
    }

    pub fn constraint_degrees(&self) -> &[usize] {
        return &CONSTRAINT_DEGREES[..self.stack_depth];
    }

    // EVALUATOR FUNCTIONS
    // --------------------------------------------------------------------------------------------

    /// Evaluates stack transition constraints at the specified step of the evaluation domain and
    /// saves the evaluations into `result`.
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

    /// Evaluates stack transition constraints at the specified x coordinate and saves the
    /// evaluations into `result`. Unlike the function above, this function can evaluate constraints
    /// at any out-of-domain point, but it is much slower than the previous function.
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

    // SIMPLE OPERATIONS
    // --------------------------------------------------------------------------------------------

    /// Evaluates transition constraints for all operations where constraints can be described as:
    /// evaluation = s_next - f(s_current), where f is the transition function.
    fn simple_transitions(&self, current: &[T], next: &[T], op_flags: [T; NUM_LD_OPS], next_op: T, result: &mut [T]) {
        
        // simple operations work only with the user portion of the stack
        let current = &current[STACK_HEAD_SIZE..];
        let next = &next[STACK_HEAD_SIZE..];
        let result = &mut result[STACK_HEAD_SIZE..];

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
    ark_values      : Vec<[T; 2 * HASH_STATE_WIDTH]>,
    ark_polys       : Vec<Vec<T>>,
}

impl<T> HashEvaluator <T>
    where T: FiniteField + Hasher
{
    /// Creates a new HashEvaluator based on the provided `trace_length` and `extension_factor`.
    pub fn new(trace_length: usize, extension_factor: usize) -> HashEvaluator<T> {
        // extend rounds constants by the specified extension factor
        let (ark_polys, ark_evaluations) = T::get_extended_constants(extension_factor);

        // transpose round constant evaluations so that constants for each round
        // are stored in a single row
        let cycle_length = HASH_CYCLE_LENGTH * extension_factor;
        let mut ark_values = Vec::with_capacity(cycle_length);
        for i in 0..cycle_length {
            ark_values.push([T::ZERO; 2 * HASH_STATE_WIDTH]);
            for j in 0..(2 * HASH_STATE_WIDTH) {
                ark_values[i][j] = ark_evaluations[j][i];
            }
        }

        return HashEvaluator { trace_length, cycle_length, ark_values, ark_polys };
    }

    /// Evaluates constraints at the specified step and adds the resulting values to `result`.
    pub fn evaluate(&self, current: &[T], next: &[T], step: usize, op_flag: T, result: &mut [T]) {
        // determine round constants for the current step
        let ark = &self.ark_values[step % self.cycle_length];
        // evaluate constraints for the hash function and for the rest of the stack
        self.eval_hash(current, next, ark, op_flag, result);
        self.eval_rest(current, next, op_flag, result);
    }

    /// Evaluates constraints at the specified x coordinate and adds the resulting values to `result`.
    /// Unlike the function above, this function can evaluate constraints for any out-of-domain 
    /// coordinate, but is significantly slower.
    pub fn evaluate_at(&self, current: &[T], next: &[T], x: T, op_flag: T, result: &mut [T]) {

        // determine round constants at the specified x coordinate
        let num_cycles = T::from_usize(self.trace_length / HASH_CYCLE_LENGTH);
        let x = T::exp(x, num_cycles);
        let mut ark = [T::ZERO; 2 * HASH_STATE_WIDTH];
        for i in 0..ark.len() {
            ark[i] = polynom::eval(&self.ark_polys[i], x);
        }

        // evaluate constraints for the hash function and for the rest of the stack
        self.eval_hash(current, next, &ark, op_flag, result);
        self.eval_rest(current, next, op_flag, result);
    }

    /// Evaluates constraints for a single round of a modified Rescue hash function. Hash state is
    /// assumed to be in the first 6 registers of the stack: 2 head registers + 4 user registers.
    fn eval_hash(&self, current: &[T], next: &[T], ark: &[T], op_flag: T, result: &mut [T]) {

        // TODO: enforce that capacity portion of the stack resets to 0 on every 16th step

        let mut state_part1 = [T::ZERO; HASH_STATE_WIDTH];
        state_part1.copy_from_slice(&current[..HASH_STATE_WIDTH]);
        let mut state_part2 = [T::ZERO; HASH_STATE_WIDTH];
        state_part2.copy_from_slice(&next[..HASH_STATE_WIDTH]);

        for i in 0..HASH_STATE_WIDTH {
            state_part1[i] = T::add(state_part1[i], ark[i]);
        }
        T::apply_sbox(&mut state_part1);
        T::apply_mds(&mut state_part1);
    
        T::apply_inv_mds(&mut state_part2);
        T::apply_sbox(&mut state_part2);
        for i in 0..HASH_STATE_WIDTH {
            state_part2[i] = T::sub(state_part2[i], ark[HASH_STATE_WIDTH + i]);
        }

        for i in 0..cmp::min(result.len(), HASH_STATE_WIDTH) {
            let evaluation = T::sub(state_part2[i], state_part1[i]);
            result[i] = T::add(result[i], T::mul(evaluation, op_flag));
        }
    }

    /// Evaluates constraints for stack registers un-affected by hash transition.
    fn eval_rest(&self, current: &[T], next: &[T], op_flag: T, result: &mut [T]) {
        for i in HASH_STATE_WIDTH..result.len() {
            let evaluation = T::sub(next[i], current[i]);
            result[i] = T::add(result[i], T::mul(evaluation, op_flag));
        }
    }
}