// FLOW CONTROL OPERATIONS
// ================================================================================================
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FlowOps {
    Hacc    = 0b000,
    Begin   = 0b001,
    Tend    = 0b010,
    Fend    = 0b011,
    Loop    = 0b100,
    Wrap    = 0b101,
    Break   = 0b110,
    Void    = 0b111,
}

// USER OPERATIONS
// ================================================================================================
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UserOps {
    
    Assert      = 0b000_00_001,
    AssertEq    = 0b000_00_010,
    Eq          = 0b000_00_011,
    Cmp         = 0b000_00_100,
    BinAcc      = 0b000_00_101,
    Choose      = 0b000_00_110,
    Choose2     = 0b000_00_111,

    Add         = 0b000_01_000,
    Mul         = 0b000_01_001,
    Inv         = 0b000_01_010,
    Neg         = 0b000_01_011,
    Not         = 0b000_01_100,
    And         = 0b000_01_101,
    Or          = 0b000_01_110,
    //???       = 0b000_01_111,

    Dup         = 0b000_10_000,
    Dup2        = 0b000_10_001,
    Dup4        = 0b000_10_010,
    Pad2        = 0b000_10_011,
    Drop        = 0b000_10_100,
    Drop4       = 0b000_10_101,
    Swap        = 0b000_10_110,
    Swap2       = 0b000_10_111,

    Swap4       = 0b000_11_000,
    Roll4       = 0b000_11_001,
    Roll8       = 0b000_11_010,
    Push        = 0b000_11_011,
    Read        = 0b000_11_100,
    Read2       = 0b000_11_101,
    RescR       = 0b000_11_110,
    
    Noop        = 0b111_11_111,
}

impl std::fmt::Display for UserOps {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        return match self {

            UserOps::Noop       => write!(f, "noop"),
            UserOps::Assert     => write!(f, "assert"),
            UserOps::AssertEq   => write!(f, "asserteq"),
    
            UserOps::Push       => write!(f, "push"),
            UserOps::Read       => write!(f, "read"),
            UserOps::Read2      => write!(f, "read2"),
    
            UserOps::Dup        => write!(f, "dup"),
            UserOps::Dup2       => write!(f, "dup2"),
            UserOps::Dup4       => write!(f, "dup4"),
            UserOps::Pad2       => write!(f, "pad2"),
    
            UserOps::Drop       => write!(f, "drop"),
            UserOps::Drop4      => write!(f, "drop4"),
    
            UserOps::Swap       => write!(f, "swap"),
            UserOps::Swap2      => write!(f, "swap2"),
            UserOps::Swap4      => write!(f, "swap4"),
    
            UserOps::Roll4      => write!(f, "roll4"),
            UserOps::Roll8      => write!(f, "roll8"),
    
            UserOps::Choose     => write!(f, "choose"),
            UserOps::Choose2    => write!(f, "choose2"),
    
            UserOps::Add        => write!(f, "add"),
            UserOps::Mul        => write!(f, "mul"),
            UserOps::Inv        => write!(f, "inv"),
            UserOps::Neg        => write!(f, "neg"),
            UserOps::Not        => write!(f, "not"),
            UserOps::And        => write!(f, "and"),
            UserOps::Or         => write!(f, "or"),
    
            UserOps::Eq         => write!(f, "eq"),
            UserOps::Cmp        => write!(f, "cmp"),
            UserOps::BinAcc     => write!(f, "binacc"),
    
            UserOps::RescR      => write!(f, "rescr")
        };
    }
}

// OPERATION HINTS
// ================================================================================================
#[derive(Copy, Clone, Debug)]
pub enum OpHint {
    EqStart,
    RcStart(u32),
    CmpStart(u32),
    PushValue(u128),
    None,
}

impl OpHint {
    pub fn value(&self) -> u128 {
        return match self {
            OpHint::PushValue(value) => *value,
            _ => 0,
        };
    }
}

impl std::fmt::Display for OpHint {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        return match self {
            OpHint::EqStart          => write!(f, "::eq"),
            OpHint::RcStart(value)   => write!(f, ".{}", value),
            OpHint::CmpStart(value)  => write!(f, ".{}", value),
            OpHint::PushValue(value) => write!(f, "({})", value),
            OpHint::None             => Ok(()),
        };
    }
}