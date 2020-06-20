use crate::math::{ F128, FiniteField };
use super::{ opcodes, AssemblyError, HintMap, ExecutionHint };

// CONTROL FLOW OPERATIONS
// ================================================================================================

/// Appends a NOOP operations to the program.
pub fn parse_noop(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    if op.len() > 1 {
        return Err(AssemblyError::extra_param(op, step));
    }
    program.push(opcodes::NOOP);
    return Ok(true);
}

/// Appends an ASSERT operations to the program.
pub fn parse_assert(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    if op.len() > 1 {
        return Err(AssemblyError::extra_param(op, step));
    }
    program.push(opcodes::ASSERT);
    return Ok(true);
}

// INPUT OPERATIONS
// ================================================================================================

/// Extends the program by a PUSH operation followed by the value to be pushed onto the stack.
pub fn parse_push(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    let value = read_value(op, step)?;
    program.extend_from_slice(&[opcodes::PUSH, value]);
    return Ok(true);
}

/// Appends either READ or READ2 operation to the program.
pub fn parse_read(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    if op.len() > 2 {
        return Err(AssemblyError::extra_param(op, step));
    }
    else if op.len() == 1 || op[1] == "a" {
        program.push(opcodes::READ);
    }
    else if op[1] == "ab" {
        program.push(opcodes::READ2);
    }
    else {
        return Err(AssemblyError::invalid_param_reason(op, step,
            format!("parameter {} is invalid; allowed values are: [a, ab]", op[1])));
    }

    return Ok(true);
}

// STACK MANIPULATION OPERATIONS
// ================================================================================================

/// Appends a sequence of operations to the program to duplicate top n values of the stack.
pub fn parse_dup(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    let n = read_param(op, step)?;
    match n {
        1 => program.push(opcodes::DUP),
        2 => program.push(opcodes::DUP2),
        3 => program.extend_from_slice(&[opcodes::DUP4, opcodes::ROLL4, opcodes::DROP]),
        4 => program.push(opcodes::DUP4),
        _ => return Err(AssemblyError::invalid_param_reason(op, step,
            format!("parameter {} is invalid; allowed values are: [1, 2, 3, 4]", n)))
    };

    return Ok(true);
}

/// Appends a sequence of operations to the program to pad the stack with n zeros.
pub fn parse_pad(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    let n = read_param(op, step)?;
    match n {
        1 => program.extend_from_slice(&[opcodes::PAD2, opcodes::DROP]),
        2 => program.push(opcodes::PAD2),
        3 => program.extend_from_slice(&[opcodes::PAD2, opcodes::PAD2, opcodes::DROP]),
        4 => program.extend_from_slice(&[opcodes::PAD2, opcodes::PAD2]),
        5 => program.extend_from_slice(&[opcodes::PAD2, opcodes::PAD2, opcodes::PAD2, opcodes::DROP]),
        6 => program.extend_from_slice(&[opcodes::PAD2, opcodes::PAD2, opcodes::PAD2]),
        7 => program.extend_from_slice(&[opcodes::PAD2, opcodes::PAD2, opcodes::DUP4, opcodes::DROP]),
        8 => program.extend_from_slice(&[opcodes::PAD2, opcodes::PAD2, opcodes::DUP4]),
        _ => return Err(AssemblyError::invalid_param_reason(op, step,
            format!("parameter {} is invalid; allowed values are: [1, 2, 3, 4, 5, 6, 7, 8]", n)))
    }

    return Ok(true);
}

/// Appends a sequence of operations to the program to copy n-th item to the top of the stack.
pub fn parse_pick(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    let n = read_param(op, step)?;
    match n {
        1 => program.extend_from_slice(&[opcodes::DUP2, opcodes::DROP]),
        2 => program.extend_from_slice(&[
            opcodes::DUP4, opcodes::ROLL4, opcodes::DROP, opcodes::DROP, opcodes::DROP
        ]),
        3 => program.extend_from_slice(&[opcodes::DUP4, opcodes::DROP, opcodes::DROP, opcodes::DROP]),
        _ => return Err(AssemblyError::invalid_param_reason(op, step,
            format!("parameter {} is invalid; allowed values are: [1, 2, 3]", n)))
    };

    return Ok(true);
}

/// Appends a sequence of operations to the program to remove top n values from the stack.
pub fn parse_drop(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    let n = read_param(op, step)?;
    match n {
        1 => program.push(opcodes::DROP),
        2 => program.extend_from_slice(&[opcodes::DROP, opcodes::DROP]),
        3 => program.extend_from_slice(&[opcodes::DUP, opcodes::DROP4]),
        4 => program.push(opcodes::DROP4),
        5 => program.extend_from_slice(&[opcodes::DROP, opcodes::DROP4]),
        6 => program.extend_from_slice(&[opcodes::DROP, opcodes::DROP, opcodes::DROP4]),
        7 => program.extend_from_slice(&[opcodes::DUP, opcodes::DROP4, opcodes::DROP4]),
        8 => program.extend_from_slice(&[opcodes::DROP4, opcodes::DROP4]),
        _ => return Err(AssemblyError::invalid_param_reason(op, step,
            format!("parameter {} is invalid; allowed values are: [1, 2, 3, 4, 5, 6, 7, 8]", n)))
    }

    return Ok(true);
}

/// Appends a sequence of operations to the program to swap n values at the top of the stack
/// with the following n values.
pub fn parse_swap(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    let n = read_param(op, step)?;
    match n {
        1 => program.push(opcodes::SWAP),
        2 => program.push(opcodes::SWAP2),
        4 => program.push(opcodes::SWAP4),
        _ => return Err(AssemblyError::invalid_param_reason(op, step,
            format!("parameter {} is invalid; allowed values are: [1, 2, 4]", n)))
    }

    return Ok(true);
}

/// Appends either ROLL4 or ROLL8 operation to the program.
pub fn parse_roll(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    let n = read_param(op, step)?;
    match n {
        4 => program.push(opcodes::ROLL4),
        8 => program.push(opcodes::ROLL8),
        _ => return Err(AssemblyError::invalid_param_reason(op, step,
            format!("parameter {} is invalid; allowed values are: [4, 8]", n)))
    }

    return Ok(true);
}

// ARITHMETIC AND BOOLEAN OPERATIONS
// ================================================================================================

/// Appends ADD operation to the program.
pub fn parse_add(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    if op.len() > 1 { return Err(AssemblyError::extra_param(op, step)); }
    program.push(opcodes::ADD);
    return Ok(true);
}

/// Appends NEG ADD operations to the program.
pub fn parse_sub(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    if op.len() > 1 { return Err(AssemblyError::extra_param(op, step)); }
    program.extend_from_slice(&[opcodes::NEG, opcodes::ADD]);
    return Ok(true);
}

/// Appends MUL operation to the program.
pub fn parse_mul(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    if op.len() > 1 { return Err(AssemblyError::extra_param(op, step)); }
    program.push(opcodes::MUL);
    return Ok(true);
}

/// Appends INV MUL operations to the program.
pub fn parse_div(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    if op.len() > 1 { return Err(AssemblyError::extra_param(op, step)); }
    program.extend_from_slice(&[opcodes::INV, opcodes::MUL]);
    return Ok(true);
}

/// Appends NEG operation to the program.
pub fn parse_neg(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    if op.len() > 1 { return Err(AssemblyError::extra_param(op, step)); }
    program.push(opcodes::NEG);
    return Ok(true);
}

/// Appends INV operation to the program.
pub fn parse_inv(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    if op.len() > 1 { return Err(AssemblyError::extra_param(op, step)); }
    program.push(opcodes::INV);
    return Ok(true);
}

/// Appends NOT operation to the program.
pub fn parse_not(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    if op.len() > 1 { return Err(AssemblyError::extra_param(op, step)); }
    program.push(opcodes::NOT);
    return Ok(true);
}

// COMPARISON OPERATIONS
// ================================================================================================

/// Appends EQ operation to the program.
pub fn parse_eq(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    if op.len() > 1 { return Err(AssemblyError::extra_param(op, step)); }
    program.push(opcodes::EQ);
    return Ok(true);
}

/// Appends a sequence of operations to the program to determine whether the top value on the 
/// stack is greater than the following value.
pub fn parse_gt(program: &mut Vec<u128>, hints: &mut HintMap, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    // n is the number of bits sufficient to represent each value; if either of the
    // values does not fit into n bits, the operation fill fail.
    let n = read_param(op, step)?;
    if n < 4 || n > 128 {
        return Err(AssemblyError::invalid_param_reason(op, step,
            format!("parameter {} is invalid; value must be between 4 and 128", n)))
    }

    // prepare the stack
    let power_of_two = u128::pow(2, n - 1);
    program.extend_from_slice(&[
        opcodes::PAD2, opcodes::PAD2, opcodes::PAD2, opcodes::PUSH, power_of_two
    ]);

    // add a hint indicating that value comparison is about to start
    hints.insert(program.len(), ExecutionHint::CmpStart(n));

    // append CMP operations
    program.resize(program.len() + (n as usize), opcodes::CMP);

    // compare binary aggregation values with the original values, and drop everything
    // but the GT value from the stack
    program.extend_from_slice(&[
        opcodes::DROP, opcodes::SWAP4,  opcodes::ROLL4, opcodes::EQ,  opcodes::ASSERT,
        opcodes::EQ,   opcodes::ASSERT, opcodes::ROLL4, opcodes::DUP, opcodes::DROP4
    ]);
    return Ok(true);
}

/// Appends a sequence of operations to the program to determine whether the top value on the 
/// stack is less than the following value.
pub fn parse_lt(program: &mut Vec<u128>, hints: &mut HintMap, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    // n is the number of bits sufficient to represent each value; if either of the
    // values does not fit into n bits, the operation fill fail.
    let n = read_param(op, step)?;
    if n < 4 || n > 128 {
        return Err(AssemblyError::invalid_param_reason(op, step,
            format!("parameter {} is invalid; value must be between 4 and 128", n)))
    }

    // prepare the stack
    let power_of_two = u128::pow(2, n - 1);
    program.extend_from_slice(&[
        opcodes::PAD2, opcodes::PAD2, opcodes::PAD2, opcodes::PUSH, power_of_two
    ]);

    // add a hint indicating that value comparison is about to start
    hints.insert(program.len(), ExecutionHint::CmpStart(n));

    // append CMP operations
    program.resize(program.len() + (n as usize), opcodes::CMP);

    // compare binary aggregation values with the original values, and drop everything
    // but the LT value from the stack
    program.extend_from_slice(&[
        opcodes::DROP, opcodes::SWAP4,  opcodes::ROLL4, opcodes::EQ,   opcodes::ASSERT,
        opcodes::EQ,   opcodes::ASSERT, opcodes::DUP,   opcodes::DROP4
    ]);
    return Ok(true);
}

/// Appends a sequence of operations to the program to determine whether the top value on the 
/// stack can be represented with n bits.
pub fn parse_rc(program: &mut Vec<u128>, hints: &mut HintMap, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    // n is the number of bits sufficient to represent each value; if either of the
    // values does not fit into n bits, the operation fill fail.
    let n = read_param(op, step)?;
    if n < 4 || n > 128 {
        return Err(AssemblyError::invalid_param_reason(op, step,
            format!("parameter {} is invalid; value must be between 4 and 128", n)))
    }

    // prepare the stack
    let power_of_two = u128::pow(2, n - 1);
    program.extend_from_slice(&[opcodes::PAD2, opcodes::DROP, opcodes::PUSH, power_of_two]);

    // add a hint indicating that range-checking is about to start
    hints.insert(program.len(), ExecutionHint::RcStart(n));

    // append BINACC operations
    program.resize(program.len() + (n as usize), opcodes::BINACC);

    // compare binary aggregation value with the original value
    program.extend_from_slice(&[opcodes::DROP, opcodes::EQ]);
    return Ok(true);
}

// SELECTOR OPERATIONS
// ================================================================================================

/// Appends either CHOOSE or CHOOSE2 operation to the program.
pub fn parse_choose(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    let n = read_param(op, step)?;
    match n {
        1 => program.push(opcodes::CHOOSE),
        2 => program.push(opcodes::CHOOSE2),
        _ => return Err(AssemblyError::invalid_param_reason(op, step,
            format!("parameter {} is invalid; allowed values are: [1, 2]", n)))
    }
    return Ok(true);
}

// CRYPTO OPERATIONS
// ================================================================================================

/// Appends a sequence of operations to the program to hash top n values of the stack.
pub fn parse_hash(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    let n = read_param(op, step)?;
    match n {
        1 => program.extend_from_slice(&[opcodes::PAD2, opcodes::PAD2, opcodes::PAD2, opcodes::DROP]),
        2 => program.extend_from_slice(&[opcodes::PAD2, opcodes::PAD2]),
        3 => program.extend_from_slice(&[opcodes::PAD2, opcodes::PAD2, opcodes::DROP]),
        4 => program.push(opcodes::PAD2),
        _ => return Err(AssemblyError::invalid_param_reason(op, step,
            format!("parameter {} is invalid; allowed values are: [1, 2, 3, 4]", n)))
    }

    // pad with NOOPs to make sure hashing starts on a step which is a multiple of 16
    let m = 16 - (program.len() % 16);
    program.resize(program.len() + m, opcodes::NOOP);

    // append operations to execute 10 rounds of Rescue
    program.extend_from_slice(&[
        opcodes::RESCR, opcodes::RESCR, opcodes::RESCR, opcodes::RESCR, opcodes::RESCR,
        opcodes::RESCR, opcodes::RESCR, opcodes::RESCR, opcodes::RESCR, opcodes::RESCR
    ]);

    // truncate the state
    program.push(opcodes::DROP4);

    return Ok(true);
}

/// Appends a sequence of operations to the program to compute the root of Merkle
/// authentication path for a tree of depth n.
pub fn parse_mpath(program: &mut Vec<u128>, op: &[&str], step: usize) -> Result<bool, AssemblyError> {
    let n = read_param(op, step)?;
    if n < 2 || n > 256 {
        return Err(AssemblyError::invalid_param_reason(op, step,
            format!("parameter {} is invalid; value must be between 2 and 256", n)))
    }

    // read the first node in the Merkle path and push it onto the stack;
    // also pad the stack to prepare it for hashing.
    program.extend_from_slice(&[opcodes::READ2, opcodes::DUP4, opcodes::PAD2]);

    // pad with NOOPs to make sure hashing starts on a step which is a multiple of 16
    let m = 16 - (program.len() % 16);
    program.resize(program.len() + m, opcodes::NOOP);

    // repeat the following cycle of operations once for each remaining node:
    // 1. compute hash(p, v)
    // 2. read next bit of position index
    // 3. compute hash(v, p)
    // 4. base on position index bit, choses either hash(p, v) or hash(v, p)
    // 5. reads the next nodes and pushes it onto the stack
    const SUB_CYCLE: [u128; 32] = [
        opcodes::RESCR, opcodes::RESCR, opcodes::RESCR, opcodes::RESCR,
        opcodes::RESCR, opcodes::RESCR, opcodes::RESCR, opcodes::RESCR,
        opcodes::RESCR, opcodes::RESCR, opcodes::DROP4, opcodes::READ2,
        opcodes::SWAP2, opcodes::SWAP4, opcodes::SWAP2, opcodes::PAD2,
        opcodes::RESCR, opcodes::RESCR, opcodes::RESCR, opcodes::RESCR,
        opcodes::RESCR, opcodes::RESCR, opcodes::RESCR, opcodes::RESCR,
        opcodes::RESCR, opcodes::RESCR, opcodes::DROP4, opcodes::CHOOSE2,
        opcodes::READ2, opcodes::DUP4,  opcodes::PAD2,  opcodes::NOOP
    ];

    for _ in 0..(n - 2) {
        program.extend_from_slice(&SUB_CYCLE);
    }

    // at the end, use the same cycle except for the last 4 operations
    // since there is no need to read in any additional nodes
    program.extend_from_slice(&SUB_CYCLE[..28]);

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
        return Err(AssemblyError::invalid_param_reason(op, step,
            format!("parameter value must be greater than 0")));
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
        return Err(AssemblyError::invalid_param_reason(op, step,
            format!("parameter value must be smaller than {}", F128::MODULUS)));
    }

    return Ok(result);
}