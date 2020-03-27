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
    let r = field::get_root_of_unity(1024);
    let xs = field::get_power_series(r, 1024);
    let mut ys = vec![0u64; 1024];
    field::rand_fill(&mut ys);
    c.bench_function("Poly interpolation (quartic batch)", |bench| {
        bench.iter(|| quartic::interpolate_batch(black_box(&xs), black_box(&ys)))
    });
}

criterion_group!(group, eval, eval4, interpolate4_batch);