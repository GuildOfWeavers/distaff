use crate::stark::{ TraceState };
use crate::utils::{ uninit_vector };
use super::{ decoder, stack, acc_hash };

// CONSTANTS
// ================================================================================================
const COMPOSITION_FACTOR: usize = 8;

// TYPES AND INTERFACES
// ================================================================================================
pub struct ConstraintTable {
    pub decoder : Vec<Vec<u64>>,
    pub op_acc  : Vec<Vec<u64>>,
    pub stack   : Vec<Vec<u64>>,
}

// CONSTRAINT TABLE IMPLEMENTATION
// ================================================================================================
impl ConstraintTable {

    pub fn new(trace_length: usize, max_stack_depth: usize) -> ConstraintTable {
        debug_assert!(trace_length.is_power_of_two(), "trace length must be a power of 2");

        let trace_length = trace_length * COMPOSITION_FACTOR;
        return ConstraintTable {
            decoder : create_vectors(decoder::CONSTRAINT_DEGREES.len(), trace_length),
            op_acc  : create_vectors(acc_hash::CONSTRAINT_DEGREES.len(), trace_length),
            stack   : create_vectors(max_stack_depth, trace_length),
        };
    }

    pub fn evaluate(&mut self, current: &TraceState, next: &TraceState, step: usize) {
        let op_dec = decoder::evaluate(&current, &next);
        for i in 0..op_dec.len() {
            self.decoder[i][step] = op_dec[i];
        }

        let op_acc = acc_hash::evaluate(&current, &next, step);
        for i in 0..op_acc.len() {
            self.op_acc[i][step] = op_acc[i];
        }

        let stack = stack::evaluate(&current, &next, self.stack.len());
        for i in 0..stack.len() {
            self.stack[i][step] = stack[i];
        }
    }

    pub fn constraint_count(&self) -> usize {
        return self.decoder.len() + self.op_acc.len() + self.stack.len();
    }

    pub fn get_composition_poly() {
        // TODO: implement
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn create_vectors(num_columns: usize, trace_length: usize) -> Vec<Vec<u64>> {
    let mut result = Vec::with_capacity(num_columns);
    for _ in 0..num_columns {
        result.push(uninit_vector(trace_length));
    }
    return result;
}