use std::time::{ Instant };
use distaff::{ field, fft, polys, quartic, parallel, hash, MerkleTree };
extern crate num_cpus;

fn main() {

    let n: usize = 1 << 22;
    //test_parallel_mul(n);
    //test_parallel_fft(n);
    //test_parallel_inv(n);
    test_merkle_tree(n);

    /*
    let n: usize = 1 << 25;
    let r = field::get_root_of_unity(n as u64);
    let xs = field::get_power_series(r, n);
    let polys = field::rand_vector(n * 4);
    let now = Instant::now();
    let ys = quartic::evaluate_batch(&polys, &xs);
    let t = now.elapsed().as_millis();
    println!("Interpolated {} quartic polynomials in {} ms", ys.len() / 4, t);
    */

    /*
    let n: usize = 1 << 21;
    let r = field::get_root_of_unity((n * 4) as u64);
    let xs = field::get_power_series(r, n * 4);
    let mut ys = vec![0u64; n * 4];
    field::rand_fill(&mut ys);
    let now = Instant::now();
    let ps = quartic::interpolate_batch(&xs, &ys);
    let t = now.elapsed().as_millis();
    println!("Interpolated {} quartic polynomials in {} ms", ps.len() / 4, t);
    */

    //fft::permute(&mut p);
    //println!("{:?}", p);
}

fn test_merkle_tree(n: usize) {
    let leaves = field::rand_vector(n);
    let index: usize = 3;
    //println!("leaves: {:?}", leaves);
    //println!("----------");
    let now = Instant::now();
    let tree = MerkleTree::new(to_quartic_vec(leaves), hash::gmimc);
    let t = now.elapsed().as_millis();
    println!("Merkle tree of {} nodes built in {} ms", n / 4, t);
    println!("----------");
    let proof = tree.prove(index);
    //println!("proof: {:?}", proof);
    //println!("----------");
    let result = MerkleTree::verify(tree.root(), index, &proof, hash::gmimc);
    println!("{}", result);
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
        parallel::mul_in_place(&x, &mut z2, num_threads);
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

fn test_fft(n: usize) {
    let p = field::rand_vector(n);
    let g = field::get_root_of_unity(n as u64);
    let twiddles = fft::get_twiddles(g, n);

    let mut v = p.clone();
    let now = Instant::now();
    fft::fft_in_place(&mut v, &twiddles, 1, 1, 0, 1);
    let t = now.elapsed().as_millis();
    println!("computed FFT over {} values in {} ms", p.len(), t);
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
}

// HELPER FUNCTIONS
// ================================================================================================
fn to_quartic_vec(vector: Vec<u64>) -> Vec<[u64; 4]> {
    let mut v = std::mem::ManuallyDrop::new(vector);
    let p = v.as_mut_ptr();
    let len = v.len() / 4;
    let cap = v.capacity() / 4;
    return unsafe { Vec::from_raw_parts(p as *mut [u64; 4], len, cap) };
}