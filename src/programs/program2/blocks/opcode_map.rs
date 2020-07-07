use super::{ opcodes, ExecutionHint };

pub fn op_to_str(op_code: u8, op_hint: ExecutionHint) -> String {

    return match op_code {

        opcodes::NOOP       => String::from("noop"),
        opcodes::ASSERT     => String::from("assert"),
        opcodes::ASSERTEQ   => String::from("asserteq"),

        opcodes::PUSH       => {
            let op_value = match op_hint {
                ExecutionHint::PushValue(value) => value,
                _ => panic!("invalid value for PUSH operation"),
            };
            format!("push({})", op_value)
        },

        opcodes::READ       => String::from("read"),
        opcodes::READ2      => String::from("read2"),

        opcodes::DUP        => String::from("dup"),
        opcodes::DUP2       => String::from("dup2"),
        opcodes::DUP4       => String::from("dup4"),
        opcodes::PAD2       => String::from("pad2"),

        opcodes::DROP       => String::from("drop"),
        opcodes::DROP4      => String::from("drop4"),

        opcodes::SWAP       => String::from("swap"),
        opcodes::SWAP2      => String::from("swap2"),
        opcodes::SWAP4      => String::from("swap4"),

        opcodes::ROLL4      => String::from("roll4"),
        opcodes::ROLL8      => String::from("roll8"),

        opcodes::CHOOSE     => String::from("choose"),
        opcodes::CHOOSE2    => String::from("choose2"),

        opcodes::ADD        => String::from("add"),
        opcodes::MUL        => String::from("mul"),
        opcodes::INV        => String::from("inv"),
        opcodes::NEG        => String::from("neg"),
        opcodes::NOT        => String::from("not"),
        opcodes::AND        => String::from("and"),
        opcodes::OR         => String::from("or"),

        opcodes::EQ         => String::from("eq"),
        opcodes::CMP        => String::from("cmp"),
        opcodes::BINACC     => String::from("binacc"),

        opcodes::RESCR      => String::from("rescr"),

        _ => format!("unknown {}", op_code),
    };
}