use crate::{
    math::{ field, polynom },
    processor::OpCode,
    stark::TraceState,
    utils::hasher::ARK,
    NUM_LD_OPS, NUM_HD_OPS, BASE_CYCLE_LENGTH, HASH_STATE_WIDTH
};
use super::utils::{ are_equal, is_binary, binary_not, extend_constants, EvaluationResult };

mod arithmetic;
use arithmetic::{ enforce_add, enforce_mul, enforce_inv, enforce_neg };

mod boolean;
use boolean::{ enforce_not, enforce_and, enforce_or };

mod comparison;
use comparison::{ enforce_eq, enforce_cmp, enforce_binacc };

mod selection;
use selection::{ enforce_choose, enforce_choose2 };

mod hash;
use hash::{ enforce_rescr };

mod utils;
use utils::{ enforce_no_change };

// CONSTANTS
// ================================================================================================
const NUM_AUX_CONSTRAINTS: usize = 2;
const AUX_CONSTRAINT_DEGREES: [usize; NUM_AUX_CONSTRAINTS] = [7, 7];

const STACK_HEAD_DEGREES: [usize; 7] = [
    7, 7, 7, 7, 7, 7, 7,    // constraints for the first 7 registers of user stack
];
const STACK_REST_DEGREE: usize = 6; // degree for the rest of the stack registers

// TYPES AND INTERFACES
// ================================================================================================
pub struct Stack {
    trace_length        : usize,
    cycle_length        : usize,
    ark_values          : Vec<[u128; 2 * HASH_STATE_WIDTH]>,
    ark_polys           : Vec<Vec<u128>>,
    constraint_degrees  : Vec<usize>,
}

// STACK CONSTRAINT EVALUATOR IMPLEMENTATION
// ================================================================================================
impl Stack {

    pub fn new(trace_length: usize, extension_factor: usize, stack_depth: usize) -> Stack 
    {
        // build an array of constraint degrees for the stack
        let mut degrees = Vec::from(&AUX_CONSTRAINT_DEGREES[..]);
        degrees.extend_from_slice(&STACK_HEAD_DEGREES[..]);
        degrees.resize(stack_depth + NUM_AUX_CONSTRAINTS, STACK_REST_DEGREE);

        // determine extended cycle length
        let cycle_length = BASE_CYCLE_LENGTH * extension_factor;

        // extend rounds constants by the specified extension factor
        let (ark_polys, ark_evaluations) = extend_constants(&ARK, extension_factor);
        let ark_values = transpose_ark_constants(ark_evaluations, cycle_length);

        return Stack {
            trace_length, cycle_length,
            ark_values, ark_polys,
            constraint_degrees: degrees,
        };
    }

    pub fn constraint_degrees(&self) -> &[usize] {
        return &self.constraint_degrees;
    }

    // EVALUATOR FUNCTIONS
    // --------------------------------------------------------------------------------------------

    /// Evaluates stack transition constraints at the specified step of the evaluation domain and
    /// saves the evaluations into `result`.
    pub fn evaluate(&self, current: &TraceState, next: &TraceState, step: usize, result: &mut [u128])
    {
        // determine round constants at the specified step
        let ark = self.ark_values[step % self.cycle_length];

        // get user stack registers from current and next steps
        let old_stack = current.user_stack();
        let new_stack = next.user_stack();

        // evaluate constraints for simple operations
        let op_flags = current.ld_op_flags();
        self.enforce_low_degree_ops(old_stack, new_stack, op_flags, result);

        // evaluate constraints for hash operation
        let op_flags = current.hd_op_flags();
        self.enforce_high_degree_ops(old_stack, new_stack, &ark, op_flags, result);
    }

    /// Evaluates stack transition constraints at the specified x coordinate and saves the
    /// evaluations into `result`. Unlike the function above, this function can evaluate constraints
    /// at any out-of-domain point, but it is much slower than the previous function.
    pub fn evaluate_at(&self, current: &TraceState, next: &TraceState, x: u128, result: &mut [u128])
    {
        // map x to the corresponding coordinate in constant cycles
        let num_cycles = (self.trace_length / BASE_CYCLE_LENGTH) as u128;
        let x = field::exp(x, num_cycles);

        // determine round constants at the specified x coordinate
        let mut ark = [field::ZERO; 2 * HASH_STATE_WIDTH];
        for i in 0..ark.len() {
            ark[i] = polynom::eval(&self.ark_polys[i], x);
        }

        // get user stack registers from current and next steps
        let old_stack = current.user_stack();
        let new_stack = next.user_stack();

        // evaluate constraints for low_degree operations
        let op_flags = current.ld_op_flags();
        self.enforce_low_degree_ops(old_stack, new_stack, op_flags, result);

        // evaluate constraints for hash operation
        let op_flags = current.hd_op_flags();
        self.enforce_high_degree_ops(old_stack, new_stack, &ark, op_flags, result);
    }

    /// Evaluates transition constraints for all operations where the operation result does not
    /// depend on the where in the execution trace it is executed. In other words, these operations
    /// are not tied to any repeating cycles in the execution trace.
    fn enforce_low_degree_ops(&self, current: &[u128], next: &[u128], op_flags: [u128; NUM_LD_OPS], result: &mut [u128]) {
        
        // split constraint evaluation result into aux constraints and stack constraints
        let (aux, result) = result.split_at_mut(NUM_AUX_CONSTRAINTS);

        // initialize a vector to hold stack constraint evaluations; this is needed because
        // constraint evaluator functions assume that the stack is at least 8 items deep; while
        // it may actually be smaller than that
        let mut evaluations = vec![field::ZERO; current.len()];

        // control flow operations
        //enforce_no_change(&mut evaluations,     current, next, op_flags[OpCode::Noop.ld_index()]);
        enforce_assert  (&mut evaluations, aux, current, next, op_flags[OpCode::Assert.ld_index()]);
        enforce_asserteq(&mut evaluations, aux, current, next, op_flags[OpCode::AssertEq.ld_index()]);

        // input operations
        enforce_read    (&mut evaluations,      current, next, op_flags[OpCode::Read.ld_index()]);
        enforce_read2   (&mut evaluations,      current, next, op_flags[OpCode::Read2.ld_index()]);
    
        // stack manipulation operations
        enforce_dup     (&mut evaluations,      current, next, op_flags[OpCode::Dup.ld_index()]);
        enforce_dup2    (&mut evaluations,      current, next, op_flags[OpCode::Dup2.ld_index()]);
        enforce_dup4    (&mut evaluations,      current, next, op_flags[OpCode::Dup4.ld_index()]);
        enforce_pad2    (&mut evaluations,      current, next, op_flags[OpCode::Pad2.ld_index()]);

        enforce_drop    (&mut evaluations,      current, next, op_flags[OpCode::Drop.ld_index()]);
        enforce_drop4   (&mut evaluations,      current, next, op_flags[OpCode::Drop4.ld_index()]);
        
        enforce_swap    (&mut evaluations,      current, next, op_flags[OpCode::Swap.ld_index()]);
        enforce_swap2   (&mut evaluations,      current, next, op_flags[OpCode::Swap2.ld_index()]);
        enforce_swap4   (&mut evaluations,      current, next, op_flags[OpCode::Swap4.ld_index()]);
    
        enforce_roll4   (&mut evaluations,      current, next, op_flags[OpCode::Roll4.ld_index()]);
        enforce_roll8   (&mut evaluations,      current, next, op_flags[OpCode::Roll8.ld_index()]);
    
        // arithmetic and boolean operations
        enforce_add     (&mut evaluations,      current, next, op_flags[OpCode::Add.ld_index()]);
        enforce_mul     (&mut evaluations,      current, next, op_flags[OpCode::Mul.ld_index()]);
        enforce_inv     (&mut evaluations,      current, next, op_flags[OpCode::Inv.ld_index()]);
        enforce_neg     (&mut evaluations,      current, next, op_flags[OpCode::Neg.ld_index()]);
        enforce_not     (&mut evaluations, aux, current, next, op_flags[OpCode::Not.ld_index()]);
        enforce_and     (&mut evaluations, aux, current, next, op_flags[OpCode::And.ld_index()]);
        enforce_or      (&mut evaluations, aux, current, next, op_flags[OpCode::Or.ld_index()]);
        
        // comparison operations
        enforce_eq      (&mut evaluations, aux, current, next, op_flags[OpCode::Eq.ld_index()]);
        enforce_binacc  (&mut evaluations,      current, next, op_flags[OpCode::BinAcc.ld_index()]);

        // conditional selection operations
        enforce_choose  (&mut evaluations, aux, current, next, op_flags[OpCode::Choose.ld_index()]);
        enforce_choose2 (&mut evaluations, aux, current, next, op_flags[OpCode::Choose2.ld_index()]);

        // copy evaluations into the result
        for i in 0..result.len() {
            result[i] = field::add(result[i], evaluations[i]);
        }
    }

    fn enforce_high_degree_ops(&self, current: &[u128], next: &[u128], ark: &[u128], op_flags: [u128; NUM_HD_OPS], result: &mut [u128])
    {
        // high-degree operations don't use aux constraints
        let (_, result) = result.split_at_mut(NUM_AUX_CONSTRAINTS);

        // initialize a vector to hold stack constraint evaluations; this is needed because
        // constraint evaluator functions assume that the stack is at least 8 items deep; while
        // it may actually be smaller than that
        let mut evaluations = vec![field::ZERO; current.len()];

        enforce_push (&mut evaluations, current, next,      op_flags[OpCode::Push.hd_index() ]);
        enforce_cmp  (&mut evaluations, current, next,      op_flags[OpCode::Cmp.hd_index()  ]);
        enforce_rescr(&mut evaluations, current, next, ark, op_flags[OpCode::RescR.hd_index()]);

        // copy evaluations into the result
        for i in 0..result.len() {
            result[i] = field::add(result[i], evaluations[i]);
        }
    }
}

// CONTROL FLOW OPERATIONS
// ================================================================================================

/// Enforces constraints for ASSERT operation. The constraints are similar to DROP operation, but
/// have an auxiliary constraint which enforces that 1 - x = 0, where x is the top of the stack.
fn enforce_assert(result: &mut [u128], aux: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    let n = next.len() - 1;
    enforce_no_change(&mut result[0..n], &current[1..], &next[0..n], op_flag);
    aux.agg_constraint(0, op_flag, are_equal(field::ONE, current[0]));
}

/// Enforces constraints for ASSERTEQ operation. The stack is shifted by 2 registers the left and
/// an auxiliary constraint enforces that the first element of the stack is equal to the second.
fn enforce_asserteq(result: &mut [u128], aux: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    let n = next.len() - 2;
    enforce_no_change(&mut result[0..n], &current[2..], &next[0..n], op_flag);
    aux.agg_constraint(0, op_flag, are_equal(current[0], current[1]));
}

// INPUT OPERATIONS
// ================================================================================================

/// Enforces constraints for PUSH operation. The constraints are based on the first element of the 
/// stack; the old stack is shifted right by 1 element.
fn enforce_push(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {

    // ensure that the rest of the stack is shifted right by 1 element
    enforce_no_change(&mut result[1..], &current[0..], &next[1..], op_flag);
}

/// Enforces constraints for READ operation. No constraints are placed on the first element of
/// the stack; the old stack is shifted right by 1 element.
fn enforce_read(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    enforce_no_change(&mut result[1..], &current[0..], &next[1..], op_flag);
}

/// Enforces constraints for READ2 operation. No constraints are placed on the first two elements
/// of the stack; the old stack is shifted right by 2 element.
fn enforce_read2(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    enforce_no_change(&mut result[2..], &current[0..], &next[2..], op_flag);
}

// STACK MANIPULATION OPERATIONS
// ================================================================================================

/// Enforces constraints for DUP operation. The constraints are based on the first element
/// of the stack; the old stack is shifted right by 1 element.
fn enforce_dup(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    result.agg_constraint(0, op_flag, are_equal(next[0], current[0]));
    enforce_no_change(&mut result[1..], &current[0..], &next[1..], op_flag);
}

/// Enforces constraints for DUP2 operation. The constraints are based on the first 2 element
/// of the stack; the old stack is shifted right by 2 element.
fn enforce_dup2(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    result.agg_constraint(0, op_flag, are_equal(next[0], current[0]));
    result.agg_constraint(1, op_flag, are_equal(next[1], current[1]));
    enforce_no_change(&mut result[2..], &current[0..], &next[2..], op_flag);
}

/// Enforces constraints for DUP4 operation. The constraints are based on the first 4 element
/// of the stack; the old stack is shifted right by 4 element.
fn enforce_dup4(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    result.agg_constraint(0, op_flag, are_equal(next[0], current[0]));
    result.agg_constraint(1, op_flag, are_equal(next[1], current[1]));
    result.agg_constraint(2, op_flag, are_equal(next[2], current[2]));
    result.agg_constraint(3, op_flag, are_equal(next[3], current[3]));
    enforce_no_change(&mut result[4..], &current[0..], &next[4..], op_flag);
}

/// Enforces constraints for PAD2 operation. The constraints are based on the first 2 element
/// of the stack; the old stack is shifted right by 2 element.
fn enforce_pad2(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    result.agg_constraint(0, op_flag, next[0]);
    result.agg_constraint(1, op_flag, next[1]);
    enforce_no_change(&mut result[2..], &current[0..], &next[2..], op_flag);
}

// Enforces constraints for DROP operation. The stack is simply shifted left by 1 element.
fn enforce_drop(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    let n = next.len() - 1;
    enforce_no_change(&mut result[0..n], &current[1..], &next[0..n], op_flag);
}

// Enforces constraints for DROP4 operation. The stack is simply shifted left by 4 element.
fn enforce_drop4(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    let n = next.len() - 4;
    enforce_no_change(&mut result[0..n], &current[4..], &next[0..n], op_flag);
}

/// Enforces constraints for SWAP operation. The constraints are based on the first 2 element
/// of the stack; the rest of the stack is unaffected.
fn enforce_swap(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    result.agg_constraint(0, op_flag, are_equal(next[0], current[1]));
    result.agg_constraint(0, op_flag, are_equal(next[1], current[0]));
    enforce_no_change(&mut result[2..], &current[2..], &next[2..], op_flag);
}

/// Enforces constraints for SWAP2 operation. The constraints are based on the first 4 element
/// of the stack; the rest of the stack is unaffected.
fn enforce_swap2(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    result.agg_constraint(0, op_flag, are_equal(next[0], current[2]));
    result.agg_constraint(1, op_flag, are_equal(next[1], current[3]));
    result.agg_constraint(2, op_flag, are_equal(next[2], current[0]));
    result.agg_constraint(3, op_flag, are_equal(next[3], current[1]));
    enforce_no_change(&mut result[4..], &current[4..], &next[4..], op_flag);
}

/// Enforces constraints for SWAP4 operation. The constraints are based on the first 8 element
/// of the stack; the rest of the stack is unaffected.
fn enforce_swap4(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    result.agg_constraint(0, op_flag, are_equal(next[0], current[4]));
    result.agg_constraint(1, op_flag, are_equal(next[1], current[5]));
    result.agg_constraint(2, op_flag, are_equal(next[2], current[6]));
    result.agg_constraint(3, op_flag, are_equal(next[3], current[7]));
    result.agg_constraint(4, op_flag, are_equal(next[4], current[0]));
    result.agg_constraint(5, op_flag, are_equal(next[5], current[1]));
    result.agg_constraint(6, op_flag, are_equal(next[6], current[2]));
    result.agg_constraint(7, op_flag, are_equal(next[7], current[3]));
    enforce_no_change(&mut result[8..], &current[8..], &next[8..], op_flag);
}

/// Enforces constraints for ROLL4 operation. The constraints are based on the first 4 element
/// of the stack; the rest of the stack is unaffected.
fn enforce_roll4(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    result.agg_constraint(0, op_flag, are_equal(next[0], current[3]));
    result.agg_constraint(1, op_flag, are_equal(next[1], current[0]));
    result.agg_constraint(2, op_flag, are_equal(next[2], current[1]));
    result.agg_constraint(3, op_flag, are_equal(next[3], current[2]));
    enforce_no_change(&mut result[4..], &current[4..], &next[4..], op_flag);
}

/// Enforces constraints for ROLL8 operation. The constraints are based on the first 8 element
/// of the stack; the rest of the stack is unaffected.
fn enforce_roll8(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    result.agg_constraint(0, op_flag, are_equal(next[0], current[7]));
    result.agg_constraint(1, op_flag, are_equal(next[1], current[0]));
    result.agg_constraint(2, op_flag, are_equal(next[2], current[1]));
    result.agg_constraint(3, op_flag, are_equal(next[3], current[2]));
    result.agg_constraint(4, op_flag, are_equal(next[4], current[3]));
    result.agg_constraint(5, op_flag, are_equal(next[5], current[4]));
    result.agg_constraint(6, op_flag, are_equal(next[6], current[5]));
    result.agg_constraint(7, op_flag, are_equal(next[7], current[6]));
    enforce_no_change(&mut result[8..], &current[8..], &next[8..], op_flag);
}

// HELPER FUNCTIONS
// ================================================================================================
fn transpose_ark_constants(constants: Vec<Vec<u128>>, cycle_length: usize) -> Vec<[u128; 2 * HASH_STATE_WIDTH]>
{
    let mut values = Vec::new();
    for i in 0..cycle_length {
        values.push([field::ZERO; 2 * HASH_STATE_WIDTH]);
        for j in 0..(2 * HASH_STATE_WIDTH) {
            values[i][j] = constants[j][i];
        }
    }
    return values;
}