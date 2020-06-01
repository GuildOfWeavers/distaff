pub const NOOP: u8     = 0b000_00_000;
//pub const VERIFY: u8 = 0b000_00_001;
//pub const ???: u8    = 0b000_00_010;
pub const INV: u8      = 0b000_00_011;
pub const NEG: u8      = 0b000_00_100;
//pub const NOT: u8    = 0b000_00_101;  // same as PUSH 1 SWAP SUB
//pub const ???: u8    = 0b000_00_110;
//pub const ???: u8    = 0b000_00_111;

pub const PUSH: u8     = 0b000_01_000;
pub const READ: u8     = 0b000_01_001;
pub const READ2: u8    = 0b000_01_010;
//pub const PAD2: u8   = 0b000_01_011;  // same as PUSH 0 DUP
pub const DUP: u8      = 0b000_01_100;  // same as DUP2 SWAP DROP
pub const DUP2: u8     = 0b000_01_101;
pub const DUP4: u8     = 0b000_01_110;
//pub const ???: u8    = 0b000_01_111;

pub const DROP: u8     = 0b000_10_000;
//pub const DROP4: u8  = 0b000_10_001;
pub const ADD: u8      = 0b000_10_010;
pub const SUB: u8      = 0b000_10_011;  // same as NEG ADD
pub const MUL: u8      = 0b000_10_100;
//pub const EQ: u8     = 0b000_10_101;
pub const CHOOSE: u8   = 0b000_10_110;
pub const CHOOSE2: u8  = 0b000_10_111;

pub const HASH: u8     = 0b000_11_000;
//pub const ???: u8    = 0b000_11_001;
pub const SWAP: u8     = 0b000_11_010;
pub const SWAP2: u8    = 0b000_11_011;
pub const SWAP4: u8    = 0b000_11_100;
pub const ROLL4: u8    = 0b000_11_101;
pub const ROLL8: u8    = 0b000_11_110;
pub const BEGIN: u8    = 0b000_11_111;

/// 128-bit versions of opcodes
pub mod f128 {
    pub const BEGIN   : u128 = super::BEGIN as u128;

    pub const PUSH    : u128 = super::PUSH as u128;
    pub const READ    : u128 = super::READ as u128;
    pub const READ2   : u128 = super::READ2 as u128;

    // stack manipulation ops
    pub const NOOP    : u128 = super::NOOP as u128;
    pub const SWAP    : u128 = super::SWAP as u128;
    pub const SWAP2   : u128 = super::SWAP2 as u128;
    pub const SWAP4   : u128 = super::SWAP4 as u128;
    pub const ROLL4   : u128 = super::ROLL4 as u128;
    pub const ROLL8   : u128 = super::ROLL8 as u128;
    pub const DUP     : u128 = super::DUP as u128;
    pub const DUP2    : u128 = super::DUP2 as u128;
    pub const DUP4    : u128 = super::DUP4 as u128;
    pub const DROP    : u128 = super::DROP as u128;
    pub const CHOOSE  : u128 = super::CHOOSE as u128;
    pub const CHOOSE2 : u128 = super::CHOOSE2 as u128;

    // math and logic ops
    pub const ADD     : u128 = super::ADD as u128;
    pub const SUB     : u128 = super::SUB as u128;
    pub const MUL     : u128 = super::MUL as u128;
    pub const INV     : u128 = super::INV as u128;
    pub const NEG     : u128 = super::NEG as u128;

    // crypto ops
    pub const HASH    : u128 = super::HASH as u128;
}