use std::env;
use std::io::Write;
use std::time::Instant;
use distaff::{ 
    ProofOptions,
    ProgramInputs,
    StarkProof,
    processor,
    processor::opcodes::f128 as opcodes,
    FiniteField, F128 };

fn main() {

    // configure logging
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter_level(log::LevelFilter::Debug).init();

    // read the length of Fibonacci sequence and proof options from the command line
    let (n, options) = read_args();
    
    // generate the program and expected results
    let program = generate_fibonacci_program(n);
    let expected_result = compute_fibonacci(n);
    let expected_hash = processor::hash_program(&program);
    println!("Generated a program to compute {}-th Fibonacci term; expected result: {}", 
        n,
        expected_result);
    println!("--------------------------------");

    // initialize stack with 2 values; 1 will be at the top
    let inputs = ProgramInputs::from_public(&[1, 0]);

    // a single element from the top of the stack will be the output
    let num_outputs = 1;

    // execute the program and generate the proof of execution
    let now = Instant::now();
    let (outputs, program_hash, proof) = processor::execute(&program, &inputs, num_outputs, &options);
    println!("--------------------------------");
    println!("Executed program with hash {} in {} ms", 
        hex::encode(program_hash),
        now.elapsed().as_millis());
    println!("Program output: {:?}", outputs);
    assert_eq!(expected_result, outputs[0], "Program result was computed incorrectly");
    assert_eq!(expected_hash, program_hash, "Program hash was generated incorrectly");

    // serialize the proof to see how big it is
    let proof_bytes = bincode::serialize(&proof).unwrap();
    println!("Execution proof size: {} KB", proof_bytes.len() / 1024);
    println!("Execution proof security: {} bits", options.security_level(true));
    println!("--------------------------------");

    // verify that executing a program with a given hash and given inputs
    // results in the expected output
    let proof = bincode::deserialize::<StarkProof<F128>>(&proof_bytes).unwrap();
    let now = Instant::now();
    match processor::verify(&program_hash, inputs.get_public_inputs(), &outputs, &proof) {
        Ok(_) => println!("Execution verified in {} ms", now.elapsed().as_millis()),
        Err(msg) => println!("Failed to verify execution: {}", msg)
    }
}

fn read_args() -> (usize, ProofOptions) {
    let default_options = ProofOptions::default();

    let args: Vec<String> = env::args().collect();
    if args.len() == 1 { return (6, default_options); }

    let n: usize = args[1].parse().unwrap();
    if args.len() == 2 { return (n, default_options); }

    let ext_factor: usize;
    let num_queries: usize;
    let grind_factor: u32;
    
    if args.len() == 3 {
        ext_factor = args[2].parse().unwrap();
        num_queries = default_options.num_queries();
        grind_factor = default_options.grinding_factor();
    }
    else if args.len() == 4 {
        ext_factor = args[2].parse().unwrap();
        num_queries = args[3].parse().unwrap();
        grind_factor = default_options.grinding_factor();
    }
    else {
        ext_factor = args[2].parse().unwrap();
        num_queries = args[3].parse().unwrap();
        grind_factor = args[4].parse().unwrap();
    }

    return (n, ProofOptions::new(ext_factor, num_queries, grind_factor, default_options.hash_function()));
}

/// Generates a program to compute the `n`-th term of Fibonacci sequence
fn generate_fibonacci_program(n: usize) -> Vec<F128> {

    let mut program = vec![opcodes::BEGIN];

    // TODO: update comment
    // the program is a simple repetition of 3 stack operations:
    // the first operation duplicates the top stack item,
    // the second operation moves the 3rd item to the top of the stack,
    // the last operation pops top 2 stack items, adds them, and pushes
    // the result back onto the stack
    for _ in 0..(n - 1) {
        program.push(opcodes::SWAP);
        program.push(opcodes::DUP2);
        program.push(opcodes::DROP);
        program.push(opcodes::ADD);
    }

    return processor::pad_program(&program);
}

/// Computes the `n`-th term of Fibonacci sequence
fn compute_fibonacci(n: usize) -> u128 {
    let mut n1 = 0;
    let mut n2 = 1;

    for _ in 0..(n - 1) {
        let n3 = F128::add(n1, n2);
        n1 = n2;
        n2 = n3;
    }

    return n2;
}