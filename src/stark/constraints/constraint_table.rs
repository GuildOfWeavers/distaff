use crate::math::{ field };
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

        let mut trace = trace.clone(COMPOSITION_FACTOR);
        trace.extend();

        let mut decoder_constraints = Vec::new();
        for _ in 0..9 {
            decoder_constraints.push(utils::uninit_vector(trace.len()));
        }

        let mut stack_constraints = Vec::new();
        for _ in 0..trace.max_stack_depth() {
            stack_constraints.push(utils::uninit_vector(trace.len()));
        }        

        let mut current = TraceState::new();
        let mut next = TraceState::new();

        for i in 0..trace.len() {
            trace.fill_state(&mut current, i);
            trace.fill_state(&mut next, (i + COMPOSITION_FACTOR) % trace.len()); // TODO

            let op_flags = get_op_flags(&current.op_bits);
            decoder::evaluate(&current, &next, &op_flags, &mut decoder_constraints, i);
            stack::evaluate(&current, &next, &op_flags, &mut stack_constraints, i);
        }

        return ConstraintTable {
            decoder : decoder_constraints,
            stack   : stack_constraints,
        };
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn get_op_flags(op_bits: &[u64; 8]) -> [u64; 32] {
    let mut op_flags = [1u64; 32];

    // expand only the first 5 bits for now
    for i in 0..5 {
        
        let segment_length = usize::pow(2, (i + 1) as u32);

        let inv_bit = field::sub(field::ONE, op_bits[i]);
        for j in 0..(segment_length / 2) {
            op_flags[j] = field::mul(op_flags[j], inv_bit);
        }

        for j in (segment_length / 2)..segment_length {
            op_flags[j] = field::mul(op_flags[j], op_bits[i]);
        }

        let segment_slice = unsafe { &*(&op_flags[0..segment_length] as *const [u64]) };
        for j in (segment_length..32).step_by(segment_length) {
            op_flags[j..(j + segment_length)].copy_from_slice(segment_slice);
        }
    }

    return op_flags;
}