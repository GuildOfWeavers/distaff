use crate::math::{ FiniteField, polynom };
use crate::processor::{ opcodes };
use crate::utils::{ Accumulator };
use crate::stark::{ TraceState };
use crate::stark::{ ACC_STATE_WIDTH, ACC_CYCLE_LENGTH };

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
pub struct Decoder<T: FiniteField> {
    op_accumulator: AccEvaluator<T>,
}

// DECODER CONSTRAINT EVALUATOR IMPLEMENTATION
// ================================================================================================
impl <T> Decoder <T>
    where T: FiniteField + Accumulator
{
    pub fn new(trace_length: usize, extension_factor: usize) -> Decoder<T> {
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
    pub fn evaluate(&self, current: &TraceState<T>, next: &TraceState<T>, step: usize, result: &mut [T]) {

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
    pub fn evaluate_at(&self, current: &TraceState<T>, next: &TraceState<T>, x: T, result: &mut [T]) {

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
    fn decode_opcode(&self, current: &TraceState<T>, next: &TraceState<T>, result: &mut [T]) {
        
        // 5 constraints, degree 2: op_bits must be binary
        let op_bits = current.get_op_bits();
        for i in 0..5 {
            result[i] = is_binary(op_bits[i]);
        }

        // 1 constraint, degree 6: if current operation is a PUSH, next op_bits must be all
        // zeros (NOOP), otherwise next op_bits must be a binary decomposition of next op_code
        let is_push = current.get_op_flags()[opcodes::PUSH as usize];
        let op_bits_value = combine_bits(next.get_op_bits());
        let op_code = T::mul(next.get_op_code(), binary_not(is_push));
        result[5] = T::sub(op_code, op_bits_value);
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn is_binary<T: FiniteField>(v: T) -> T {
    return T::sub(T::mul(v, v), v);
}

fn binary_not<T: FiniteField>(v: T) -> T {
    return T::sub(T::ONE, v);
}

fn combine_bits<T: FiniteField>(op_bits: &[T]) -> T {
    let mut value = op_bits[0];
    let mut power_of_two = 1;
    for i in 1..op_bits.len() {
        power_of_two = power_of_two << 1;
        value = T::add(value, T::mul(op_bits[i], T::from(power_of_two)));
    }
    return value;
}

// ACC EVALUATOR
// ================================================================================================

struct AccEvaluator<T: FiniteField> {
    trace_length    : usize,
    cycle_length    : usize,
    ark_values      : Vec<[T; 2 * ACC_STATE_WIDTH]>,
    ark_polys       : Vec<Vec<T>>,
}

impl<T> AccEvaluator <T>
    where T: FiniteField + Accumulator
{
    /// Creates a new AccEvaluator based on the provided `trace_length` and `extension_factor`.
    pub fn new(trace_length: usize, extension_factor: usize) -> AccEvaluator<T> {
        // extend rounds constants by the specified extension factor
        let (ark_polys, ark_evaluations) = T::get_extended_constants(extension_factor);

        // transpose round constant evaluations so that constants for each round
        // are stored in a single row
        let cycle_length = ACC_CYCLE_LENGTH * extension_factor;
        let mut ark_values = Vec::with_capacity(cycle_length);
        for i in 0..cycle_length {
            ark_values.push([T::ZERO; 2 * ACC_STATE_WIDTH]);
            for j in 0..(2 * ACC_STATE_WIDTH) {
                ark_values[i][j] = ark_evaluations[j][i];
            }
        }

        return AccEvaluator { trace_length, cycle_length, ark_values, ark_polys };
    }

    /// Evaluates constraints at the specified step and saves the resulting values into `result`.
    pub fn evaluate(&self, current: &[T], next: &[T], op_code: T, step: usize, result: &mut [T]) {
        // determine round constants for the current step
        let ark = &self.ark_values[step % self.cycle_length];
        // evaluate constraints for op code accumulator
        self.acc_opcode(current, next, ark, op_code, result);
    }

    /// Evaluates constraints at the specified x coordinate and saves the resulting values into
    /// `result`. Unlike the function above, this function can evaluate constraints for any
    /// out-of-domain coordinate, but is significantly slower.
    pub fn evaluate_at(&self, current: &[T], next: &[T], op_code: T, x: T, result: &mut [T]) {

        // determine round constants at the specified x coordinate
        let num_cycles = T::from_usize(self.trace_length / ACC_CYCLE_LENGTH);
        let x = T::exp(x, num_cycles);
        let mut ark = [T::ZERO; 2 * ACC_STATE_WIDTH];
        for i in 0..ark.len() {
            ark[i] = polynom::eval(&self.ark_polys[i], x);
        }

        // evaluate constraints for op code accumulator
        self.acc_opcode(current, next, &ark, op_code, result);
    }

    /// Uses a modified version of Rescue hash function round to accumulate op_code values.
    /// The state consists of 4 elements and the op_code is injected into the state between
    /// the first and the second half of the round.
    fn acc_opcode(&self, current: &[T], next: &[T], ark: &[T], op_code: T, result: &mut [T]) {

        let mut state_part1 = [T::ZERO; ACC_STATE_WIDTH];
        state_part1.copy_from_slice(current);
        let mut state_part2 = [T::ZERO; ACC_STATE_WIDTH];
        state_part2.copy_from_slice(next);

        // first half of Rescue round
        for i in 0..ACC_STATE_WIDTH {
            state_part1[i] = T::add(state_part1[i], ark[i]);
        }
        T::apply_sbox(&mut state_part1);
        T::apply_mds(&mut state_part1);

        // op_code injection
        state_part1[0] = T::add(state_part1[0], T::mul(state_part1[2], op_code));
        state_part1[1] = T::mul(state_part1[1], T::add(state_part1[3], op_code));
        
        // second half of Rescue round
        T::apply_inv_mds(&mut state_part2);
        T::apply_sbox(&mut state_part2);
        for i in 0..ACC_STATE_WIDTH {
            state_part2[i] = T::sub(state_part2[i], ark[ACC_STATE_WIDTH + i]);
        }

        for i in 0..ACC_STATE_WIDTH {
            result[i] = T::sub(state_part2[i], state_part1[i]);
        }
    }
}