use criterion::{ black_box, criterion_group, Criterion };
use distaff::{ F64, F128, FiniteField, parallel };

pub fn add64(c: &mut Criterion) {
    let x: u64 = 20;
    let y: u64 = 20;
    c.bench_function("add64", |bench| {
        bench.iter(|| F64::add(black_box(x), black_box(y)))
    });
}

pub fn add128(c: &mut Criterion) {
    let x: u128 = 20;
    let y: u128 = 20;
    c.bench_function("add128", |bench| {
        bench.iter(|| F128::add(black_box(x), black_box(y)))
    });
}

pub fn mul64(c: &mut Criterion) {
    let x: u64 = 20;
    let y: u64 = 20;
    c.bench_function("mul64", |bench| {
        bench.iter(|| F64::mul(black_box(x), black_box(y)))
    });
}

pub fn mul128(c: &mut Criterion) {
    let x: u128 = 20;
    let y: u128 = 20;
    c.bench_function("mul128", |bench| {
        bench.iter(|| F128::mul(black_box(x), black_box(y)))
    });
}

pub fn mul_parallel(c: &mut Criterion) {

    let n = (1 << 10) as usize;
    let x = F64::rand_vector(n);
    let y = F64::rand_vector(n);
    let threads = 2;

    c.bench_function("mul (parallel)", |bench| {
        bench.iter(|| parallel::mul(black_box(&x), black_box(&y), black_box(threads)))
    });
}

pub fn mul_parallel_in_place(c: &mut Criterion) {

    let n = (1 << 10) as usize;
    let x = F64::rand_vector(n);
    let mut y = F64::rand_vector(n);
    let threads = 2;

    c.bench_function("mul (parallel, in place)", |bench| {
        bench.iter(|| parallel::mul_in_place(black_box(&mut y), black_box(&x), black_box(threads)))
    });
}

pub fn exp(c: &mut Criterion) {
    let x: u64 = 20;
    let y: u64 = 20;
    c.bench_function("exp", |bench| {
        bench.iter(|| F64::exp(black_box(x), black_box(y)))
    });
}

pub fn inv(c: &mut Criterion) {
    let x: u64 = 20;
    c.bench_function("inv", |bench| {
        bench.iter(|| F64::inv(black_box(x)))
    });
}

criterion_group!(group, add64, add128, mul64, mul128, mul_parallel, mul_parallel_in_place, exp, inv);