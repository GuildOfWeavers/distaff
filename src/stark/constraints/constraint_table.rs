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

        // create vectors to hold constraint evaluations
        let mut decoder_constraints = Vec::new();
        for _ in 0..decoder::CONSTRAINT_DEGREES.len() {
            decoder_constraints.push(uninit_vector(trace_length));
        }

        let mut op_acc_constraints = Vec::new();
        for _ in 0..acc_hash::CONSTRAINT_DEGREES.len() {
            op_acc_constraints.push(uninit_vector(trace_length));
        }

        let mut stack_constraints = Vec::new();
        for _ in 0..max_stack_depth {
            stack_constraints.push(uninit_vector(trace_length));
        }

        return ConstraintTable {
            decoder : decoder_constraints,
            stack   : stack_constraints,
            op_acc  : op_acc_constraints,
        };
    }

    pub fn evaluate(&mut self, current: &TraceState, next: &TraceState, index: usize) {
        decoder::evaluate(&current, &next, &mut self.decoder, index);
        acc_hash::evaluate(&current, &next, &mut self.op_acc, index);
        stack::evaluate(&current, &next, &mut self.stack, index);
    }

    pub fn constraint_count(&self) -> usize {
        return self.decoder.len() + self.op_acc.len() + self.stack.len();
    }

    pub fn get_composition_poly() {
        // TODO: implement
    }
}