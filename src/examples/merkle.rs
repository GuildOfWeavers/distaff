use distaff::{ Program, ProgramInputs, opcodes::f128 as opcodes, math::{ FiniteField, F128 }, utils::Hasher };
use super::{ Example, utils::parse_args };

pub fn get_example(args: &[String]) -> Example  {

    // get the length of Merkle authentication path and proof options from the arguments
    let (depth, options) = parse_args(args);
    assert!(depth >= 2, "tree depth must be at least 2, but received {}", depth);
    
    // generate the program to verify Merkle path of given length
    let program = generate_merkle_program(depth);
    println!("Generated a program to verify Merkle proof for a tree of depth {}", depth);

    // generate a pseudo-random Merkle authentication path
    let (auth_path, leaf_index) = generate_authentication_path(depth);

    // compute root of the Merkle tree to which the path resolves
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

/// Returns a program to verify Merkle authentication path for a tree of depth `n`
fn generate_merkle_program(n: usize) -> Program {

    // the program starts by reading the first two nodes in the Merkle
    // path and pushing them onto the stack (each node is represented
    // by two field elements). This part also pads the stack to prepare
    // it for hashing.
    let mut program = vec![
        opcodes::BEGIN, opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
        opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,  opcodes::NOOP,
        opcodes::READ2, opcodes::READ2, opcodes::DUP4,  opcodes::PAD2
    ];

    // this cycle of operation gets repeated once for each remaining node. It does
    // roughly the following :
    // 1. computes hash(p, v)
    // 2. reads next bit of position index
    // 3. computes hash(v, p)
    // 4. based on position index bit, choses either hash(p, v) or hash(v, p)
    // 5. reads the next nodes and pushes it onto the stack
    let level_sub = vec![
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
        program.extend_from_slice(&level_sub);
    }

    // at the end, we use the same cycle except we don't need to read in
    // any more nodes - so, we omit the last 4 operations.
    program.extend_from_slice(&level_sub[..28]);

    return Program::from_path(program);
}

/// Converts Merkle authentication path for a node at the specified `index` into 
/// a set of inputs which can be consumed by the program created by the function above.
fn generate_program_inputs(path: &[Vec<F128>; 2], index: usize) -> ProgramInputs<F128> {

    let mut a = Vec::new();
    let mut b = Vec::new();
    let n = path[0].len();
    let mut index = index + usize::pow(2, (n - 1) as u32);

    // push the leaf node of the authentication path onto secret input tapes A and B
    a.push(path[0][0]);
    b.push(path[1][0]);

    for i in 1..n {
        // push the next node onto tapes A and B
        a.push(path[0][i]);
        b.push(path[1][i]);

        // push next bit of the position index onto tapes A and B; we use both tapes
        // here so that we can use READ2 instruction when reading inputs from the tapes
        a.push(F128::ZERO);
        b.push(F128::from_usize(index & 1));
        index = index >> 1;
    }

    return ProgramInputs::new(&[], &a, &b);
}

/// Pseudo-randomly generates a Merkle authentication path for an imaginary Merkle tree
/// of depth equal to `n`
fn generate_authentication_path(n: usize) -> ([Vec<F128>; 2], usize) {
    let mut s1 = [0u8; 32];
    s1[0] = 1; s1[1] = 2; s1[2] = 3;
    let mut s2 = [0u8; 32];
    s2[0] = 4; s2[1] = 5; s2[2] = 6;

    let leaves = u128::pow(2, (n - 1) as u32);
    let leaf_index = (F128::prng(s1) % leaves) as usize;

    return ([F128::prng_vector(s1, n), F128::prng_vector(s2, n)], leaf_index);
}

/// Computes tree root to which a given authentication path resolves assuming the
/// path is for a leaf node at position specified by `index` parameter.
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