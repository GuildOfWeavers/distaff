use crate::math::{ FiniteField, polynom };
use crate::processor::{ opcodes };
use crate::utils::{ accumulator };
use crate::stark::{ TraceState };
use crate::{ ACC_STATE_WIDTH, ACC_CYCLE_LENGTH };

// CONSTANTS
// ================================================================================================
const OP_CODE_CONSTRAINTS: usize = 6;
const NUM_CONSTRAINTS: usize = OP_CODE_CONSTRAINTS + ACC_STATE_WIDTH;

const CONSTRAINT_DEGREES: [usize; NUM_CONSTRAINTS] = [
    2, 2, 2, 2, 2,  // op_bits are binary
    6,              // op_code decomposition constraint
    4, 6, 3, 3      // op_code hash accumulator constraints
];

// TYPES AND INTERFACES
// ================================================================================================
pub struct Decoder {
    op_accumulator: AccEvaluator,
}

// DECODER CONSTRAINT EVALUATOR IMPLEMENTATION
// ================================================================================================
impl Decoder
{
    pub fn new(trace_length: usize, extension_factor: usize) -> Decoder {
        return Decoder {
            op_accumulator : AccEvaluator::new(trace_length, extension_factor)
        };
    }

    pub fn constraint_count(&self) -> usize {
        return NUM_CONSTRAINTS;
    }

    pub fn constraint_degrees(&self) -> &[usize] {
        return &CONSTRAINT_DEGREES;
    }

    // EVALUATOR FUNCTIONS
    // --------------------------------------------------------------------------------------------

    /// Evaluates decoder transition constraints at the specified step of the evaluation domain and
    /// saves the evaluations into `result`.
    pub fn evaluate(&self, current: &TraceState, next: &TraceState, step: usize, result: &mut [u128]) {

        // evaluate constraints for decoding op codes
        self.decode_opcode(current, next, result);

        // evaluate constraints for program accumulator
        self.op_accumulator.evaluate(
            current.get_op_acc(),
            next.get_op_acc(),
            current.get_op_code(), 
            step,
            &mut result[OP_CODE_CONSTRAINTS..]);
    }

    /// Evaluates decoder transition constraints at the specified x coordinate and saves the
    /// evaluations into `result`. Unlike the function above, this function can evaluate constraints
    /// at any out-of-domain point, but it is much slower than the previous function.
    pub fn evaluate_at(&self, current: &TraceState, next: &TraceState, x: u128, result: &mut [u128]) {

        // evaluate constraints for decoding op codes
        self.decode_opcode(current, next, result);

        // evaluate constraints for program accumulator
        self.op_accumulator.evaluate_at(
            current.get_op_acc(),
            next.get_op_acc(),
            current.get_op_code(),
            x,
            &mut result[OP_CODE_CONSTRAINTS..]);
    }

    // EVALUATION HELPERS
    // --------------------------------------------------------------------------------------------
    fn decode_opcode(&self, current: &TraceState, next: &TraceState, result: &mut [u128]) {
        
        // 5 constraints, degree 2: op_bits must be binary
        let op_bits = current.get_op_bits();
        for i in 0..5 {
            result[i] = is_binary(op_bits[i]);
        }

        // 1 constraint, degree 6: if current operation is a PUSH, next op_bits must be all
        // zeros (NOOP), otherwise next op_bits must be a binary decomposition of next op_code
        let is_push = current.get_op_flags()[opcodes::PUSH as usize];
        let op_bits_value = combine_bits(next.get_op_bits());
        let op_code = u128::mul(next.get_op_code(), binary_not(is_push));
        result[5] = u128::sub(op_code, op_bits_value);
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn is_binary(v: u128) -> u128 {
    return u128::sub(u128::mul(v, v), v);
}

fn binary_not(v: u128) -> u128 {
    return u128::sub(u128::ONE, v);
}

fn combine_bits(op_bits: &[u128]) -> u128 {
    let mut value = op_bits[0];
    let mut power_of_two = 1;
    for i in 1..op_bits.len() {
        power_of_two = power_of_two << 1;
        value = u128::add(value, u128::mul(op_bits[i], power_of_two));
    }
    return value;
}

// ACC EVALUATOR
// ================================================================================================

struct AccEvaluator {
    trace_length    : usize,
    cycle_length    : usize,
    ark_values      : Vec<[u128; 2 * ACC_STATE_WIDTH]>,
    ark_polys       : Vec<Vec<u128>>,
}

impl AccEvaluator {
    /// Creates a new AccEvaluator based on the provided `trace_length` and `extension_factor`.
    pub fn new(trace_length: usize, extension_factor: usize) -> AccEvaluator {
        // extend rounds constants by the specified extension factor
        let (ark_polys, ark_evaluations) = accumulator::get_extended_constants(extension_factor);

        // transpose round constant evaluations so that constants for each round
        // are stored in a single row
        let cycle_length = ACC_CYCLE_LENGTH * extension_factor;
        let mut ark_values = Vec::with_capacity(cycle_length);
        for i in 0..cycle_length {
            ark_values.push([u128::ZERO; 2 * ACC_STATE_WIDTH]);
            for j in 0..(2 * ACC_STATE_WIDTH) {
                ark_values[i][j] = ark_evaluations[j][i];
            }
        }

        return AccEvaluator { trace_length, cycle_length, ark_values, ark_polys };
    }

    /// Evaluates constraints at the specified step and saves the resulting values into `result`.
    pub fn evaluate(&self, current: &[u128], next: &[u128], op_code: u128, step: usize, result: &mut [u128]) {
        // determine round constants for the current step
        let ark = &self.ark_values[step % self.cycle_length];
        // evaluate constraints for op code accumulator
        self.acc_opcode(current, next, ark, op_code, result);
    }

    /// Evaluates constraints at the specified x coordinate and saves the resulting values into
    /// `result`. Unlike the function above, this function can evaluate constraints for any
    /// out-of-domain coordinate, but is significantly slower.
    pub fn evaluate_at(&self, current: &[u128], next: &[u128], op_code: u128, x: u128, result: &mut [u128]) {

        // determine round constants at the specified x coordinate
        let num_cycles = u128::from_usize(self.trace_length / ACC_CYCLE_LENGTH);
        let x = u128::exp(x, num_cycles);
        let mut ark = [u128::ZERO; 2 * ACC_STATE_WIDTH];
        for i in 0..ark.len() {
            ark[i] = polynom::eval(&self.ark_polys[i], x);
        }

        // evaluate constraints for op code accumulator
        self.acc_opcode(current, next, &ark, op_code, result);
    }

    /// Uses a modified version of Rescue hash function round to accumulate op_code values.
    /// The state consists of 4 elements and the op_code is injected into the state between
    /// the first and the second half of the round.
    fn acc_opcode(&self, current: &[u128], next: &[u128], ark: &[u128], op_code: u128, result: &mut [u128]) {

        let mut state_part1 = [u128::ZERO; ACC_STATE_WIDTH];
        state_part1.copy_from_slice(current);
        let mut state_part2 = [u128::ZERO; ACC_STATE_WIDTH];
        state_part2.copy_from_slice(next);

        // first half of Rescue round
        for i in 0..ACC_STATE_WIDTH {
            state_part1[i] = u128::add(state_part1[i], ark[i]);
        }
        accumulator::apply_sbox(&mut state_part1);
        accumulator::apply_mds(&mut state_part1);

        // op_code injection
        state_part1[0] = u128::add(state_part1[0], u128::mul(state_part1[2], op_code));
        state_part1[1] = u128::mul(state_part1[1], u128::add(state_part1[3], op_code));
        
        // second half of Rescue round
        accumulator::apply_inv_mds(&mut state_part2);
        accumulator::apply_sbox(&mut state_part2);
        for i in 0..ACC_STATE_WIDTH {
            state_part2[i] = u128::sub(state_part2[i], ark[ACC_STATE_WIDTH + i]);
        }

        for i in 0..ACC_STATE_WIDTH {
            result[i] = u128::sub(state_part2[i], state_part1[i]);
        }
    }
}