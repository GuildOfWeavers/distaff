use crate::math::{ FiniteField };
use crate::processor::{ opcodes };
use crate::stark::{ TraceState };
use crate::utils::{ Hasher, Accumulator };
use crate::{ NUM_LD_OPS };

mod comparisons;
mod hashing;
mod selections;
mod utils;

use hashing::HashEvaluator;
use comparisons::{ enforce_eq, enforce_cmp, enforce_binacc };
use selections::{ enforce_choose, enforce_choose2 };
use utils::{ agg_op_constraint, enforce_no_change, are_equal, is_binary };

// CONSTANTS
// ================================================================================================
const STACK_HEAD_DEGREES: [usize; 7] = [
    7,                  // aux constraints
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
        self.enforce_acyclic_ops(current_stack, next_stack, op_flags, next_op, result);

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
        self.enforce_acyclic_ops(current_stack, next_stack, op_flags, next_op, result);

        // evaluate constraints for hash operation
        let hash_flag = op_flags[opcodes::HASHR as usize];
        self.hash_evaluator.evaluate_at(current_stack, next_stack, x, hash_flag, result);
    }

    /// Evaluates transition constraints for all operations where the operation result does not
    /// depend on the where in the execution trace it is executed. In other words, these operations
    /// are not tied to any repeating cycles in the execution trace.
    fn enforce_acyclic_ops(&self, current: &[T], next: &[T], op_flags: [T; NUM_LD_OPS], next_op: T, result: &mut [T]) {
        
        // save the aux register of the stack
        let aux = current[0];

        // trim stack states only to the user portion of the stack (excluding aux register)
        // TODO: use a separate constant for user stack offset
        let current = &current[1..];
        let next = &next[1..];

        // initialize a vector to hold constraint evaluations; this is needed because constraint
        // evaluator functions assume that the stack is at least 8 items deep; while it may
        // actually be smaller than that
        let mut evaluations = vec![T::ZERO; current.len()];

        // control flow operations
        enforce_no_change(&mut evaluations, current, next, op_flags[opcodes::BEGIN as usize]);
        enforce_no_change(&mut evaluations, current, next, op_flags[opcodes::NOOP as usize]);
        result[0] = T::add(result[0],
            enforce_assert(&mut evaluations, current, next, op_flags[opcodes::ASSERT as usize]));

        // input operations
        enforce_push(&mut evaluations,      current, next, next_op, op_flags[opcodes::PUSH as usize]);
        enforce_read(&mut evaluations,      current, next, op_flags[opcodes::READ as usize]);
        enforce_read2(&mut evaluations,     current, next, op_flags[opcodes::READ2 as usize]);
    
        // stack manipulation operations
        enforce_dup(&mut evaluations,       current, next, op_flags[opcodes::DUP as usize]);
        enforce_dup2(&mut evaluations,      current, next, op_flags[opcodes::DUP2 as usize]);
        enforce_dup4(&mut evaluations,      current, next, op_flags[opcodes::DUP4 as usize]);
        enforce_pad2(&mut evaluations,      current, next, op_flags[opcodes::PAD2 as usize]);

        enforce_drop(&mut evaluations,      current, next, op_flags[opcodes::DROP as usize]);
        enforce_drop4(&mut evaluations,     current, next, op_flags[opcodes::DROP4 as usize]);
        
        enforce_swap(&mut evaluations,      current, next, op_flags[opcodes::SWAP as usize]);
        enforce_swap2(&mut evaluations,     current, next, op_flags[opcodes::SWAP2 as usize]);
        enforce_swap4(&mut evaluations,     current, next, op_flags[opcodes::SWAP4 as usize]);
    
        enforce_roll4(&mut evaluations,     current, next, op_flags[opcodes::ROLL4 as usize]);
        enforce_roll8(&mut evaluations,     current, next, op_flags[opcodes::ROLL8 as usize]);
    
        // arithmetic and boolean operations
        enforce_add(&mut evaluations,       current, next, op_flags[opcodes::ADD as usize]);
        enforce_mul(&mut evaluations,       current, next, op_flags[opcodes::MUL as usize]);
        enforce_inv(&mut evaluations,       current, next, op_flags[opcodes::INV as usize]);
        enforce_neg(&mut evaluations,       current, next, op_flags[opcodes::NEG as usize]);
        result[0] = T::add(result[0],
            enforce_not(&mut evaluations,   current, next, op_flags[opcodes::NOT as usize]));
        
        // comparison operations
        result[0] = T::add(result[0],
            enforce_eq(&mut evaluations,      current, next, aux, op_flags[opcodes::EQ as usize]));
        result[0] = T::add(result[0],
            enforce_cmp(&mut evaluations,     current, next, aux, op_flags[opcodes::CMP as usize]));
        result[0] = T::add(result[0],
            enforce_binacc(&mut evaluations,  current, next, aux, op_flags[opcodes::BINACC as usize]));

        // conditional selection operations
        result[0] = T::add(result[0],
            enforce_choose(&mut evaluations,  current, next, op_flags[opcodes::CHOOSE as usize]));
        result[0] = T::add(result[0],
            enforce_choose2(&mut evaluations, current, next, op_flags[opcodes::CHOOSE2 as usize]));

        // copy evaluations into the result while skipping the aux constraint because it
        // is already updated in the result vector
        let result = &mut result[1..];  // TODO: use constant
        for i in 0..result.len() {
            result[i] = T::add(result[i], evaluations[i]);
        }
    }
}

// CONTROL FLOW OPERATIONS
// ================================================================================================

/// Enforces constraints for ASSERT operation. The constraints are similar to DROP operation, but
/// have an auxiliary constraint which enforces that 1 - x = 0, where x is the top of the stack.
fn enforce_assert<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) -> T {
    let n = next.len() - 1;
    enforce_no_change(&mut result[0..n], &current[1..], &next[0..n], op_flag);
    return T::mul(op_flag, T::sub(T::ONE, current[0]));
}

// INPUT OPERATIONS
// ================================================================================================

/// Enforces constraints for PUSH operation. The constraints are based on the first element of the 
/// stack; the old stack is shifted right by 1 element.
fn enforce_push<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_code: T, op_flag: T) {

    // op_code becomes the first element on the stack
    result[0] = agg_op_constraint(result[0], op_flag, are_equal(next[0], op_code));

    // ensure that the rest of the stack is shifted right by 1 element
    enforce_no_change(&mut result[1..], &current[0..], &next[1..], op_flag);
}

/// Enforces constraints for READ operation. No constraints are placed on the first element of
/// the stack; the old stack is shifted right by 1 element.
fn enforce_read<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
    enforce_no_change(&mut result[1..], &current[0..], &next[1..], op_flag);
}

/// Enforces constraints for READ2 operation. No constraints are placed on the first two elements
/// of the stack; the old stack is shifted right by 2 element.
fn enforce_read2<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
    enforce_no_change(&mut result[2..], &current[0..], &next[2..], op_flag);
}

// STACK MANIPULATION OPERATIONS
// ================================================================================================

/// Enforces constraints for DUP operation. The constraints are based on the first element
/// of the stack; the old stack is shifted right by 1 element.
fn enforce_dup<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
    result[0] = agg_op_constraint(result[0], op_flag, are_equal(next[0], current[0]));
    enforce_no_change(&mut result[1..], &current[0..], &next[1..], op_flag);
}

/// Enforces constraints for DUP2 operation. The constraints are based on the first 2 element
/// of the stack; the old stack is shifted right by 2 element.
fn enforce_dup2<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
    result[0] = agg_op_constraint(result[0], op_flag, are_equal(next[0], current[0]));
    result[1] = agg_op_constraint(result[1], op_flag, are_equal(next[1], current[1]));
    enforce_no_change(&mut result[2..], &current[0..], &next[2..], op_flag);
}

/// Enforces constraints for DUP4 operation. The constraints are based on the first 4 element
/// of the stack; the old stack is shifted right by 4 element.
fn enforce_dup4<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
    result[0] = agg_op_constraint(result[0], op_flag, are_equal(next[0], current[0]));
    result[1] = agg_op_constraint(result[1], op_flag, are_equal(next[1], current[1]));
    result[2] = agg_op_constraint(result[2], op_flag, are_equal(next[2], current[2]));
    result[3] = agg_op_constraint(result[3], op_flag, are_equal(next[3], current[3]));
    enforce_no_change(&mut result[4..], &current[0..], &next[4..], op_flag);
}

/// Enforces constraints for PAD2 operation. The constraints are based on the first 2 element
/// of the stack; the old stack is shifted right by 2 element.
fn enforce_pad2<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
    result[0] = agg_op_constraint(result[0], op_flag, next[0]);
    result[1] = agg_op_constraint(result[1], op_flag, next[1]);
    enforce_no_change(&mut result[2..], &current[0..], &next[2..], op_flag);
}

// Enforces constraints for DROP operation. The stack is simply shifted left by 1 element.
fn enforce_drop<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
    let n = next.len() - 1;
    enforce_no_change(&mut result[0..n], &current[1..], &next[0..n], op_flag);
}

// Enforces constraints for DROP4 operation. The stack is simply shifted left by 4 element.
fn enforce_drop4<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
    let n = next.len() - 4;
    enforce_no_change(&mut result[0..n], &current[4..], &next[0..n], op_flag);
}

/// Enforces constraints for SWAP operation. The constraints are based on the first 2 element
/// of the stack; the rest of the stack is unaffected.
fn enforce_swap<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
    result[0] = agg_op_constraint(result[0], op_flag, are_equal(next[0], current[1]));
    result[1] = agg_op_constraint(result[0], op_flag, are_equal(next[1], current[0]));
    enforce_no_change(&mut result[2..], &current[2..], &next[2..], op_flag);
}

/// Enforces constraints for SWAP2 operation. The constraints are based on the first 4 element
/// of the stack; the rest of the stack is unaffected.
fn enforce_swap2<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
    result[0] = agg_op_constraint(result[0], op_flag, are_equal(next[0], current[2]));
    result[1] = agg_op_constraint(result[1], op_flag, are_equal(next[1], current[3]));
    result[2] = agg_op_constraint(result[2], op_flag, are_equal(next[2], current[0]));
    result[3] = agg_op_constraint(result[3], op_flag, are_equal(next[3], current[1]));
    enforce_no_change(&mut result[4..], &current[4..], &next[4..], op_flag);
}

/// Enforces constraints for SWAP4 operation. The constraints are based on the first 8 element
/// of the stack; the rest of the stack is unaffected.
fn enforce_swap4<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
    result[0] = agg_op_constraint(result[0], op_flag, are_equal(next[0], current[4]));
    result[1] = agg_op_constraint(result[1], op_flag, are_equal(next[1], current[5]));
    result[2] = agg_op_constraint(result[2], op_flag, are_equal(next[2], current[6]));
    result[3] = agg_op_constraint(result[3], op_flag, are_equal(next[3], current[7]));
    result[4] = agg_op_constraint(result[4], op_flag, are_equal(next[4], current[0]));
    result[5] = agg_op_constraint(result[5], op_flag, are_equal(next[5], current[1]));
    result[6] = agg_op_constraint(result[6], op_flag, are_equal(next[6], current[2]));
    result[7] = agg_op_constraint(result[7], op_flag, are_equal(next[7], current[3]));
    enforce_no_change(&mut result[8..], &current[8..], &next[8..], op_flag);
}

/// Enforces constraints for ROLL4 operation. The constraints are based on the first 4 element
/// of the stack; the rest of the stack is unaffected.
fn enforce_roll4<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
    result[0] = agg_op_constraint(result[0], op_flag, are_equal(next[0], current[3]));
    result[1] = agg_op_constraint(result[1], op_flag, are_equal(next[1], current[0]));
    result[2] = agg_op_constraint(result[2], op_flag, are_equal(next[2], current[1]));
    result[3] = agg_op_constraint(result[3], op_flag, are_equal(next[3], current[2]));
    enforce_no_change(&mut result[4..], &current[4..], &next[4..], op_flag);
}

/// Enforces constraints for ROLL8 operation. The constraints are based on the first 8 element
/// of the stack; the rest of the stack is unaffected.
fn enforce_roll8<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) {
    result[0] = agg_op_constraint(result[0], op_flag, are_equal(next[0], current[7]));
    result[1] = agg_op_constraint(result[1], op_flag, are_equal(next[1], current[0]));
    result[2] = agg_op_constraint(result[2], op_flag, are_equal(next[2], current[1]));
    result[3] = agg_op_constraint(result[3], op_flag, are_equal(next[3], current[2]));
    result[4] = agg_op_constraint(result[4], op_flag, are_equal(next[4], current[3]));
    result[5] = agg_op_constraint(result[5], op_flag, are_equal(next[5], current[4]));
    result[6] = agg_op_constraint(result[6], op_flag, are_equal(next[6], current[5]));
    result[7] = agg_op_constraint(result[7], op_flag, are_equal(next[7], current[6]));
    enforce_no_change(&mut result[8..], &current[8..], &next[8..], op_flag);
}

// ARITHMETIC and BOOLEAN OPERATION
// ================================================================================================

/// Enforces constraints for ADD operation. The constraints are based on the first 2 elements of
/// the stack; the rest of the stack is shifted left by 1 element.
fn enforce_add<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) {

    let x = current[0];
    let y = current[1];
    let op_result = T::add(x, y);
    result[0] = agg_op_constraint(result[0], op_flag, are_equal(next[0], op_result));

    // ensure that the rest of the stack is shifted left by 1 element
    let n = next.len() - 1;
    enforce_no_change(&mut result[1..n], &current[2..], &next[1..n], op_flag);
}

/// Enforces constraints for MUL operation. The constraints are based on the first 2 elements of
/// the stack; the rest of the stack is shifted left by 1 element.
fn enforce_mul<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) {

    let x = current[0];
    let y = current[1];
    let op_result = T::mul(x, y);
    result[0] = agg_op_constraint(result[0], op_flag, are_equal(next[0], op_result));

    // ensure that the rest of the stack is shifted left by 1 element
    let n = next.len() - 1;
    enforce_no_change(&mut result[1..n], &current[2..], &next[1..n], op_flag);
}

/// Enforces constraints for INV operation. The constraints are based on the first element of
/// the stack; the rest of the stack is unaffected.
fn enforce_inv<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) {

    // Constraints for INV operation is defined as: x * inv(x) = 1; this also means
    // that if x = 0, the constraint will not be satisfied
    let x = current[0];
    let inv_x = next[0];
    result[0] = agg_op_constraint(result[0], op_flag, are_equal(T::ONE, T::mul(inv_x, x)));

    // ensure nothing changed beyond the first item of the stack 
    enforce_no_change(&mut result[1..], &current[1..], &next[1..], op_flag);
}

/// Enforces constraints for NEG operation. The constraints are based on the first element of
/// the stack; the rest of the stack is unaffected.
fn enforce_neg<T: FiniteField>(result: &mut [T], current: &[T], next: &[T], op_flag: T) {

    // Constraint for NEG operation is defined as: x + neg(x) = 0
    let x = current[0];
    let neg_x = next[0];
    result[0] = agg_op_constraint(result[0], op_flag, T::add(neg_x, x));

    // ensure nothing changed beyond the first item of the stack 
    enforce_no_change(&mut result[1..], &current[1..], &next[1..], op_flag);
}

/// Enforces constraints for NOT operation. The constraints are based on the first element of
/// the stack, but also evaluates an auxiliary constraint which guarantees that the first
/// element of the stack is binary.
fn enforce_not<T: FiniteField>(evaluations: &mut [T], current: &[T], next: &[T], op_flag: T) -> T {

    // NOT operation is defined simply as: 1 - x; this means 0 becomes 1, and 1 becomes 0
    let x = current[0];
    let op_result = T::sub(T::ONE, x);
    evaluations[0] = agg_op_constraint(evaluations[0], op_flag, are_equal(next[0], op_result));

    // ensure nothing changed beyond the first item of the stack 
    enforce_no_change(&mut evaluations[1..], &current[1..], &next[1..], op_flag);

    // we also need to make sure that the operand is binary (i.e. 0 or 1)
    return T::mul(op_flag, is_binary(x));
}