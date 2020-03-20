use criterion::criterion_main;

mod math;
mod hash;

criterion_main!(math::group, hash::group);