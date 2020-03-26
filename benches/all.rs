use criterion::criterion_main;

mod math;
mod hash;
mod fft;
mod polys;

criterion_main!(math::group, hash::group, fft::group, polys::group);