use crate::stark::{ TraceState };
use crate::utils::{ uninit_vector };
use super::{ decoder, stack, hash_acc };

// CONSTANTS
// ================================================================================================
pub const MAX_CONSTRAINT_DEGREE: usize = 8;

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

        let trace_length = trace_length * MAX_CONSTRAINT_DEGREE;
        return ConstraintTable {
            decoder : create_vectors(decoder::CONSTRAINT_DEGREES.len(), trace_length),
            op_acc  : create_vectors(hash_acc::CONSTRAINT_DEGREES.len(), trace_length),
            stack   : create_vectors(max_stack_depth, trace_length),
        };
    }

    pub fn evaluate(&mut self, current: &TraceState, next: &TraceState, step: usize) {
        let should_be_zero = (step % MAX_CONSTRAINT_DEGREE == 0)
            && (step < self.len() - MAX_CONSTRAINT_DEGREE);

        let op_dec = decoder::evaluate(&current, &next);
        copy_constraints(&op_dec, &mut self.decoder, step, should_be_zero);
        
        let op_acc = hash_acc::evaluate(&current, &next, step);
        copy_constraints(&op_acc, &mut self.op_acc, step, should_be_zero);

        let stack = stack::evaluate(&current, &next, self.stack.len());
        copy_constraints(&stack, &mut self.stack, step, should_be_zero);
    }

    pub fn constraint_count(&self) -> usize {
        return self.decoder.len() + self.op_acc.len() + self.stack.len();
    }

    pub fn len(&self) -> usize {
        return self.decoder[0].len();
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

fn copy_constraints(source: &[u64], target: &mut Vec<Vec<u64>>, step: usize, should_be_zero: bool) {
    if should_be_zero {
        for i in 0..source.len() {
            assert!(source[i] == 0, "constraint at step {} didn't evaluate to 0", step / MAX_CONSTRAINT_DEGREE);
            target[i][step] = source[i];
        }
    }
    else {
        for i in 0..source.len() {
            target[i][step] = source[i];
        }
    }
}