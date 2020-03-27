use criterion::criterion_main;

mod field;
mod hash;
mod fft;
mod polys;

criterion_main!(field::group, hash::group, fft::group, polys::group);