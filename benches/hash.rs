use criterion::{ black_box, criterion_group, Criterion };
use distaff::hash;

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

pub fn blake3(c: &mut Criterion) {
    let v = [1, 2, 3, 4, 5, 6, 7, 8];
    let mut r = [0u64; 4];
    c.bench_function("Blake3", |bench| {
        bench.iter(|| hash::blake3(black_box(&v), black_box(&mut r)))
    });
}

pub fn sha3(c: &mut Criterion) {
    let v = [1, 2, 3, 4, 5, 6, 7, 8];
    let mut r = [0u64; 4];
    c.bench_function("Sha3", |bench| {
        bench.iter(|| hash::sha3(black_box(&v), black_box(&mut r)))
    });
}

criterion_group!(group, poseidon, rescue, gmimc, blake3, sha3);
