use criterion::criterion_main;

mod math;
mod hash;
mod fft;

criterion_main!(math::group, hash::group, fft::group);