use crate::math::{ FiniteField };
use crate::stark::{ TraceState, Accumulator, Hasher, AUX_WIDTH, NUM_LD_OPS };
use crate::processor::{ opcodes };

mod hashing;
use hashing::HashEvaluator;

mod comparisons;
use comparisons::{ enforce_eq, enforce_cmp };

mod utils;
use utils::{ agg_op_constraint, is_binary, enforce_no_change };

// CONSTANTS
// ================================================================================================
const STACK_HEAD_DEGREES: [usize; 8] = [
    7, 0,               // aux constraints
    8, 8, 8, 8, 8, 8,   // constraints for the first 6 registers of user stack
];
const STACK_REST_DEGREE: usize = 6; // degree for the rest of the stack registers

// TYPES AND INTERFACES
// ================================================================================================
pub struct Stack<T: FiniteField> {
    hash_evaluator      : HashEvaluator<T>,
    constraint_degrees  : Vec<usize>
}

// STACK CONSTRAINT EVALUATOR IMPLEMENTATION
// ================================================================================================
impl <T> Stack<T>
    where T: FiniteField + Accumulator + Hasher
{
    pub fn new(trace_length: usize, extension_factor: usize, stack_depth: usize) -> Stack<T> {

        let mut degrees = Vec::from(&STACK_HEAD_DEGREES[..]);
        degrees.resize(stack_depth, STACK_REST_DEGREE);

        return Stack {
            hash_evaluator      : HashEvaluator::new(trace_length, extension_factor),
            constraint_degrees  : degrees,
        };
    }

    pub fn constraint_degrees(&self) -> &[usize] {
        return &self.constraint_degrees;
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
        self.enforce_simple_ops(current_stack, next_stack, op_flags, next_op, result);

        // evaluate constraints for logic operations
        self.enforce_logic_ops(current_stack, next_stack, op_flags, result);

        // evaluate constraints for hash operation
        let hash_flag = op_flags[opcodes::HASHR as usize];
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
        self.enforce_simple_ops(current_stack, next_stack, op_flags, next_op, result);

        // evaluate constraints for logic operations
        self.enforce_logic_ops(current_stack, next_stack, op_flags, result);

        // evaluate constraints for hash operation
        let hash_flag = op_flags[opcodes::HASHR as usize];
        self.hash_evaluator.evaluate_at(current_stack, next_stack, x, hash_flag, result);
    }

    // SIMPLE OPERATIONS
    // --------------------------------------------------------------------------------------------

    /// Evaluates transition constraints for all operations where constraints can be described as:
    /// evaluation = s_next - f(s_current), where f is the transition function.
    fn enforce_simple_ops(&self, current: &[T], next: &[T], op_flags: [T; NUM_LD_OPS], next_op: T, result: &mut [T]) {
        
        debug_assert!(AUX_WIDTH == 2, "expected 2 aux registers but found {}", AUX_WIDTH);

        // simple operations work only with the user portion of the stack
        let current = &current[AUX_WIDTH..];
        let next = &next[AUX_WIDTH..];

        let mut evaluations = vec![T::ZERO; current.len()];

        enforce_no_change(&mut evaluations,     current, next, op_flags[opcodes::BEGIN as usize]);
        enforce_no_change(&mut evaluations,     current, next, op_flags[opcodes::NOOP as usize]);

        Self::enforce_push(&mut evaluations,    current, next, next_op, op_flags[opcodes::PUSH as usize]);
        Self::enforce_read(&mut evaluations,    current, next, op_flags[opcodes::READ as usize]);
        Self::enforce_read2(&mut evaluations,   current, next, op_flags[opcodes::READ2 as usize]);
    
        Self::enforce_drop(&mut evaluations,    current, next, op_flags[opcodes::DROP as usize]);
        Self::enforce_drop4(&mut evaluations,   current, next, op_flags[opcodes::DROP4 as usize]);
        
        Self::enforce_swap(&mut evaluations,    current, next, op_flags[opcodes::SWAP as usize]);
        Self::enforce_swap2(&mut evaluations,   current, next, op_flags[opcodes::SWAP2 as usize]);
        Self::enforce_swap4(&mut evaluations,   current, next, op_flags[opcodes::SWAP4 as usize]);
    
        Self::enforce_roll4(&mut evaluations,   current, next, op_flags[opcodes::ROLL4 as usize]);
        Self::enforce_roll8(&mut evaluations,   current, next, op_flags[opcodes::ROLL8 as usize]);

        Self::enforce_pad2(&mut evaluations,    current, next, op_flags[opcodes::PAD2 as usize]);
        Self::enforce_dup(&mut evaluations,     current, next, op_flags[opcodes::DUP as usize]);
        Self::enforce_dup2(&mut evaluations,    current, next, op_flags[opcodes::DUP2 as usize]);
        Self::enforce_dup4(&mut evaluations,    current, next, op_flags[opcodes::DUP4 as usize]);
    
        Self::enforce_add(&mut evaluations,     current, next, op_flags[opcodes::ADD as usize]);
        Self::enforce_mul(&mut evaluations,     current, next, op_flags[opcodes::MUL as usize]);
        Self::enforce_inv(&mut evaluations,     current, next, op_flags[opcodes::INV as usize]);
        Self::enforce_neg(&mut evaluations,     current, next, op_flags[opcodes::NEG as usize]);

        let result = &mut result[AUX_WIDTH..];
        for i in 0..result.len() {
            result[i] = T::add(result[i], evaluations[i]);
        }
    }

    fn enforce_push(result: &mut [T], current: &[T], next: &[T], op_code: T, op_flag: T) {
        result[0] = agg_op_constraint(result[0], op_flag, T::sub(next[0], op_code));
        enforce_no_change(&mut result[1..], &current[0..], &next[1..], op_flag);
    }

    fn enforce_read(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
        enforce_no_change(&mut result[1..], &current[0..], &next[1..], op_flag);
    }

    fn enforce_read2(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
        enforce_no_change(&mut result[2..], &current[0..], &next[2..], op_flag);
    }

    fn enforce_drop(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
        let n = next.len() - 1;
        enforce_no_change(&mut result[0..n], &current[1..], &next[0..n], op_flag);
    }

    fn enforce_drop4(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
        let n = next.len() - 4;
        enforce_no_change(&mut result[0..n], &current[4..], &next[0..n], op_flag);
    }

    fn enforce_swap(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
        result[0] = agg_op_constraint(result[0], op_flag, T::sub(next[0], current[1]));
        result[1] = agg_op_constraint(result[0], op_flag, T::sub(next[1], current[0]));
        enforce_no_change(&mut result[2..], &current[2..], &next[2..], op_flag);
    }
    
    fn enforce_swap2(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
        result[0] = agg_op_constraint(result[0], op_flag, T::sub(next[0], current[2]));
        result[1] = agg_op_constraint(result[1], op_flag, T::sub(next[1], current[3]));
        result[2] = agg_op_constraint(result[2], op_flag, T::sub(next[2], current[0]));
        result[3] = agg_op_constraint(result[3], op_flag, T::sub(next[3], current[1]));
        enforce_no_change(&mut result[4..], &current[4..], &next[4..], op_flag);
    }
    
    fn enforce_swap4(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
        result[0] = agg_op_constraint(result[0], op_flag, T::sub(next[0], current[4]));
        result[1] = agg_op_constraint(result[1], op_flag, T::sub(next[1], current[5]));
        result[2] = agg_op_constraint(result[2], op_flag, T::sub(next[2], current[6]));
        result[3] = agg_op_constraint(result[3], op_flag, T::sub(next[3], current[7]));
        result[4] = agg_op_constraint(result[4], op_flag, T::sub(next[4], current[0]));
        result[5] = agg_op_constraint(result[5], op_flag, T::sub(next[5], current[1]));
        result[6] = agg_op_constraint(result[6], op_flag, T::sub(next[6], current[2]));
        result[7] = agg_op_constraint(result[7], op_flag, T::sub(next[7], current[3]));
        enforce_no_change(&mut result[8..], &current[8..], &next[8..], op_flag);
    }

    fn enforce_roll4(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
        result[0] = agg_op_constraint(result[0], op_flag, T::sub(next[0], current[3]));
        result[1] = agg_op_constraint(result[1], op_flag, T::sub(next[1], current[0]));
        result[2] = agg_op_constraint(result[2], op_flag, T::sub(next[2], current[1]));
        result[3] = agg_op_constraint(result[3], op_flag, T::sub(next[3], current[2]));
        enforce_no_change(&mut result[4..], &current[4..], &next[4..], op_flag);
    }
    
    fn enforce_roll8(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
        result[0] = agg_op_constraint(result[0], op_flag, T::sub(next[0], current[7]));
        result[1] = agg_op_constraint(result[1], op_flag, T::sub(next[1], current[0]));
        result[2] = agg_op_constraint(result[2], op_flag, T::sub(next[2], current[1]));
        result[3] = agg_op_constraint(result[3], op_flag, T::sub(next[3], current[2]));
        result[4] = agg_op_constraint(result[4], op_flag, T::sub(next[4], current[3]));
        result[5] = agg_op_constraint(result[5], op_flag, T::sub(next[5], current[4]));
        result[6] = agg_op_constraint(result[6], op_flag, T::sub(next[6], current[5]));
        result[7] = agg_op_constraint(result[7], op_flag, T::sub(next[7], current[6]));
        enforce_no_change(&mut result[8..], &current[8..], &next[8..], op_flag);
    }

    fn enforce_pad2(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
        result[0] = agg_op_constraint(result[0], op_flag, next[0]);
        result[1] = agg_op_constraint(result[1], op_flag, next[1]);
        enforce_no_change(&mut result[2..], &current[0..], &next[2..], op_flag);
    }
    
    fn enforce_dup(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
        result[0] = agg_op_constraint(result[0], op_flag, T::sub(next[0], current[0]));
        enforce_no_change(&mut result[1..], &current[0..], &next[1..], op_flag);
    }
    
    fn enforce_dup2(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
        result[0] = agg_op_constraint(result[0], op_flag, T::sub(next[0], current[0]));
        result[1] = agg_op_constraint(result[1], op_flag, T::sub(next[1], current[1]));
        enforce_no_change(&mut result[2..], &current[0..], &next[2..], op_flag);
    }
    
    fn enforce_dup4(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
        result[0] = agg_op_constraint(result[0], op_flag, T::sub(next[0], current[0]));
        result[1] = agg_op_constraint(result[1], op_flag, T::sub(next[1], current[1]));
        result[2] = agg_op_constraint(result[2], op_flag, T::sub(next[2], current[2]));
        result[3] = agg_op_constraint(result[3], op_flag, T::sub(next[3], current[3]));
        enforce_no_change(&mut result[4..], &current[0..], &next[4..], op_flag);
    }
    
    fn enforce_add(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
        let n = next.len() - 1;
        let op_result = T::add(current[0], current[1]);
        result[0] = agg_op_constraint(result[0], op_flag, T::sub(next[0], op_result));
        enforce_no_change(&mut result[1..n], &current[2..], &next[1..n], op_flag);
    }
    
    fn enforce_mul(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
        let n = next.len() - 1;
        let op_result = T::mul(current[1], current[0]);
        result[0] = agg_op_constraint(result[0], op_flag, T::sub(next[0], op_result));
        enforce_no_change(&mut result[1..n], &current[2..], &next[1..n], op_flag);
    }

    fn enforce_inv(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
        result[0] = agg_op_constraint(result[0], op_flag, T::sub(T::ONE, T::mul(next[0], current[0])));
        enforce_no_change(&mut result[1..], &current[1..], &next[1..], op_flag);
    }

    fn enforce_neg(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
        result[0] = agg_op_constraint(result[0], op_flag, T::add(next[0], current[0]));
        enforce_no_change(&mut result[1..], &current[1..], &next[1..], op_flag);
    }

    // LOGIC OPS
    // --------------------------------------------------------------------------------------------

    /// Evaluates transition constraints for operations where some operands must be binary values.
    fn enforce_logic_ops(&self, current: &[T], next: &[T], op_flags: [T; NUM_LD_OPS], result: &mut [T]) {

        // TODO: transform into a generic way to handle aux constraints/registers
        let aux = current[0];

        // logic operations work only with the user portion of the stack
        let current = &current[AUX_WIDTH..];
        let next = &next[AUX_WIDTH..];

        let mut evaluations = vec![T::ZERO; current.len()];

        // CHOOSE
        let op_flag = op_flags[opcodes::CHOOSE as usize];
        let n = next.len() - 2;
        let condition1 = current[2];
        let condition2 = T::sub(T::ONE, condition1);
        let op_result = T::add(T::mul(condition1, current[0]), T::mul(condition2, current[1]));
        evaluations[0] = agg_op_constraint(evaluations[0], op_flag, T::sub(next[0], op_result));
        enforce_no_change(&mut evaluations[1..n], &current[3..], &next[1..n], op_flag);
        result[0] = agg_op_constraint(result[0], op_flag, is_binary(condition1));

        // CHOOSE2
        let op_flag = op_flags[opcodes::CHOOSE2 as usize];
        let n = next.len() - 4;
        let condition1 = current[4];
        let condition2 = T::sub(T::ONE, condition1);
        let op_result1 = T::add(T::mul(condition1, current[0]), T::mul(condition2, current[2]));
        let op_result2 = T::add(T::mul(condition1, current[1]), T::mul(condition2, current[3]));
        evaluations[0] = agg_op_constraint(evaluations[0], op_flag, T::sub(next[0], op_result1));
        evaluations[1] = agg_op_constraint(evaluations[1], op_flag, T::sub(next[1], op_result2));
        enforce_no_change(&mut evaluations[2..n], &current[6..], &next[2..n], op_flag);
        result[0] = agg_op_constraint(result[0], op_flag, is_binary(condition1));

        // NOT
        let op_flag = op_flags[opcodes::NOT as usize];
        let op_result = T::sub(T::ONE, current[0]);
        evaluations[0] = agg_op_constraint(evaluations[0], op_flag, T::sub(next[0], op_result));
        enforce_no_change(&mut evaluations[1..n], &current[1..], &next[1..n], op_flag);
        result[0] = agg_op_constraint(result[0], op_flag, is_binary(current[0]));

        // EQ
        let aux_constraint = enforce_eq(&mut evaluations, current, next, aux, op_flags[opcodes::EQ as usize]);
        result[0] = T::add(result[0], aux_constraint);
        
        // CMP
        let aux_constraint = enforce_cmp(&mut evaluations, current, next, aux, op_flags[opcodes::CMP as usize]);
        result[0] = T::add(result[0], aux_constraint);
        

        let result = &mut result[AUX_WIDTH..];
        for i in 0..result.len() {
            result[i] = T::add(result[i], evaluations[i]);
        }
    }
}