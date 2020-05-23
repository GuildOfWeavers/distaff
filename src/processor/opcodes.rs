pub const NOOP: u8     = 0b000_00_000;
//pub const DUP_B4: u8   = 0b000_00_001;
//pub const DUP_B8: u8   = 0b000_00_010;
//pub const PULL_B4: u8  = 0b000_00_011;
//pub const PULL_B8: u8  = 0b000_00_100;
//pub const HASH2: u8    = 0b000_00_101;

//pub const VERIFY: u8   = 0b000_01_000;
//pub const NOT: u8      = 0b000_01_001;
//pub const NEG: u8      = 0b000_01_010;
//pub const INV: u8      = 0b000_01_011;
pub const PULL1: u8    = 0b000_01_100;
pub const PULL2: u8    = 0b000_01_101;
//pub const PULL3: u8    = 0b000_01_110;
//pub const HASH: u8     = 0b000_01_111;

pub const PUSH: u8     = 0b000_10_000;
pub const DUP0: u8     = 0b000_10_001;
pub const DUP1: u8     = 0b000_10_010;
//pub const DUP2: u8     = 0b000_10_011;
//pub const DUP3: u8     = 0b000_10_100;
//pub const READ: u8     = 0b000_10_101;

pub const DROP : u8    = 0b000_11_000;
pub const ADD : u8     = 0b000_11_001;
pub const SUB : u8     = 0b000_11_010;
pub const MUL : u8     = 0b000_11_011;
//pub const EQ : u8     = 0b000_11_100;
//pub const LT : u8     = 0b000_11_101;
//pub const GT : u8     = 0b000_11_110;

/// 128-bit versions of opcodes
pub mod f128 {
    pub const NOOP  : u128 = super::NOOP as u128;
    pub const PULL1 : u128 = super::PULL1 as u128;
    pub const PULL2 : u128 = super::PULL2 as u128;
    pub const PUSH  : u128 = super::PUSH as u128;
    pub const DUP0  : u128 = super::DUP0 as u128;
    pub const DUP1  : u128 = super::DUP1 as u128;
    pub const DROP  : u128 = super::DROP as u128;
    pub const ADD   : u128 = super::ADD as u128;
    pub const SUB   : u128 = super::SUB as u128;
    pub const MUL   : u128 = super::MUL as u128;
}