// TYPES AND INTERFACES
// ================================================================================================
#[derive(Debug)]
pub struct TraceState {
    pub op_code     : u64,
    pub op_bits     : [u64; 8],
    pub copy_flag   : u64,
    pub stack       : [u64; 32],
}

// TRACE STATE IMPLEMENTATION
// ================================================================================================
impl TraceState {

    pub fn new() -> TraceState {
        return TraceState {
            op_code     : 0,
            op_bits     : [0; 8],
            copy_flag   : 0,
            stack       : [0; 32]
        };
    }
    
}