use crate::trace::{ TraceTable, TraceState };
use crate::constraints::{ decoder, stack };
use crate::utils;

// CONSTANTS
// ================================================================================================
const COMPOSITION_FACTOR: usize = 8;

// TYPES AND INTERFACES
// ================================================================================================
pub struct ConstraintTable {
    pub decoder : Vec<Vec<u64>>,
    pub stack   : Vec<Vec<u64>>,
}

// CONSTRAINT TABLE IMPLEMENTATION
// ================================================================================================
impl ConstraintTable {

    pub fn new(trace: &TraceTable) -> ConstraintTable {
        assert!(!trace.is_extended(), "cannot evaluate extended trace table");
        assert!(trace.is_interpolated(), "cannot evaluate un-interpolated trace table");

        // clone the trace and extended it by the COMPOSITION_FACTOR
        let mut trace = trace.clone(COMPOSITION_FACTOR);
        trace.extend();

        // create vectors to hold constraint evaluations
        let mut decoder_constraints = Vec::new();
        for _ in 0..decoder::CONSTRAINT_DEGREES.len() {
            decoder_constraints.push(utils::uninit_vector(trace.len()));
        }

        let mut stack_constraints = Vec::new();
        for _ in 0..trace.max_stack_depth() {
            stack_constraints.push(utils::uninit_vector(trace.len()));
        }        

        // evaluate the constraints
        let mut current = TraceState::new(trace.max_stack_depth());
        let mut next = TraceState::new(trace.max_stack_depth());
        for i in 0..trace.len() {
            trace.fill_state(&mut current, i);
            trace.fill_state(&mut next, (i + COMPOSITION_FACTOR) % trace.len()); // TODO

            let op_flags = current.get_op_flags();
            decoder::evaluate(&current, &next, &op_flags, &mut decoder_constraints, i);
            stack::evaluate(&current, &next, &op_flags, &mut stack_constraints, i);
        }

        return ConstraintTable {
            decoder : decoder_constraints,
            stack   : stack_constraints,
        };
    }

    pub fn constraint_count(&self) -> usize {
        return self.decoder.len() + self.stack.len();
    }

    pub fn get_composition_poly() {
        // TODO: implement
    }
}
