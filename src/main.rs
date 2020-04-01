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
    /*
    let leaves = vec![
        6240958401110583462u64,  7913251457734141410, 10424272014552449446,  8926189284258310218,
          16554193988646091251, 18107256576288978408,  9223357806195242659,  7591105067405469359,
          11143668108497789195,  3289331174328174429, 18085733244798495096, 16874288619384630339,
          13458213771757530415, 15574026171644776407,  2236303685881236230, 16652047415881651529
    ];
    let leaves = vec![
        10241768711231905139u64, 9543515656056738355, 3787122002184510141, 9354315911492805116,
           14373792471285313076, 10259803863341799909, 4361913119464376502, 14664313136545201958,
           10131098303839284098, 5921316728206729490, 10334290713044556732, 8643164606753777491,
            3453858615599341263, 17558389957719367849, 9827054574735249697, 8012452355193068045,
            9196785718850699443, 6184806869699853092, 1586592971438511472, 555830527090219830,
            9952908082911899749, 3740909091289176615, 284496432800007785, 12636108119248205469,
           15468185072990248985, 9202716477534013353, 15320321401254534633, 9244660312647244009,
           13492130182068317175, 11411250703184174957, 5614217056664461616, 12322142689514354888
    ];
    */
    let index: usize = 3;
    //println!("leaves: {:?}", leaves);
    //println!("----------");
    let now = Instant::now();
    let tree = MerkleTree::new(leaves, hash::gmimc);
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