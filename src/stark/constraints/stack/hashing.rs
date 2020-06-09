use std::cmp;
use crate::math::{ FiniteField, polynom };
use crate::stark::{ Hasher };
use crate::stark::{ HASH_STATE_WIDTH, HASH_CYCLE_LENGTH };

// TYPES AND INTERFACES
// ================================================================================================
pub struct HashEvaluator<T: FiniteField> {
    trace_length    : usize,
    cycle_length    : usize,
    ark_values      : Vec<[T; 2 * HASH_STATE_WIDTH]>,
    ark_polys       : Vec<Vec<T>>,
}

// HASH EVALUATOR IMPLEMENTATION
// ================================================================================================
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
        let step = step % self.cycle_length;

        // determine round constants for the current step
        let ark = &self.ark_values[step];

        // evaluate constraints for the hash function and for the rest of the stack
        self.eval_hash(current, next, ark, op_flag, result);
        self.eval_rest(current, next, op_flag, result);
    }

    /// Evaluates constraints at the specified x coordinate and adds the resulting values to `result`.
    /// Unlike the function above, this function can evaluate constraints for any out-of-domain 
    /// coordinate, but is significantly slower.
    pub fn evaluate_at(&self, current: &[T], next: &[T], x: T, op_flag: T, result: &mut [T]) {

        // determine mask and round constants at the specified x coordinate
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
    /// assumed to be in the first 6 registers of user stack (aux registers are not affected).
    fn eval_hash(&self, current: &[T], next: &[T], ark: &[T], op_flag: T, result: &mut [T]) {

        // TODO: use a constant for user stack offset
        let mut state_part1 = [T::ZERO; HASH_STATE_WIDTH];
        state_part1.copy_from_slice(&current[1..(1 + HASH_STATE_WIDTH)]);
        let mut state_part2 = [T::ZERO; HASH_STATE_WIDTH];
        state_part2.copy_from_slice(&next[1..(1 + HASH_STATE_WIDTH)]);

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

        let result = &mut result[1..]; // TODO: use constant
        for i in 0..cmp::min(result.len(), HASH_STATE_WIDTH) {
            let evaluation = T::sub(state_part2[i], state_part1[i]);
            result[i] = T::add(result[i], T::mul(evaluation, op_flag));
        }
    }

    /// Evaluates constraints for stack registers un-affected by hash transition.
    fn eval_rest(&self, current: &[T], next: &[T], op_flag: T, result: &mut [T]) {
        for i in (1 + HASH_STATE_WIDTH)..result.len() { // TODO: use constant
            let evaluation = T::sub(next[i], current[i]);
            result[i] = T::add(result[i], T::mul(evaluation, op_flag));
        }
    }
}