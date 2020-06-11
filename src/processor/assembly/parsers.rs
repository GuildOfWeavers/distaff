use crate::math::{ F128, FiniteField };
use super::{ opcodes, AssemblyError };

// OPERATION PARSERS
// ================================================================================================
pub fn parse_noop(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    let param = read_param(op, step).unwrap();
    for _ in 0..param {
        program.push(opcodes::NOOP);
    }

    return Ok(true);
}

pub fn parse_assert(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    let param = read_param(op, step).unwrap();
    for _ in 0..param {
        program.push(opcodes::ASSERT);
    }

    return Ok(true);
}

pub fn parse_push(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    let value = read_value(op, step).unwrap();
    program.extend_from_slice(&[opcodes::PUSH, value]);
    return Ok(true);
}

pub fn parse_read(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    if op.len() == 1 || op[1] == "a" {
        program.push(opcodes::READ);
    }
    else if op[1] == "ab" {
        program.push(opcodes::READ2);
    }
    else {
        // TODO: error
    }

    return Ok(true);
}

pub fn parse_dup(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    let param = read_param(op, step).unwrap();
    match param {
        1 => program.push(opcodes::DUP),
        2 => program.push(opcodes::DUP2),
        3 => program.extend_from_slice(&[opcodes::DUP4, opcodes::ROLL4, opcodes::DROP]),
        4 => program.push(opcodes::DUP4),
        _ => {} // TODO
    };

    return Ok(true);
}

pub fn parse_pad(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    let param = read_param(op, step).unwrap();
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

    return Ok(true);
}

pub fn parse_drop(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    let param = read_param(op, step).unwrap();
    match param {
        1 => program.push(opcodes::DROP),
        2 => program.extend_from_slice(&[opcodes::DROP, opcodes::DROP]),
        3 => program.extend_from_slice(&[opcodes::DROP, opcodes::DROP, opcodes::DROP]),
        4 => program.push(opcodes::DROP4),
        _ => {} // TODO
    }

    return Ok(true);
}

pub fn parse_swap(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    let param = read_param(op, step).unwrap();
    match param {
        1 => program.push(opcodes::SWAP),
        2 => program.push(opcodes::SWAP2),
        3 => {}, // TODO
        4 => program.push(opcodes::SWAP4),
        _ => {} // TODO
    }

    return Ok(true);
}

pub fn parse_roll(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    let param = read_param(op, step).unwrap();
    match param {
        2 => program.push(opcodes::SWAP),
        4 => program.push(opcodes::ROLL4),
        8 => program.push(opcodes::ROLL8),
        _ => {} // TODO
    }

    return Ok(true);
}

pub fn parse_add(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    // TODO: check for parameters
    program.push(opcodes::ADD);
    return Ok(true);
}

pub fn parse_sub(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    // TODO: check for parameters
    program.extend_from_slice(&[opcodes::NEG, opcodes::ADD]);
    return Ok(true);
}

pub fn parse_mul(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    // TODO: check for parameters
    program.push(opcodes::MUL);
    return Ok(true);
}

pub fn parse_div(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    // TODO: check for parameters
    program.extend_from_slice(&[opcodes::INV, opcodes::MUL]);
    return Ok(true);
}

pub fn parse_neg(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    // TODO: check for parameters
    program.push(opcodes::NEG);
    return Ok(true);
}

pub fn parse_inv(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    // TODO: check for parameters
    program.push(opcodes::INV);
    return Ok(true);
}

pub fn parse_not(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    // TODO: check for parameters
    program.push(opcodes::NOT);
    return Ok(true);
}

pub fn parse_eq(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    // TODO: check for parameters
    program.push(opcodes::EQ);
    return Ok(true);
}

pub fn parse_gt(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    // TODO: check for parameters
    let n = read_param(op, step).unwrap();

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
    return Ok(true);
}

pub fn parse_lt(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    // TODO: check for parameters
    let n = read_param(op, step).unwrap();

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
    return Ok(true);
}

pub fn parse_rc(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    // TODO: check for parameters
    let n = read_param(op, step).unwrap();

    // prepare the stack
    let power_of_two = u128::pow(2, n);
    program.extend_from_slice(&[opcodes::PAD2, opcodes::DROP, opcodes::PUSH, power_of_two]);

    // execute BINACC operations
    program.resize(program.len() + (n as usize), opcodes::BINACC);

    // compare binary aggregation value with the original value
    program.extend_from_slice(&[opcodes::DROP, opcodes::EQ]);
    return Ok(true);
}

pub fn parse_choose(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    let param = read_param(op, step).unwrap();
    match param {
        1 => program.push(opcodes::CHOOSE),
        2 => program.push(opcodes::CHOOSE2),
        _ => {} // TODO
    }
    return Ok(true);
}

pub fn parse_hash(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    if op[1] == "r" {
        program.push(opcodes::HASHR);
        return Ok(true);
    }

    let param = read_param(op, step).unwrap();
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
    return Ok(true);
}

pub fn parse_mpath(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, String> {
    // TODO: check for parameters
    return Ok(true);
}

// HELPER FUNCTIONS
// ================================================================================================

fn read_param(op: &[&str], step: usize) -> Result<u32, AssemblyError> {
    if op.len() == 1 {
        // if no parameters were provided, assume parameter value 1
        return Ok(1);
    } else if op.len() > 2 {
        return Err(AssemblyError::extra_param(op, step));
    }

    // try to parse the parameter value
    let result = match op[1].parse::<u32>() {
        Ok(i) => i,
        Err(_) => return Err(AssemblyError::invalid_param(op, step))
    };

    // parameter value 0 is never valid
    if result == 0 {
        return Err(AssemblyError::invalid_param_reason(op,
            format!("parameter value must be greater than 0"), step));
    }

    return Ok(result);
}

fn read_value(op: &[&str], step: usize) -> Result<u128, AssemblyError> {
    // make sure exactly 1 parameter was supplied
    if op.len() == 1 {
        return Err(AssemblyError::missing_param(op, step));
    }
    else if op.len() > 2 {
        return Err(AssemblyError::extra_param(op, step));
    }

    let result = if op[1].starts_with("0x") {
        // parse hexadecimal number
        match u128::from_str_radix(&op[1][2..], 16) {
            Ok(i) => i,
            Err(_) => return Err(AssemblyError::invalid_param(op, step))
        }
    }
    else {
        // parse decimal number
        match u128::from_str_radix(&op[1], 10) {
            Ok(i) => i,
            Err(_) => return Err(AssemblyError::invalid_param(op, step))
        }
    };

    // make sure the value is a valid field element
    if result >= F128::MODULUS {
        return Err(AssemblyError::invalid_param_reason(op,
            format!("parameter value must be smaller than {}", F128::MODULUS), step));
    }

    return Ok(result);
}