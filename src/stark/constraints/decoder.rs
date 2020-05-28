use crate::math::{ F128, FiniteField, polynom };
use crate::processor::{ opcodes };
use crate::stark::{ TraceState, utils::Accumulator };

// CONSTANTS
// ================================================================================================
const OP_CODE_CONSTRAINTS: usize = 6;
const NUM_CONSTRAINTS: usize = F128::STATE_WIDTH + OP_CODE_CONSTRAINTS; // TODO

/// Degree of operation decoder constraints.
const CONSTRAINT_DEGREES: [usize; NUM_CONSTRAINTS] = [
    2, 2, 2, 2, 2,  // op_bits are binary
    6,              // when current op is not a push, next op_bits are a binary decomposition of op_code
    6, 6, 6, 6      // op_code hash accumulator constraints
];

// TYPES AND INTERFACES
// ================================================================================================
pub struct Decoder<T> {
    rescue_ark  : Vec<Vec<T>>,
    rescue_polys: Vec<Vec<T>>,
    hash_cycle  : usize,
    trace_length: usize,
}

// DECODER CONSTRAINT EVALUATOR IMPLEMENTATION
// ================================================================================================
impl <T> Decoder <T>
    where T: FiniteField + Accumulator
{
    pub fn new(trace_length: usize, extension_factor: usize) -> Decoder<T> {
        let (rescue_polys, ark_evaluations) = T::get_extended_constants(extension_factor);

        let hash_cycle = T::NUM_ROUNDS * extension_factor;
        let mut rescue_ark = Vec::with_capacity(hash_cycle);
        for i in 0..(T::NUM_ROUNDS * extension_factor) {
            rescue_ark.push(vec![T::ZERO; 2 * T::STATE_WIDTH]);
            for j in 0..(2 * T::STATE_WIDTH) {
                rescue_ark[i][j] = ark_evaluations[j][i];
            }
        }

        return Decoder { rescue_ark, rescue_polys, hash_cycle, trace_length };
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

        // 6 constraints to decode op_code
        self.decode_opcode(current, next, result);

        // constraints to hash op_code
        self.hash_opcode(current, next, &self.rescue_ark[step % self.hash_cycle], &mut result[9..]);
    }

    pub fn evaluate_at(&self, current: &TraceState<T>, next: &TraceState<T>, x: T, result: &mut [T]) {
        // 96 constraints to decode op_code
        self.decode_opcode(current, next, result);

        // constraints to hash op_code
        let num_cycles = T::from_usize(self.trace_length / T::NUM_ROUNDS);
        let x = T::exp(x, num_cycles);

        let mut rescue_ark = vec![T::ZERO; 2 * T::STATE_WIDTH];
        for i in 0..rescue_ark.len() {
            rescue_ark[i] = polynom::eval(&self.rescue_polys[i], x);
        }

        self.hash_opcode(current, next, &rescue_ark, &mut result[9..]);
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

    fn hash_opcode(&self, current: &TraceState<T>, next: &TraceState<T>, ark: &[T], result: &mut [T]) {
        let op_code = current.get_op_code();

        let mut current_acc = vec![T::ZERO; T::STATE_WIDTH]; // TODO: convert to array
        current_acc.copy_from_slice(current.get_op_acc());
        let mut next_acc = vec![T::ZERO; T::STATE_WIDTH];    // TODO: convert to array
        next_acc.copy_from_slice(next.get_op_acc());

        current_acc[0] = T::add(current_acc[0], op_code);
        current_acc[1] = T::mul(current_acc[1], op_code);
        for i in 0..T::STATE_WIDTH {
            current_acc[i] = T::add(current_acc[i], ark[i]);
        }
        T::apply_sbox(&mut current_acc);
        T::apply_mds(&mut current_acc);
    
        T::apply_inv_mds(&mut next_acc);
        T::apply_sbox(&mut next_acc);
        for i in 0..T::STATE_WIDTH {
            next_acc[i] = T::sub(next_acc[i], ark[T::STATE_WIDTH + i]);
        }

        for i in 0..T::STATE_WIDTH {
            result[i] = T::sub(next_acc[i], current_acc[i]);
        }
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