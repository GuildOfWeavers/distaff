use criterion::criterion_main;

mod field;
mod hash;
mod fft;
mod polynom;

criterion_main!(field::group, hash::group, fft::group, polynom::group);