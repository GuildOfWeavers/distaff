use std::time::{Instant};
use distaff::{ hash };

const N: usize = 10_000;

fn main() {

    let values = vec![42u64; 8 * N];
    let mut result = vec![0u64; 4 * N];

    let now = Instant::now();
    for i in 0..N {
        hash::poseidon(&values[(i * 8)..(i * 8 + 8)], &mut result[(i * 4)..(i * 4 + 4)]);
    }
    let t = now.elapsed().as_millis();
    println!("completed {} poseidon hashes in: {} ms", N, t);

    let now = Instant::now();
    for i in 0..N {
        hash::rescue(&values[(i * 8)..(i * 8 + 8)], &mut result[(i * 4)..(i * 4 + 4)]);
    }
    let t = now.elapsed().as_millis();
    println!("completed {} rescue hashes in: {} ms", N, t);

    let now = Instant::now();
    for i in 0..N {
        hash::gmimc(&values[(i * 8)..(i * 8 + 8)], &mut result[(i * 4)..(i * 4 + 4)]);
    }
    let t = now.elapsed().as_millis();
    println!("completed {} GMiMC hashes in: {} ms", N, t);

}
