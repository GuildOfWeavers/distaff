#[derive(Copy, Clone, Debug)]
pub enum UserOps {
    Noop    = 0b111_11_111,
}

#[derive(Copy, Clone, Debug)]
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