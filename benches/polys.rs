use criterion::{ black_box, criterion_group, Criterion };
use distaff::{ math, polys, quartic };

pub fn eval(c: &mut Criterion) {
    let mut p = vec![0u64; 1024];
    math::rand_fill(&mut p);
    let x = math::rand();
    c.bench_function("Poly eval", |bench| {
        bench.iter(|| polys::eval(black_box(&p), black_box(x)))
    });
}

pub fn eval4(c: &mut Criterion) {
    let mut p = vec![0u64; 4];
    math::rand_fill(&mut p);
    let x = math::rand();
    c.bench_function("Poly eval (quartic)", |bench| {
        bench.iter(|| quartic::eval(black_box(&p), black_box(x)))
    });
}

criterion_group!(group, eval, eval4);