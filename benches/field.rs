use criterion::{ black_box, criterion_group, Criterion };
use distaff::math::{ field, parallel };

pub fn add128(c: &mut Criterion) {
    let x = field::rand();
    let y = field::rand();
    c.bench_function("add128", |bench| {
        bench.iter(|| field::add(black_box(x), black_box(y)))
    });
}

pub fn mul128(c: &mut Criterion) {
    let x = field::rand();
    let y = field::rand();
    c.bench_function("mul128", |bench| {
        bench.iter(|| field::mul(black_box(x), black_box(y)))
    });
}

pub fn mul_parallel(c: &mut Criterion) {

    let n = (1 << 10) as usize;
    let x = field::rand_vector(n);
    let y = field::rand_vector(n);
    let threads = 2;

    c.bench_function("mul (parallel)", |bench| {
        bench.iter(|| parallel::mul(black_box(&x), black_box(&y), black_box(threads)))
    });
}

pub fn mul_parallel_in_place(c: &mut Criterion) {

    let n = (1 << 10) as usize;
    let x = field::rand_vector(n);
    let mut y = field::rand_vector(n);
    let threads = 2;

    c.bench_function("mul (parallel, in place)", |bench| {
        bench.iter(|| parallel::mul_in_place(black_box(&mut y), black_box(&x), black_box(threads)))
    });
}

pub fn exp128(c: &mut Criterion) {
    let x = field::rand();
    let y = field::rand();
    c.bench_function("exp128", |bench| {
        bench.iter(|| field::exp(black_box(x), black_box(y)))
    });
}

pub fn inv128(c: &mut Criterion) {
    let x = field::rand();
    c.bench_function("inv128", |bench| {
        bench.iter(|| field::inv(black_box(x)))
    });
}

criterion_group!(group, add128, mul128, mul_parallel, mul_parallel_in_place, exp128, inv128);