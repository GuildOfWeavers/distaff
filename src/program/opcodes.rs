pub const NOOP: u64     = 0b000_00_000;
//pub const DUP_B4: u64   = 0b000_00_001;
//pub const DUP_B8: u64   = 0b000_00_010;
//pub const PULL_B4: u64  = 0b000_00_011;
//pub const PULL_B8: u64  = 0b000_00_100;
//pub const HASH2: u64    = 0b000_00_101;
//pub const MK_HASH: u64  = 0b000_00_111;

//pub const VERIFY: u64   = 0b000_01_000;
//pub const NOT: u64      = 0b000_01_001;
//pub const NEG: u64      = 0b000_01_010;
//pub const INV: u64      = 0b000_01_011;
pub const PULL1: u64    = 0b000_01_100;
pub const PULL2: u64    = 0b000_01_101;
//pub const PULL3: u64    = 0b000_01_110;
//pub const HASH: u64     = 0b000_01_111;

pub const PUSH: u64     = 0b000_10_000;
pub const DUP0: u64     = 0b000_10_001;
pub const DUP1: u64     = 0b000_10_010;
//pub const DUP2: u64     = 0b000_10_011;
//pub const DUP3: u64     = 0b000_10_100;
//pub const READ: u64     = 0b000_10_101;

pub const DROP : u64    = 0b000_11_000;
pub const ADD : u64     = 0b000_11_001;
pub const SUB : u64     = 0b000_11_010;
pub const MUL : u64     = 0b000_11_011;
//pub const EQ : u64     = 0b000_11_100;
//pub const LT : u64     = 0b000_11_101;
//pub const GT : u64     = 0b000_11_110;