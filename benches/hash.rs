use criterion::{ black_box, criterion_group, Criterion };
use distaff::hash;

pub fn poseidon(c: &mut Criterion) {
    let v = [1, 2, 3, 4, 5, 6, 7, 8];
    let mut r = [0u64; 4];
    c.bench_function("poseidon", |bench| {
        bench.iter(|| hash::poseidon(black_box(&v), black_box(&mut r)))
    });
}

pub fn rescue(c: &mut Criterion) {
    let v = [1, 2, 3, 4, 5, 6, 7, 8];
    let mut r = [0u64; 4];
    c.bench_function("rescue", |bench| {
        bench.iter(|| hash::rescue(black_box(&v), black_box(&mut r)))
    });
}

criterion_group!(group, poseidon, rescue);