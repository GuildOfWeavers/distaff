use crate::trace::{ MAX_STACK_DEPTH };

// TYPES AND INTERFACES
// ================================================================================================
#[derive(Debug, PartialEq)]
pub struct TraceState {
    pub op_code     : u64,
    pub op_bits     : [u64; 8],
    pub stack       : [u64; MAX_STACK_DEPTH],
}

// TRACE STATE IMPLEMENTATION
// ================================================================================================
impl TraceState {

    pub fn new() -> TraceState {
        return TraceState {
            op_code     : 0,
            op_bits     : [0; 8],
            stack       : [0; MAX_STACK_DEPTH]
        };
    }
}