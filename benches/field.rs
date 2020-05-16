use criterion::{ black_box, criterion_group, Criterion };
use distaff::{ Field, FiniteField, parallel };

pub fn add64(c: &mut Criterion) {
    let x: u64 = 20;
    let y: u64 = 20;
    c.bench_function("add64", |bench| {
        bench.iter(|| Field::add(black_box(x), black_box(y)))
    });
}

pub fn add128(c: &mut Criterion) {
    let x: u128 = 20;
    let y: u128 = 20;
    c.bench_function("add128", |bench| {
        bench.iter(|| Field::add(black_box(x), black_box(y)))
    });
}

pub fn mul(c: &mut Criterion) {
    let x: u64 = 20;
    let y: u64 = 20;
    c.bench_function("mul", |bench| {
        bench.iter(|| Field::mul(black_box(x), black_box(y)))
    });
}

pub fn mul_parallel(c: &mut Criterion) {

    let n = (1 << 10) as usize;
    let x = Field::rand_vector(n);
    let y = Field::rand_vector(n);
    let threads = 2;

    c.bench_function("mul (parallel)", |bench| {
        bench.iter(|| parallel::mul(black_box(&x), black_box(&y), black_box(threads)))
    });
}

pub fn mul_parallel_in_place(c: &mut Criterion) {

    let n = (1 << 10) as usize;
    let x = Field::rand_vector(n);
    let mut y = Field::rand_vector(n);
    let threads = 2;

    c.bench_function("mul (parallel, in place)", |bench| {
        bench.iter(|| parallel::mul_in_place(black_box(&mut y), black_box(&x), black_box(threads)))
    });
}

pub fn exp(c: &mut Criterion) {
    let x: u64 = 20;
    let y: u64 = 20;
    c.bench_function("exp", |bench| {
        bench.iter(|| Field::exp(black_box(x), black_box(y)))
    });
}

pub fn inv(c: &mut Criterion) {
    let x: u64 = 20;
    c.bench_function("inv", |bench| {
        bench.iter(|| Field::inv(black_box(x)))
    });
}

criterion_group!(group, add64, add128, mul, mul_parallel, mul_parallel_in_place, exp, inv);