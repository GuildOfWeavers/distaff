use crate::{
    math::{ field, polynom },
    processor::OpCode,
    stark::TraceState,
    utils::hasher::ARK,
    NUM_LD_OPS, NUM_HD_OPS, BASE_CYCLE_LENGTH, HASH_STATE_WIDTH
};
use super::utils::{
    are_equal, is_binary, binary_not, extend_constants, EvaluationResult,
    enforce_stack_copy, enforce_left_shift, enforce_right_shift,
};

mod input;
use input::{ enforce_push, enforce_read, enforce_read2 };

mod arithmetic;
use arithmetic::{
    enforce_add, enforce_mul, enforce_inv, enforce_neg,
    enforce_not, enforce_and, enforce_or,
};

mod manipulation;
use manipulation::{
    enforce_dup, enforce_dup2, enforce_dup4, enforce_pad2, enforce_drop, enforce_drop4,
    enforce_swap, enforce_swap2, enforce_swap4, enforce_roll4, enforce_roll8,
};

mod comparison;
use comparison::{ enforce_assert, enforce_asserteq, enforce_eq, enforce_cmp, enforce_binacc };

mod selection;
use selection::{ enforce_choose, enforce_choose2 };

mod hash;
use hash::{ enforce_rescr };

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
    fn enforce_low_degree_ops(&self, current: &[u128], next: &[u128], op_flags: [u128; NUM_LD_OPS], result: &mut [u128])
    {    
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