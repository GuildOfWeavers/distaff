use crate::math::{ F64, FiniteField };
use crate::processor::{ opcodes };
use crate::stark::{ TraceState };
use crate::stark::utils::{ Accumulator, AccumulatorBuilder };

// CONSTANTS
// ================================================================================================
const NUM_CONSTRAINTS: usize = F64::ACC_STATE_WIDTH + 9;

/// Degree of operation decoder constraints.
const CONSTRAINT_DEGREES: [usize; NUM_CONSTRAINTS] = [
    2, 2, 2, 2, 2,      // op_bits are binary
    2,                  // push_flag is binary
    5,                  // push_flag is set after a PUSH operation
    2,                  // push_flag gets reset on the next step
    2,                  // when push_flag = 0, op_bits are a binary decomposition of op_code
    6, 6, 6, 6, 6, 6,   // op_code hash accumulator constraints
    6, 6, 6, 6, 6, 6
];

// TYPES AND INTERFACES
// ================================================================================================
pub struct Decoder<T> {
    accumulator : Accumulator<T>,
    trace_length: usize,
}

// DECODER CONSTRAINT EVALUATOR IMPLEMENTATION
// ================================================================================================
impl <T> Decoder <T>
    where T: FiniteField + AccumulatorBuilder<T>
{
    pub fn new(trace_length: usize, extension_factor: usize) -> Decoder<T> {
        let accumulator = T::get_accumulator(extension_factor);
        return Decoder { accumulator, trace_length };
    }

    pub fn constraint_count(&self) -> usize {
        return NUM_CONSTRAINTS;
    }

    pub fn constraint_degrees(&self) -> &[usize] {
        return &CONSTRAINT_DEGREES;
    }

    // EVALUATOR FUNCTION
    // --------------------------------------------------------------------------------------------
    pub fn evaluate(&self, current: &TraceState<T>, next: &TraceState<T>, step: usize, result: &mut [T]) {

        // 9 constraints to decode op_code
        self.decode_opcode(current, next, result);

        // 12 constraints to hash op_code
        self.hash_opcode(current, next, self.accumulator.get_constants_at(step), &mut result[9..]);
    }

    pub fn evaluate_at(&self, current: &TraceState<T>, next: &TraceState<T>, x: T, result: &mut [T]) {
        // 9 constraints to decode op_code
        self.decode_opcode(current, next, result);

        // 12 constraints to hash op_code
        let num_cycles = T::from_usize(self.trace_length / T::ACC_NUM_ROUNDS);
        let x = T::exp(x, num_cycles);
        self.hash_opcode(current, next, &self.accumulator.evaluate_constants_at(x), &mut result[9..]);
    }

    // EVALUATION HELPERS
    // --------------------------------------------------------------------------------------------
    fn decode_opcode(&self, current: &TraceState<T>, next: &TraceState<T>, result: &mut [T]) {
        // TODO: degree of expanded op_bits is assumed to be 5, but in reality can be less than 5
        // if opcodes used in the program don't touch some op_bits. Thus, all degrees that assume
        // op_flag values to have degree 5, may be off.

        // 5 constraints, degree 2: op_bits must be binary
        let op_bits = current.get_op_bits();
        for i in 0..5 {
            result[i] = is_binary(op_bits[i]);
        }

        // 1 constraint, degree 2: push_flag must be binary
        result[5] = is_binary(current.get_push_flag());

        // 1 constraint, degree 5: push_flag must be set to 1 after a PUSH operation
        let op_flags = current.get_op_flags();
        result[6] = T::sub(op_flags[opcodes::PUSH as usize], next.get_push_flag());

        // 1 constraint, degree 2: push_flag cannot be 1 for two consecutive operations
        result[7] = T::mul(current.get_push_flag(), next.get_push_flag());

        // 1 constraint, degree 2: when push_flag = 0, op_bits must be a binary decomposition
        // of op_code, otherwise all op_bits must be 0 (NOOP)
        let op_bits_value = current.get_op_bits_value();
        let op_code = T::mul(current.get_op_code(), binary_not(current.get_push_flag()));
        result[8] = T::sub(op_code, op_bits_value);
    }

    fn hash_opcode(&self, current: &TraceState<T>, next: &TraceState<T>, ark: &[T], result: &mut [T]) {
        let op_code = current.get_op_code();

        let mut current_acc = vec![T::ZERO; T::ACC_STATE_WIDTH];
        current_acc.copy_from_slice(current.get_op_acc());
        let mut next_acc = vec![T::ZERO; T::ACC_STATE_WIDTH];
        next_acc.copy_from_slice(next.get_op_acc());

        current_acc[0] = T::add(current_acc[0], op_code);
        current_acc[1] = T::mul(current_acc[1], op_code);
        for i in 0..T::ACC_STATE_WIDTH {
            current_acc[i] = T::add(current_acc[i], ark[i]);
        }
        self.accumulator.apply_sbox(&mut current_acc);
        self.accumulator.apply_mds(&mut current_acc);
    
        self.accumulator.apply_inv_mds(&mut next_acc);
        self.accumulator.apply_sbox(&mut next_acc);
        for i in 0..T::ACC_STATE_WIDTH {
            next_acc[i] = T::sub(next_acc[i], ark[T::ACC_STATE_WIDTH + i]);
        }

        for i in 0..T::ACC_STATE_WIDTH {
            result[i] = T::sub(next_acc[i], current_acc[i]);
        }
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn is_binary<T>(v: T) -> T
    where T: FiniteField
{
    return T::sub(T::mul(v, v), v);
}

fn binary_not<T>(v: T) -> T
    where T: FiniteField
{
    return T::sub(T::ONE, v);
}