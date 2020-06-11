use super::opcodes;

// OPERATION PARSERS
// ================================================================================================
pub fn parse_noop(program: &mut Vec<u128>, op: &[&str]) {
    let param = read_param(op);
    for _ in 0..param {
        program.push(opcodes::NOOP);
    }
}

pub fn parse_assert(program: &mut Vec<u128>, op: &[&str]) {
    let param = read_param(op);
    for _ in 0..param {
        program.push(opcodes::ASSERT);
    }
}

pub fn parse_push(program: &mut Vec<u128>, op: &[&str]) {
    let value = read_value(op);
    program.extend_from_slice(&[opcodes::PUSH, value]);
}

pub fn parse_read(program: &mut Vec<u128>, op: &[&str]) {
    if op.len() == 1 || op[1] == "a" {
        program.push(opcodes::READ);
    }
    else if op[1] == "ab" {
        program.push(opcodes::READ2);
    }
    else {
        // TODO: error
    }
}

pub fn parse_dup(program: &mut Vec<u128>, op: &[&str]) {
    let param = read_param(op);
    match param {
        1 => program.push(opcodes::DUP),
        2 => program.push(opcodes::DUP2),
        3 => program.extend_from_slice(&[opcodes::DUP4, opcodes::ROLL4, opcodes::DROP]),
        4 => program.push(opcodes::DUP4),
        _ => {} // TODO
    }
}

pub fn parse_pad(program: &mut Vec<u128>, op: &[&str]) {
    let param = read_param(op);
    match param {
        1 => program.extend_from_slice(&[opcodes::PAD2, opcodes::DROP]),
        2 => program.push(opcodes::PAD2),
        3 => program.extend_from_slice(&[opcodes::PAD2, opcodes::PAD2, opcodes::DROP]),
        4 => program.extend_from_slice(&[opcodes::PAD2, opcodes::PAD2]),
        5 => program.extend_from_slice(&[opcodes::PAD2, opcodes::PAD2, opcodes::PAD2, opcodes::DROP]),
        6 => program.extend_from_slice(&[opcodes::PAD2, opcodes::PAD2, opcodes::PAD2]),
        7 => program.extend_from_slice(&[opcodes::PAD2, opcodes::PAD2, opcodes::DUP4, opcodes::DROP]),
        8 => program.extend_from_slice(&[opcodes::PAD2, opcodes::PAD2, opcodes::DUP4]),
        _ => {} // TODO
    }
}

pub fn parse_drop(program: &mut Vec<u128>, op: &[&str]) {
    let param = read_param(op);
    match param {
        1 => program.push(opcodes::DROP),
        2 => program.extend_from_slice(&[opcodes::DROP, opcodes::DROP]),
        3 => program.extend_from_slice(&[opcodes::DROP, opcodes::DROP, opcodes::DROP]),
        4 => program.push(opcodes::DROP4),
        _ => {} // TODO
    }
}

pub fn parse_swap(program: &mut Vec<u128>, op: &[&str]) {
    let param = read_param(op);
    match param {
        1 => program.push(opcodes::SWAP),
        2 => program.push(opcodes::SWAP2),
        3 => {}, // TODO
        4 => program.push(opcodes::SWAP4),
        _ => {} // TODO
    }
}

pub fn parse_roll(program: &mut Vec<u128>, op: &[&str]) {
    let param = read_param(op);
    match param {
        2 => program.push(opcodes::SWAP),
        4 => program.push(opcodes::ROLL4),
        8 => program.push(opcodes::ROLL8),
        _ => {} // TODO
    }
}

pub fn parse_add(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    program.push(opcodes::ADD);
}

pub fn parse_sub(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    program.extend_from_slice(&[opcodes::NEG, opcodes::ADD]);
}

pub fn parse_mul(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    program.push(opcodes::MUL);
}

pub fn parse_div(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    program.extend_from_slice(&[opcodes::INV, opcodes::MUL]);
}

pub fn parse_neg(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    program.push(opcodes::NEG);
}

pub fn parse_inv(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    program.push(opcodes::INV);
}

pub fn parse_not(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    program.push(opcodes::NOT);
}

pub fn parse_eq(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    program.push(opcodes::EQ);
}

pub fn parse_gt(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    let n = read_param(op);

    // prepare the stack
    let power_of_two = u128::pow(2, n);
    program.extend_from_slice(&[
        opcodes::PAD2, opcodes::PAD2, opcodes::PAD2, opcodes::PUSH, power_of_two
    ]);

    // execute CMP operations
    program.resize(program.len() + (n as usize), opcodes::CMP);

    // compare binary aggregation values with the original values, and drop everything
    // but the GT value from the stack
    program.extend_from_slice(&[
        opcodes::DROP, opcodes::SWAP4,  opcodes::ROLL4, opcodes::EQ,  opcodes::ASSERT,
        opcodes::EQ,   opcodes::ASSERT, opcodes::ROLL4, opcodes::DUP, opcodes::DROP4
    ]);
}

pub fn parse_lt(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    let n = read_param(op);

    // prepare the stack
    let power_of_two = u128::pow(2, n);
    program.extend_from_slice(&[
        opcodes::PAD2, opcodes::PAD2, opcodes::PAD2, opcodes::PUSH, power_of_two
    ]);

    // execute CMP operations
    program.resize(program.len() + (n as usize), opcodes::CMP);

    // compare binary aggregation values with the original values, and drop everything
    // but the LT value from the stack
    program.extend_from_slice(&[
        opcodes::DROP, opcodes::SWAP4,  opcodes::ROLL4, opcodes::EQ,   opcodes::ASSERT,
        opcodes::EQ,   opcodes::ASSERT, opcodes::DUP,   opcodes::DROP4
    ]);
}

pub fn parse_rc(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    let n = read_param(op);

    // prepare the stack
    let power_of_two = u128::pow(2, n);
    program.extend_from_slice(&[opcodes::PAD2, opcodes::DROP, opcodes::PUSH, power_of_two]);

    // execute BINACC operations
    program.resize(program.len() + (n as usize), opcodes::BINACC);

    // compare binary aggregation value with the original value
    program.extend_from_slice(&[opcodes::DROP, opcodes::EQ]);
}

pub fn parse_choose(program: &mut Vec<u128>, op: &[&str]) {
    let param = read_param(op);
    match param {
        1 => program.push(opcodes::CHOOSE),
        2 => program.push(opcodes::CHOOSE2),
        _ => {} // TODO
    }
}

pub fn parse_hash(program: &mut Vec<u128>, op: &[&str]) {
    if op[1] == "r" {
        program.push(opcodes::HASHR);
        return;
    }

    let param = read_param(op);
    match param {
        2 => {
            program.extend_from_slice(&[opcodes::PAD2, opcodes::PAD2]);
        },
        4 => {
            program.push(opcodes::PAD2);
        },
        _ => {} // TODO
    }

    // TODO: padding with NOOPs

    // append operations to execute 10 rounds of Rescue
    program.extend_from_slice(&[
        opcodes::HASHR, opcodes::HASHR, opcodes::HASHR, opcodes::HASHR, opcodes::HASHR,
        opcodes::HASHR, opcodes::HASHR, opcodes::HASHR, opcodes::HASHR, opcodes::HASHR
    ]);
}

pub fn parse_mpath(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
}

// HELPER FUNCTIONS
// ================================================================================================

fn read_param(op: &[&str]) -> u32 {
    if op.len() == 1 { return 1; };

    let result = match op[1].parse::<u32>() {
        Ok(i) => i,
        Err(e) => {
          1 // TODO
        }
    };

    return result;
}

fn read_value(op: &[&str]) -> u128 {

    let result = match op[1].parse::<u128>() {
        Ok(i) => i,
        Err(e) => {
          1 // TODO
        }
    };

    return result;
}