pub const NOOP: u64 = 0b000_00_000;
pub const PUSH: u64 = 0b000_00_001;

pub const COPY: u64 = 0b000_01_000;
pub const DUP0: u64 = 0b000_01_001;
pub const DUP1: u64 = 0b000_01_010;

pub const POP : u64 = 0b000_10_000;
pub const ADD : u64 = 0b000_10_001;
pub const SUB : u64 = 0b000_10_010;
pub const MUL : u64 = 0b000_10_011;