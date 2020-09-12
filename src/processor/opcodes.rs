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

impl std::fmt::Display for FlowOps {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        return match self {

            FlowOps::Hacc   => write!(f, "hacc"),

            FlowOps::Begin  => write!(f, "begin"),
            FlowOps::Tend   => write!(f, "tend"),
            FlowOps::Fend   => write!(f, "fend"),

            FlowOps::Loop   => write!(f, "loop"),
            FlowOps::Wrap   => write!(f, "wrap"),
            FlowOps::Break  => write!(f, "break"),

            FlowOps::Void   => write!(f, "void"),
        };
    }
}

impl std::convert::TryFrom<u8> for FlowOps {

    type Error = String;

    fn try_from(value: u8) -> Result<Self, String> {
        return match value {

            0b00000_000 => Ok(FlowOps::Hacc),
            0b00000_001 => Ok(FlowOps::Begin),
            0b00000_010 => Ok(FlowOps::Tend),
            0b00000_011 => Ok(FlowOps::Fend),
            0b00000_100 => Ok(FlowOps::Loop),
            0b00000_101 => Ok(FlowOps::Wrap),
            0b00000_110 => Ok(FlowOps::Break),
            0b00000_111 => Ok(FlowOps::Void),
            
            _ => Err(format!("value {} is not a valid control flow opcode", value))
        };
    }
}

// USER OPERATIONS
// ================================================================================================
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UserOps {
    
    Noop        = 0b00_00_0000,         // no shift
    Begin       = 0b00_11_1111,         // no shift

    // degree 1 operations
    Assert      = 0b00_00_0001,         // left shift: 1
    AssertEq    = 0b00_00_0010,         // left shift: 2
    Drop        = 0b00_00_0011,         // left shift: 1
    Drop4       = 0b00_00_0100,         // left shift: 4
    Read        = 0b00_00_0101,         // right shift: 1
    Read2       = 0b00_00_0110,         // right shift: 2
    Dup         = 0b00_00_0111,         // right shift: 1
    Dup2        = 0b00_00_1000,         // right shift: 2
    Dup4        = 0b00_00_1001,         // right shift: 4
    Pad2        = 0b00_00_1010,         // right shift: 2
    Swap        = 0b00_00_1011,         // no shift
    Swap2       = 0b00_00_1100,         // no shift
    Swap4       = 0b00_00_1101,         // no shift
    Roll4       = 0b00_00_1110,         // no shift
    Roll8       = 0b00_00_1111,         // no shift

    // degree 2 operations
    Eq          = 0b00_01_0000,         // left shift: 2
    Choose      = 0b00_01_0001,         // left shift: 2
    Choose2     = 0b00_01_0010,         // left shift: 4
    CSwap2      = 0b00_01_0011,         // left shift: 2
    Add         = 0b00_01_0100,         // left shift: 1
    Mul         = 0b00_01_0101,         // left shift: 1
    And         = 0b00_01_0110,         // left shift: 1
    Or          = 0b00_01_0111,         // left shift: 1
    Inv         = 0b00_01_1000,         // no shift
    Neg         = 0b00_01_1001,         // no shift
    Not         = 0b00_01_1010,         // no shift
    BinAcc      = 0b00_01_1011,         // no shift
    MLoad       = 0b00_01_1100,
    MStore      = 0b00_01_1101,
    Future1     = 0b00_01_1110,
    //invalid   = 0b00_01_1111,

    // high-degree operations
    Push        = 0b00_10_0001,         // right shift: 1
    Cmp         = 0b00_10_0010,         // no shift
    RescR       = 0b00_10_0100,         // no shift
    MemRR       = 0b00_10_1000,         // no shift
}

impl std::fmt::Display for UserOps {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        return match self {

            UserOps::Begin      => write!(f, "begin"),
            UserOps::Noop       => write!(f, "noop"),

            UserOps::Assert     => write!(f, "assert"),
            UserOps::AssertEq   => write!(f, "asserteq"),
    
            UserOps::Push       => write!(f, "push"),
            UserOps::Read       => write!(f, "read"),
            UserOps::Read2      => write!(f, "read2"),

            UserOps::MLoad      => write!(f, "mload"),
            UserOps::MStore     => write!(f, "mstore"),
            UserOps::MemRR      => write!(f, "memrr"),
    
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
            UserOps::CSwap2     => write!(f, "cswap2"),
    
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
    
            UserOps::RescR      => write!(f, "rescr"),
            UserOps::Future1    => write!(f, "future1")
        };
    }
}

impl std::convert::TryFrom<u8> for UserOps {

    type Error = String;

    fn try_from(value: u8) -> Result<Self, String> {
        return match value {

            0b00_11_1111 => Ok(UserOps::Begin),
            0b00_00_0000 => Ok(UserOps::Noop),

            0b00_00_0001 => Ok(UserOps::Assert),
            0b00_00_0010 => Ok(UserOps::AssertEq),
            0b00_00_0011 => Ok(UserOps::Drop),
            0b00_00_0100 => Ok(UserOps::Drop4),
            0b00_00_0101 => Ok(UserOps::Read),
            0b00_00_0110 => Ok(UserOps::Read2),
            0b00_00_0111 => Ok(UserOps::Dup),
            0b00_00_1000 => Ok(UserOps::Dup2),
            0b00_00_1001 => Ok(UserOps::Dup4),
            0b00_00_1010 => Ok(UserOps::Pad2),
            0b00_00_1011 => Ok(UserOps::Swap),
            0b00_00_1100 => Ok(UserOps::Swap2),
            0b00_00_1101 => Ok(UserOps::Swap4),
            0b00_00_1110 => Ok(UserOps::Roll4),
            0b00_00_1111 => Ok(UserOps::Roll8),
            
            0b00_01_0000 => Ok(UserOps::Eq),
            0b00_01_0001 => Ok(UserOps::Choose),
            0b00_01_0010 => Ok(UserOps::Choose2),
            0b00_01_0011 => Ok(UserOps::CSwap2),
            0b00_01_0100 => Ok(UserOps::Add),
            0b00_01_0101 => Ok(UserOps::Mul),
            0b00_01_0110 => Ok(UserOps::And),
            0b00_01_0111 => Ok(UserOps::Or),
            0b00_01_1000 => Ok(UserOps::Inv),
            0b00_01_1001 => Ok(UserOps::Neg),
            0b00_01_1010 => Ok(UserOps::Not),
            0b00_01_1011 => Ok(UserOps::BinAcc),
            0b00_01_1100 => Ok(UserOps::MLoad),
            0b00_01_1101 => Ok(UserOps::MStore),
            0b00_01_1110 => Ok(UserOps::Future1),

            0b00_10_0001 => Ok(UserOps::Push),
            0b00_10_0010 => Ok(UserOps::Cmp),
            0b00_10_0100 => Ok(UserOps::RescR),
            0b00_10_1000 => Ok(UserOps::MemRR),

            _ => Err(format!("value {} is not a valid user opcode", value))
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
    PmpathStart(u32),
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
            OpHint::CmpStart(value)     => write!(f, ".{}", value),
            OpHint::PmpathStart(value)  => write!(f, ".{}", value),
            OpHint::PushValue(value)    => write!(f, "({})", value),
            OpHint::None             => Ok(()),
        };
    }
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {

    use std::convert::TryFrom;

    #[test]
    fn parse_user_ops() {

        for i in 0..64 {
            match super::UserOps::try_from(i) {
                Ok(opcode) => assert_eq!(i, opcode as u8),
                Err(_) => ()
            }
        }
    }

    #[test]
    fn parse_flow_ops() {

        for i in 0..8 {
            match super::FlowOps::try_from(i) {
                Ok(opcode) => assert_eq!(i, opcode as u8),
                Err(_) => ()
            }
        }
    }
}