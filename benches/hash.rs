use criterion::{ black_box, criterion_group, Criterion };
use distaff::{ hash, blake2s };

pub fn poseidon(c: &mut Criterion) {
    let v = [1, 2, 3, 4, 5, 6, 7, 8];
    let mut r = [0u64; 4];
    c.bench_function("Poseidon", |bench| {
        bench.iter(|| hash::poseidon(black_box(&v), black_box(&mut r)))
    });
}

pub fn rescue(c: &mut Criterion) {
    let v = [1, 2, 3, 4, 5, 6, 7, 8];
    let mut r = [0u64; 4];
    c.bench_function("Rescue", |bench| {
        bench.iter(|| hash::rescue(black_box(&v), black_box(&mut r)))
    });
}

pub fn gmimc(c: &mut Criterion) {
    let v = [1, 2, 3, 4, 5, 6, 7, 8];
    let mut r = [0u64; 4];
    c.bench_function("GMiMC", |bench| {
        bench.iter(|| hash::gmimc(black_box(&v), black_box(&mut r)))
    });
}

pub fn blake2s(c: &mut Criterion) {
    let v = [1, 2, 3, 4, 5, 6, 7, 8];
    let mut r = [0u64; 4];
    c.bench_function("Blake2s", |bench| {
        bench.iter(|| blake2s::hash(black_box(&v), black_box(&mut r)))
    });
}

pub fn blake2s_fixed(c: &mut Criterion) {
    let v = [1, 2, 3, 4, 5, 6, 7, 8];
    let mut r = [0u64; 4];
    c.bench_function("Blake2s (fixed)", |bench| {
        bench.iter(|| blake2s::hash_fixed(black_box(&v), black_box(&mut r)))
    });
}

criterion_group!(group, poseidon, rescue, gmimc, blake2s, blake2s_fixed);