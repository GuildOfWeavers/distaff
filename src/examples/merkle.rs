use distaff::{ ProgramInputs, processor::opcodes::f128 as opcodes, FiniteField, F128, stark::Hasher };
use super::{ Example, utils::parse_args };

pub fn get_example(args: &[String]) -> Example  {

    // get the length of Merkle authentication path and proof options from the arguments
    let (depth, options) = parse_args(args);
    assert!(depth >= 2, "tree depth must be at least 2, but received {}", depth);
    
    // generate the program to verify Merkle path of given length
    let program = generate_merkle_program(depth);
    println!("Generated a program to verify Merkle proof for a tree of depth {}", depth);

    // compute root of the Merkle tree to which the path resolves
    let auth_path = generate_authentication_path(depth);
    let leaf_index = 2;
    let mut expected_result = compute_merkle_root(&auth_path, leaf_index);
    println!("Expected tree root: {:?}", expected_result);

    // transform Merkle path into a set of inputs for the program
    let inputs = generate_program_inputs(&auth_path, leaf_index);

    // a single element from the top of the stack will be the output
    let num_outputs = 2;

    // reverse tree root because values on the stack are in reverse order
    expected_result.reverse();

    return Example {
        program,
        inputs,
        options,
        expected_result,
        num_outputs
    };
}

fn generate_merkle_program(n: usize) -> Vec<F128> {
    let mut program = vec![
        opcodes::BEGIN, opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
        opcodes::READ2, opcodes::READ2, opcodes::DUP4,  opcodes::PAD2
    ];

    let level_sub = vec![
        opcodes::HASHR, opcodes::HASHR, opcodes::HASHR, opcodes::HASHR,
        opcodes::HASHR, opcodes::HASHR, opcodes::HASHR, opcodes::HASHR,
        opcodes::HASHR, opcodes::HASHR, opcodes::DROP4, opcodes::READ2,
        opcodes::SWAP2, opcodes::SWAP4, opcodes::SWAP2, opcodes::PAD2,
        opcodes::HASHR, opcodes::HASHR, opcodes::HASHR, opcodes::HASHR,
        opcodes::HASHR, opcodes::HASHR, opcodes::HASHR, opcodes::HASHR,
        opcodes::HASHR, opcodes::HASHR, opcodes::DROP4, opcodes::CHOOSE2,
        opcodes::READ2, opcodes::DUP4,  opcodes::PAD2,  opcodes::NOOP
    ];

    for _ in 0..(n - 2) {
        program.extend_from_slice(&level_sub);
    }

    program.extend_from_slice(&level_sub[..28]);

    return program;
}

fn generate_authentication_path(n: usize) -> [Vec<F128>; 2] {
    let mut s1 = [0u8; 32];
    s1[0] = 1; s1[1] = 2; s1[2] = 3;
    let mut s2 = [0u8; 32];
    s2[0] = 4; s2[1] = 5; s2[2] = 6;

    return [
        F128::prng_vector(s1, n),
        F128::prng_vector(s2, n),
    ];
}

fn generate_program_inputs(path: &[Vec<F128>; 2], index: usize) -> ProgramInputs<F128> {

    let mut a = Vec::new();
    let mut b = Vec::new();
    let n = path[0].len();
    let mut index = index + usize::pow(2, n as u32);

    a.push(path[0][0]);
    b.push(path[1][0]);

    for i in 1..n {
        a.push(path[0][i]);
        b.push(path[1][i]);

        a.push(F128::ZERO);
        b.push(F128::from_usize(index & 1));
        index = index >> 1;
    }

    return ProgramInputs::new(&[], &a, &b);
}

fn compute_merkle_root(path: &[Vec<F128>; 2], index: usize) -> Vec<F128> {

    let mut buf = [F128::ZERO; 4];
    let mut v: Vec<F128>;
    let n = path[0].len();

    let r = index & 1;
    buf[0] = path[0][r];
    buf[1] = path[1][r];
    buf[2] = path[0][1 - r];
    buf[3] = path[1][1 - r];

    v = F128::digest(&buf);

    let mut index = (index + usize::pow(2, (n - 1) as u32)) >> 1;
    for i in 2..n {
        if index & 1 == 0 {
            buf[0] = v[0];
            buf[1] = v[1];
            buf[2] = path[0][i];
            buf[3] = path[1][i];
        }
        else {
            buf[0] = path[0][i];
            buf[1] = path[1][i];
            buf[2] = v[0];
            buf[3] = v[1];
        }
        
        v = F128::digest(&buf);
        index = index >> 1;
    }

    return v.to_vec();
}