mod trace_state;
mod trace_table;
mod decoder;
mod stack;

pub use trace_state::TraceState;
pub use trace_table::TraceTable;

pub const NUM_OP_BITS       : usize = 5;
pub const NUM_LD_OPS        : usize = 32;

pub const MAX_INPUTS        : usize = 8;
pub const MAX_OUTPUTS       : usize = 8;
pub const MIN_STACK_DEPTH   : usize = 8;
pub const MAX_STACK_DEPTH   : usize = 32;