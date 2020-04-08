use std::fmt;
use crate::trace::{ MAX_STACK_DEPTH };

// TYPES AND INTERFACES
// ================================================================================================
#[derive(Debug, PartialEq)]
pub struct TraceState {
    pub op_code     : u64,
    pub push_flag   : u64,
    pub op_bits     : [u64; 5],
    pub stack       : [u64; MAX_STACK_DEPTH],
}

// TRACE STATE IMPLEMENTATION
// ================================================================================================
impl TraceState {

    pub fn new() -> TraceState {
        return TraceState {
            op_code     : 0,
            push_flag   : 0,
            op_bits     : [0; 5],
            stack       : [0; MAX_STACK_DEPTH]
        };
    }
}

impl fmt::Display for TraceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]\t[{}]\t{:?}\t{:?}", self.op_code, self.push_flag, self.op_bits, self.stack)
    }
}