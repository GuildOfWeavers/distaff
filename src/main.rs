use std::time::{ Instant };
use distaff::{ ProofOptions, StarkProof, processor, processor::opcodes };

fn main() {

    let op_count = usize::pow(2, 10) - 1;
    let mut program = Vec::new();
    while op_count > program.len() {
        program.push(opcodes::DUP0);
        program.push(opcodes::PULL2);
        program.push(opcodes::ADD);
    }
    
    let options = ProofOptions::default();
    let inputs = [1, 1];

    let now = Instant::now();
    let (outputs, program_hash, proof) = processor::execute(&program, &inputs, 1, &options);
    println!("----------------------");
    println!("Executed program with hash {} in {} ms", 
        hex::encode(program_hash),
        now.elapsed().as_millis());
    println!("Program output: {:?}", outputs);

    let proof_bytes = bincode::serialize(&proof).unwrap();
    println!("Execution proof size: {} KB", proof_bytes.len() / 1024);
    println!("Execution proof security: {} bits", options.security_level());

    println!("----------------------");
    let proof = bincode::deserialize::<StarkProof>(&proof_bytes).unwrap();
    let now = Instant::now();
    match processor::verify(&program_hash, &inputs, &outputs, &proof) {
        Ok(_) => println!("Execution verified in {} ms", now.elapsed().as_millis()),
        Err(msg) => println!("Failed to verify execution: {}", msg)
    }
}