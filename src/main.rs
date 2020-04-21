use std::env;
use std::io::Write;
use std::time::Instant;
use distaff::{ ProofOptions, StarkProof, processor, processor::opcodes, field, hash_acc };

fn main() {

    // configure logging
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter_level(log::LevelFilter::Debug).init();

    // read the length of Fibonacci sequence from command line or use 6 as default
    let args: Vec<String> = env::args().collect();
    let n: usize = if args.len() > 1 { args[1].parse().unwrap() } else { 6 };
    
    // generate the program and expected results
    let program = generate_fibonacci_program(n);
    let expected_result = compute_fibonacci(n);
    let expected_hash = hash_acc(&program[..(program.len() - 1)]);
    let expected_hash = unsafe { *(&expected_hash as *const _ as *const [u8; 32]) };
    println!("Generated a program to compute {}-th Fibonacci term; expected result: {}", 
        n,
        expected_result);
    println!("--------------------------------");

    let options = ProofOptions::default();
    let inputs = [1, 0];    // initialize stack with 2 values; 1 will be at the top
    let num_outputs = 1;    // a single element from the top of the stack will be the output

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
    println!("Execution proof security: {} bits", options.security_level());
    println!("--------------------------------");

    // verify that executing a program with a given hash and given inputs
    // results in the expected output
    let proof = bincode::deserialize::<StarkProof>(&proof_bytes).unwrap();
    let now = Instant::now();
    match processor::verify(&program_hash, &inputs, &outputs, &proof) {
        Ok(_) => println!("Execution verified in {} ms", now.elapsed().as_millis()),
        Err(msg) => println!("Failed to verify execution: {}", msg)
    }
}

/// Generates a program to compute the `n`-th term of Fibonacci sequence
fn generate_fibonacci_program(n: usize) -> Vec<u64> {
    let mut program = Vec::new();

    // the program is a simple repetition of 3 stack operations:
    // the first operation duplicates the top stack item,
    // the second operation moves the 3rd item to the top of the stack,
    // the last operation pops top 2 stack items, adds them, and pushes
    // the result back onto the stack
    for _ in 0..(n - 1) {
        program.push(opcodes::DUP0);
        program.push(opcodes::PULL2);
        program.push(opcodes::ADD);
    }

    return processor::pad_program(&program);
}

/// Computes the `n`-th term of Fibonacci sequence
fn compute_fibonacci(n: usize) -> u64 {
    let mut n1 = 0;
    let mut n2 = 1;

    for _ in 0..(n - 1) {
        let n3 = field::add(n1, n2);
        n1 = n2;
        n2 = n3;
    }

    return n2;
}