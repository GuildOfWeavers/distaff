use std::time::{ Instant };
use distaff::{ field, fft, polys, quartic };

const N: usize = 10_000;

fn main() {

    let n: usize = 1 << 25;
    let r = field::get_root_of_unity(n as u64);
    let xs = field::get_power_series(r, n);
    let polys = field::rand_vector(n * 4);
    let now = Instant::now();
    let ys = quartic::evaluate_batch(&polys, &xs);
    let t = now.elapsed().as_millis();
    println!("Interpolated {} quartic polynomials in {} ms", ys.len() / 4, t);

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

    /*
    let ds: usize = 1 << 22;
    let mut p = vec![0u64; ds];
    math::rand_fill(&mut p);
    let g = math::get_root_of_unity(ds as u64);
    let twiddles = fft::get_twiddles(g, ds);

    let now = Instant::now();
    polys::eval_fft_twiddles(&mut p, &twiddles, true);
    let t = now.elapsed().as_millis();
    println!("evaluated degree {} polynomial in {} ms", p.len(), t);
    */

    /*
    let p = [1u64, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let r = field::get_root_of_unity(16);
    let twiddles = fft::get_twiddles(r, 16);
    let mut v = p.clone();
    polys::eval_fft_twiddles(&mut v, &twiddles, true);
    println!("{:?}", v);

    let mut p2 = v.clone();
    polys::interpolate_fft(&mut p2, true);
    println!("{:?}", p2);

    let roots = field::get_power_series(r, 16);
    let p3 = polys::interpolate(&roots, &v);
    println!("{:?}", p3);
    */

    //fft::permute(&mut p);
    //println!("{:?}", p);
    

    /*
    let p = [384863712573444386u64, 7682273369345308472, 13294661765012277990];
    let x = 11269864713250585702u64;
    let y = polys::eval(&p, x);
    println!("{}", y);
    */

    /*
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
    */
}

fn test() -> Vec<i32> {
    let result = vec![1, 2, 3];
    return result;
}