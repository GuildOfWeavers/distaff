use std::time::{ Instant };
use distaff::crypto::{ MerkleTree, hash };
use distaff::{ field, fft, polys, quartic, parallel, ProofOptions, StarkProof };
use distaff::processor::{ self, opcodes };
use rand::prelude::*;
use rand::distributions::Uniform;
extern crate num_cpus;

fn main() {

    let n: usize = 1 << 20;
    execute_program();
    //test_merkle_tree(n);
    //test_parallel_fft(n);
    //test_poly_eval_fft(n);
    //test_parallel_mul(n);
    //test_parallel_inv(n);
    //test_quartic_batch(n);
    //test_hash_functions(n);
}

fn execute_program() {

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

    println!("----------------------");
    let proof = bincode::deserialize::<StarkProof>(&proof_bytes).unwrap();
    let now = Instant::now();
    match processor::verify(&program_hash, &inputs, &outputs, &proof) {
        Ok(_) => println!("Execution proof verified in {} ms", now.elapsed().as_millis()),
        Err(msg) => println!("Failed to verify execution proof: {}", msg)
    }

}

fn test_merkle_tree(n: usize) {
    let leaves = quartic::to_quartic_vec(field::rand_vector(n * 4));
    let indexes = rand_vector(48, n);
    let now = Instant::now();
    let tree = MerkleTree::new(leaves, hash::blake3);
    let t = now.elapsed().as_millis();
    println!("Merkle tree of {} nodes built in {} ms", n, t);
    
    let now = Instant::now();
    let proof = tree.prove_batch(&indexes);
    let t = now.elapsed().as_millis();
    println!("Generated proof for {} indexes in {} ms", indexes.len(), t);

    let now = Instant::now();
    let result = MerkleTree::verify_batch(tree.root(), &indexes, &proof, hash::blake3);
    let t = now.elapsed().as_millis();
    println!("Verified proof for {} indexes in {} ms", indexes.len(), t);
    println!("{}", result);
}

fn test_parallel_fft(n: usize) {
    let p = field::rand_vector(n);
    let g = field::get_root_of_unity(n as u64);
    let twiddles = fft::get_twiddles(g, n);

    let mut v1 = p.clone();
    let now = Instant::now();
    fft::fft_in_place(&mut v1, &twiddles, 1, 1, 0, 1);
    let t = now.elapsed().as_millis();
    println!("computed FFT over {} values in {} ms", p.len(), t);

    for i in 0..4 {
        let num_threads = usize::pow(2, i);
        let mut v2 = p.clone();
        let now = Instant::now();
        fft::fft_in_place(&mut v2, &twiddles, 1, 1, 0, num_threads);
        let t = now.elapsed().as_millis();
        println!("computed FFT over {} values using {} threads in {} ms", p.len(), num_threads, t);
        for i in 0..n { assert_eq!(v1[i], v2[i]); }
    }
}

fn test_poly_eval_fft(n: usize) {

    let mut p = field::rand_vector(n);
    let g = field::get_root_of_unity(n as u64);
    let twiddles = fft::get_twiddles(g, n);

    let now = Instant::now();
    polys::eval_fft_twiddles(&mut p, &twiddles, true);
    let t = now.elapsed().as_millis();
    println!("evaluated degree {} polynomial in {} ms", p.len(), t);
}

fn test_parallel_mul(n: usize) {
            
    let x = field::rand_vector(n);
    let y = field::rand_vector(n);
    let mut z1 = vec![0u64; n];

    let now = Instant::now();
    for i in 0..n {
        z1[i] = field::mul(x[i], y[i]);
    }
    let t = now.elapsed().as_millis();
    println!("Multiplied {} values in {} ms", z1.len(), t);
    
    for i in 0..4 {
        let num_threads = usize::pow(2, i);
        let now = Instant::now();
        let z2 = parallel::mul(&x, &y, num_threads);
        let t = now.elapsed().as_millis();
        println!("Multiplied {} values using {} threads in {} ms", z2.len(), num_threads, t);
        for i in 0..n { assert_eq!(z1[i], z2[i]); }
    }

    for i in 0..4 {
        let num_threads = usize::pow(2, i);
        let mut z2 = y.clone();
        let now = Instant::now();
        parallel::mul_in_place(&mut z2, &x, num_threads);
        let t = now.elapsed().as_millis();
        println!("Multiplied {} values in place using {} threads in {} ms", z2.len(), num_threads, t);
        for i in 0..n { assert_eq!(z1[i], z2[i]); }
    }
}

fn test_parallel_inv(n: usize) {
    let num_threads = num_cpus::get_physical();
    println!("cores: {}, threads: {}", num_cpus::get_physical(), num_threads);

    let v = field::rand_vector(n);
    let now = Instant::now();
    let z1 = field::inv_many(&v);
    let t = now.elapsed().as_millis();
    println!("Inverted {} values using one thread in {} ms", z1.len(), t);

    let now = Instant::now();
    let z2 = parallel::inv(&v, num_threads);
    let t = now.elapsed().as_millis();
    println!("Inverted {} values using {} threads in {} ms", z2.len(), num_threads, t);

    for i in 0..n {
        assert_eq!(z1[i], z2[i]);
    }
}

fn test_quartic_batch(n: usize) {

    let x = field::rand();
    let polys = quartic::to_quartic_vec(field::rand_vector(n * 4));
    let now = Instant::now();
    let ys = quartic::evaluate_batch(&polys, x);
    let t = now.elapsed().as_millis();
    println!("Evaluated {} quartic polynomials in {} ms", ys.len(), t);

    let r = field::get_root_of_unity((n * 4) as u64);
    let xs = quartic::to_quartic_vec(field::get_power_series(r, n * 4));
    let ys = quartic::to_quartic_vec(field::rand_vector(n * 4));
    let now = Instant::now();
    let polys = quartic::interpolate_batch(&xs, &ys);
    let t = now.elapsed().as_millis();
    println!("Interpolated {} quartic polynomials in {} ms", polys.len(), t);
}

fn test_hash_functions(n: usize) {
    let values = vec![42u64; 8 * n];
    let mut result = vec![0u64; 4 * n];

    let now = Instant::now();
    for i in 0..n {
        hash::poseidon(&values[(i * 8)..(i * 8 + 8)], &mut result[(i * 4)..(i * 4 + 4)]);
    }
    let t = now.elapsed().as_millis();
    println!("completed {} poseidon hashes in: {} ms", n, t);

    let now = Instant::now();
    for i in 0..n {
        hash::rescue(&values[(i * 8)..(i * 8 + 8)], &mut result[(i * 4)..(i * 4 + 4)]);
    }
    let t = now.elapsed().as_millis();
    println!("completed {} rescue hashes in: {} ms", n, t);

    let now = Instant::now();
    for i in 0..n {
        hash::gmimc(&values[(i * 8)..(i * 8 + 8)], &mut result[(i * 4)..(i * 4 + 4)]);
    }
    let t = now.elapsed().as_millis();
    println!("completed {} GMiMC hashes in: {} ms", n, t);

    let now = Instant::now();
    for i in 0..n {
        hash::blake3(&values[(i * 8)..(i * 8 + 8)], &mut result[(i * 4)..(i * 4 + 4)]);
    }
    let t = now.elapsed().as_millis();
    println!("completed {} Blake3 hashes in: {} ms", n, t);
}

// HELPER FUNCTIONS
// ================================================================================================
pub fn rand_vector(count: usize, max: usize) -> Vec<usize> {
    let range = Uniform::from(0..max);
    let g = thread_rng();
    return g.sample_iter(range).take(count).collect();
}