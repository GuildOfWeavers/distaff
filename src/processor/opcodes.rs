pub const NOOP: u8     = 0b000_00_000;
pub const CMP: u8      = 0b000_00_001;
pub const BINACC: u8   = 0b000_00_010;
//pub const ??: u8     = 0b000_00_011;
pub const INV: u8      = 0b000_00_100;
pub const NEG: u8      = 0b000_00_101;
pub const NOT: u8      = 0b000_00_110;  // same as: PUSH 1 SWAP NEG ADD
//pub const ???: u8    = 0b000_00_111;

pub const PUSH: u8     = 0b000_01_000;
pub const READ: u8     = 0b000_01_001;  // same as: READ2 DROP
pub const READ2: u8    = 0b000_01_010;
pub const DUP: u8      = 0b000_01_011;  // same as: DUP2 SWAP DROP
pub const DUP2: u8     = 0b000_01_100;
pub const DUP4: u8     = 0b000_01_101;
pub const PAD2: u8     = 0b000_01_110;  // same as: PUSH 0 DUP
//pub const ???: u8    = 0b000_01_111;

pub const ASSERT: u8   = 0b000_10_000;
pub const DROP: u8     = 0b000_10_001;
pub const DROP4: u8    = 0b000_10_010;  // same as: DROP DROP DROP DROP
pub const ADD: u8      = 0b000_10_011;
pub const MUL: u8      = 0b000_10_100;
pub const EQ: u8       = 0b000_10_101;
pub const CHOOSE: u8   = 0b000_10_110;
pub const CHOOSE2: u8  = 0b000_10_111;

pub const HASHR: u8    = 0b000_11_000;
//pub const ???: u8    = 0b000_11_001;
pub const SWAP: u8     = 0b000_11_010;
pub const SWAP2: u8    = 0b000_11_011;  // same as: ROLL4 ROLL4
pub const SWAP4: u8    = 0b000_11_100;  // same as: ROLL8 ROLL8 ROLL8 ROLL8
pub const ROLL4: u8    = 0b000_11_101;
pub const ROLL8: u8    = 0b000_11_110;
pub const BEGIN: u8    = 0b000_11_111;

/// 128-bit versions of opcodes
pub mod f128 {
    pub const BEGIN   : u128 = super::BEGIN as u128;
    pub const NOOP    : u128 = super::NOOP as u128;
    pub const ASSERT  : u128 = super::ASSERT as u128;

    // input ops
    pub const PUSH    : u128 = super::PUSH as u128;
    pub const READ    : u128 = super::READ as u128;
    pub const READ2   : u128 = super::READ2 as u128;

    // stack manipulation ops
    pub const DUP     : u128 = super::DUP as u128;
    pub const DUP2    : u128 = super::DUP2 as u128;
    pub const DUP4    : u128 = super::DUP4 as u128;
    pub const PAD2    : u128 = super::PAD2 as u128;
    pub const DROP    : u128 = super::DROP as u128;
    pub const DROP4   : u128 = super::DROP4 as u128;
    pub const SWAP    : u128 = super::SWAP as u128;
    pub const SWAP2   : u128 = super::SWAP2 as u128;
    pub const SWAP4   : u128 = super::SWAP4 as u128;
    pub const ROLL4   : u128 = super::ROLL4 as u128;
    pub const ROLL8   : u128 = super::ROLL8 as u128;

    // conditional ops
    pub const CHOOSE  : u128 = super::CHOOSE as u128;
    pub const CHOOSE2 : u128 = super::CHOOSE2 as u128;

    // math and boolean ops
    pub const ADD     : u128 = super::ADD as u128;
    pub const MUL     : u128 = super::MUL as u128;
    pub const INV     : u128 = super::INV as u128;
    pub const NEG     : u128 = super::NEG as u128;
    pub const NOT     : u128 = super::NOT as u128;

    // comparison ops
    pub const EQ      : u128 = super::EQ as u128;
    pub const CMP     : u128 = super::CMP as u128;

    // crypto ops
    pub const HASHR   : u128 = super::HASHR as u128;
}