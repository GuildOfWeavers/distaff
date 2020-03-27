use criterion::{ black_box, criterion_group, Criterion };
use distaff::{ field, polys, quartic };

pub fn eval(c: &mut Criterion) {
    let mut p = vec![0u64; 1024];
    field::rand_fill(&mut p);
    let x = field::rand();
    c.bench_function("Poly eval", |bench| {
        bench.iter(|| polys::eval(black_box(&p), black_box(x)))
    });
}

pub fn eval4(c: &mut Criterion) {
    let mut p = vec![0u64; 4];
    field::rand_fill(&mut p);
    let x = field::rand();
    c.bench_function("Poly eval (quartic)", |bench| {
        bench.iter(|| quartic::eval(black_box(&p), black_box(x)))
    });
}

pub fn interpolate4_batch(c: &mut Criterion) {
    let n: usize = 1 << 10;
    let r = field::get_root_of_unity((n * 4) as u64);
    let xs = field::get_power_series(r, n * 4);
    let mut ys = vec![0u64; n * 4];
    field::rand_fill(&mut ys);
    c.bench_function("Poly interpolation (quartic batch)", |bench| {
        bench.iter(|| quartic::interpolate_batch(black_box(&xs), black_box(&ys)))
    });
    
    /*
    c.bench_function("Poly interpolation (quartic batch)", |bench| {
        bench.iter(|| {
            for i in (0..n).step_by(4) {
                polys::interpolate(black_box(&xs[i..(i+4)]), black_box(&ys[i..(i+4)]));
            }
        })
    });
    */
}

criterion_group!(group, eval, eval4, interpolate4_batch);