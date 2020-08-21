use distaff::{ Program, ProgramInputs, assembly, math::field, utils::hasher };
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

    let source = format!("
    begin
        read.ab
        smpath.{}
    end
    ", n);

    return assembly::compile(&source).unwrap();
}

/// Converts Merkle authentication path for a node at the specified `index` into 
/// a set of inputs which can be consumed by the program created by the function above.
fn generate_program_inputs(path: &[Vec<u128>; 2], index: usize) -> ProgramInputs {

    let mut a = Vec::new();
    let mut b = Vec::new();
    let n = path[0].len();
    let mut index = index + usize::pow(2, (n - 1) as u32);

    // push the leaf node onto secret input tapes A and B
    a.push(path[0][0]);
    b.push(path[1][0]);

    for i in 1..n {
        // push next bit of the position index onto tapes A and B; we use both tapes
        // here so that we can use READ2 instruction when reading inputs from the tapes
        a.push(field::ZERO);
        b.push((index & 1) as u128);
        index = index >> 1;

        // push the next node onto tapes A and B
        a.push(path[0][i]);
        b.push(path[1][i]);
    }

    return ProgramInputs::new(&[], &a, &b);
}

/// Pseudo-randomly generates a Merkle authentication path for an imaginary Merkle tree
/// of depth equal to `n`
fn generate_authentication_path(n: usize) -> ([Vec<u128>; 2], usize) {
    let mut s1 = [0u8; 32];
    s1[0] = 1; s1[1] = 2; s1[2] = 3;
    let mut s2 = [0u8; 32];
    s2[0] = 4; s2[1] = 5; s2[2] = 6;

    let leaves = u128::pow(2, (n - 1) as u32);
    let leaf_index = (field::prng(s1) % leaves) as usize;

    return ([field::prng_vector(s1, n), field::prng_vector(s2, n)], leaf_index);
}

/// Computes tree root to which a given authentication path resolves assuming the
/// path is for a leaf node at position specified by `index` parameter.
fn compute_merkle_root(path: &[Vec<u128>; 2], index: usize) -> Vec<u128> {

    let mut buf = [field::ZERO; 4];
    let mut v: Vec<u128>;
    let n = path[0].len();

    let r = index & 1;
    buf[0] = path[0][r];
    buf[1] = path[1][r];
    buf[2] = path[0][1 - r];
    buf[3] = path[1][1 - r];

    v = hasher::digest(&buf);

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
        
        v = hasher::digest(&buf);
        index = index >> 1;
    }

    return v.to_vec();
}