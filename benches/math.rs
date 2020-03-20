use criterion::{ black_box, criterion_group, Criterion };
use distaff::math;

pub fn add(c: &mut Criterion) {
    let x = 20;
    let y = 20;
    c.bench_function("add", |bench| {
        bench.iter(|| math::add(black_box(x), black_box(y)))
    });
}

pub fn mul(c: &mut Criterion) {
    let x = 20;
    let y = 20;
    c.bench_function("mul", |bench| {
        bench.iter(|| math::mul(black_box(x), black_box(y)))
    });
}

pub fn exp(c: &mut Criterion) {
    let x = 20;
    let y = 20;
    c.bench_function("exp", |bench| {
        bench.iter(|| math::exp(black_box(x), black_box(y)))
    });
}

pub fn inv(c: &mut Criterion) {
    let x = 20;
    c.bench_function("inv", |bench| {
        bench.iter(|| math::inv(black_box(x)))
    });
}

criterion_group!(group, add, mul, exp, inv);