use std::ops::Range;
use super::{ ACC_STATE_WIDTH, ACC_STATE_RATE };

mod trace_state;
mod trace_table;
mod decoder;
mod stack;

pub use trace_state::TraceState;
pub use trace_table::TraceTable;

pub const MIN_TRACE_LENGTH  : usize = 16;
pub const MAX_REGISTER_COUNT: usize = 128;

// DECODER
// ================================================================================================
// Decoder trace consists of 10 registers and has the following layout:
//
//   op  ╒═════════ op_bits ═══════════╕╒══════ op_acc ════════╕
//    0      1    2     3     4     5     6     7     8     9
// ├─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┤
//

pub const NUM_OP_BITS       : usize = 5;
pub const DECODER_WIDTH     : usize = 1 + NUM_OP_BITS + ACC_STATE_WIDTH;

pub const OP_CODE_INDEX     : usize = 0;
pub const OP_BITS_RANGE     : Range<usize> = Range { start: 1, end: 6 };
pub const OP_ACC_RANGE      : Range<usize> = Range { start: 6, end: 6 + ACC_STATE_WIDTH };
pub const PROG_HASH_RANGE   : Range<usize> = Range { start: 6, end: 6 + ACC_STATE_RATE  };

// STACK
// ================================================================================================
// Stack trace consists of at between 3 and 32 registers and has the following layout:
//
// ╒══ aux ════╕╒═════════════ user registers ═════════════════╕
//    0      1    2    .................................    30
// ├─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┤

pub const MAX_INPUTS        : usize = 8;
pub const MAX_OUTPUTS       : usize = 8;
pub const MIN_STACK_DEPTH   : usize = 10;
pub const MAX_STACK_DEPTH   : usize = 32;
pub const AUX_WIDTH         : usize = 2;