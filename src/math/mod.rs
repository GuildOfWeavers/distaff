mod field;
pub use field::{ FiniteField, FieldElement, prime64::F64, prime128::F128 };

pub mod fft;
pub mod polynom;
pub mod quartic;
pub mod parallel;