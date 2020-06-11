use super::{ opcodes::f128 as opcodes };

pub fn translate(assembly: &str) {

    println!("{}", assembly);

    let mut program = vec![opcodes::BEGIN];

    for token in assembly.split_whitespace() {

        let parts: Vec<&str> = token.split(".").collect();

        match parts[0] {
            "noop"   => parse_noop(&mut program, &parts),
            "assert" => parse_assert(&mut program, &parts),

            "push"   => parse_push(&mut program, &parts),
            "read"   => parse_read(&mut program, &parts),

            "dup"    => parse_dup(&mut program, &parts),
            "pad"    => parse_pad(&mut program, &parts),
            "drop"   => parse_drop(&mut program, &parts),
            "swap"   => parse_swap(&mut program, &parts),
            "roll"   => parse_roll(&mut program, &parts),

            "add"   => parse_add(&mut program, &parts),
            "sub"   => parse_sub(&mut program, &parts),
            "mul"   => parse_mul(&mut program, &parts),
            "div"   => parse_div(&mut program, &parts),
            "neg"   => parse_neg(&mut program, &parts),
            "inv"   => parse_inv(&mut program, &parts),
            "not"   => parse_not(&mut program, &parts),

            "eq"     => parse_eq(&mut program, &parts),
            _ => {
                println!("not found: {:?}", parts);
            }
        }
    }

    println!("{:?}", program);
}

// OPERATION PARSERS
// ================================================================================================
fn parse_noop(program: &mut Vec<u128>, op: &[&str]) {
    let param = read_param(op);
    for _ in 0..param {
        program.push(opcodes::NOOP);
    }
}

fn parse_assert(program: &mut Vec<u128>, op: &[&str]) {
    let param = read_param(op);
    for _ in 0..param {
        program.push(opcodes::ASSERT);
    }
}

fn parse_push(program: &mut Vec<u128>, op: &[&str]) {
    let value = read_value(op);
    program.extend_from_slice(&[opcodes::PUSH, value]);
}

fn parse_read(program: &mut Vec<u128>, op: &[&str]) {
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

fn parse_dup(program: &mut Vec<u128>, op: &[&str]) {
    let param = read_param(op);
    match param {
        1 => program.push(opcodes::DUP),
        2 => program.push(opcodes::DUP2),
        3 => program.extend_from_slice(&[opcodes::DUP4, opcodes::ROLL4, opcodes::DROP]),
        4 => program.push(opcodes::DUP4),
        _ => {} // TODO
    }
}

fn parse_pad(program: &mut Vec<u128>, op: &[&str]) {
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

fn parse_drop(program: &mut Vec<u128>, op: &[&str]) {
    let param = read_param(op);
    match param {
        1 => program.push(opcodes::DROP),
        2 => program.extend_from_slice(&[opcodes::DROP, opcodes::DROP]),
        3 => program.extend_from_slice(&[opcodes::DROP, opcodes::DROP, opcodes::DROP]),
        4 => program.push(opcodes::DROP4),
        _ => {} // TODO
    }
}

fn parse_swap(program: &mut Vec<u128>, op: &[&str]) {
    let param = read_param(op);
    match param {
        1 => program.push(opcodes::SWAP),
        2 => program.push(opcodes::SWAP2),
        3 => {}, // TODO
        4 => program.push(opcodes::SWAP4),
        _ => {} // TODO
    }
}

fn parse_roll(program: &mut Vec<u128>, op: &[&str]) {
    let param = read_param(op);
    match param {
        2 => program.push(opcodes::SWAP),
        4 => program.push(opcodes::ROLL4),
        8 => program.push(opcodes::ROLL8),
        _ => {} // TODO
    }
}

fn parse_add(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    program.push(opcodes::ADD);
}

fn parse_sub(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    program.extend_from_slice(&[opcodes::NEG, opcodes::ADD]);
}

fn parse_mul(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    program.push(opcodes::MUL);
}

fn parse_div(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    program.extend_from_slice(&[opcodes::INV, opcodes::MUL]);
}

fn parse_neg(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    program.push(opcodes::NEG);
}

fn parse_inv(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    program.push(opcodes::INV);
}

fn parse_not(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    program.push(opcodes::NOT);
}

fn parse_eq(program: &mut Vec<u128>, op: &[&str]) {
    // TODO: check for parameters
    program.push(opcodes::EQ);
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