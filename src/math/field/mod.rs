use std::ops::Range;
use std::fmt::{ Debug, Display };
use rand::prelude::*;
use rand::distributions::{ Distribution, Uniform, uniform::SampleUniform };
use crate::utils::{ uninit_vector };

// RE-EXPORTS
// ================================================================================================
pub mod prime64;
pub mod prime128;

// TYPES AND INTERFACES
// ================================================================================================
pub trait FieldElement: Copy + Eq + PartialOrd + Display + Debug + SampleUniform + Send + Sync {
    fn from(value: usize) -> Self;
}

pub trait FiniteField<T> 
    where T: Copy + Eq + PartialOrd + Display + Debug + SampleUniform + Send + Sync
{
    const MODULUS: T;
    const RANGE: Range<T>;

    const ZERO: T;
    const ONE: T;

    // BASIC ARITHMETIC
    // --------------------------------------------------------------------------------------------

    /// Computes (a + b) % m; a and b are assumed to be valid field elements.
    fn add(a: T, b: T) -> T;

    /// Computes (a - b) % m; a and b are assumed to be valid field elements.
    fn sub(a: T, b: T) -> T;

    /// Computes (a * b) % m; a and b are assumed to be valid field elements.
    fn mul(a: T, b: T) -> T;

    /// Computes a[i] + b[i] * c for all i and saves result into a.
    fn mul_acc(a: &mut [T], b: &[T], c: T) {
        for i in 0..a.len() {
            a[i] = Self::add(a[i], Self::mul(b[i], c));
        }
    }

    /// Computes y such that (x * y) % m = 1; x is assumed to be a valid field element.
    fn inv(x: T) -> T;

    /// Computes multiplicative inverses of all slice elements using batch inversion method.
    fn inv_many(values: &[T]) -> Vec<T> {
        let mut result = uninit_vector(values.len());
        Self::inv_many_fill(values, &mut result);
        return result;
    }

    /// Computes multiplicative inverses of all slice elements using batch inversion method
    /// and stores the result into the provided slice.
    fn inv_many_fill(values: &[T], result: &mut [T]) {
        let mut last = Self::ONE;
        for i in 0..values.len() {
            result[i] = last;
            if values[i] != Self::ZERO {
                last = Self::mul(last, values[i]);
            }
        }

        last = Self::inv(last);
        for i in (0..values.len()).rev() {
            if values[i] == Self::ZERO {
                result[i] = Self::ZERO;
            }
            else {
                result[i] = Self::mul(last, result[i]);
                last = Self::mul(last, values[i]);
            }
        }
    }

    /// Computes y = (a / b) such that (b * y) % m = a; a and b are assumed to be valid field elements.
    fn div(a: T, b: T) -> T {
        let b = Self::inv(b);
        return Self::mul(a, b);
    }

    /// Computes (b^p) % m; b and p are assumed to be valid field elements.
    fn exp(b: T, p: T) -> T;

    /// Computes (0 - x) % m; x is assumed to be a valid field element.
    fn neg(x: T) -> T {
        return Self::sub(Self::ZERO, x);
    }

    // ROOT OF UNITY
    // --------------------------------------------------------------------------------------------

    /// Computes primitive root of unity for the specified `order`.
    fn get_root_of_unity(order: usize) -> T;

    /// Generates a vector with values [1, b, b^2, b^3, b^4, ..., b^length].
    fn get_power_series(b: T, length: usize) -> Vec<T> {
        let mut result = uninit_vector(length);
        result[0] = Self::ONE;
        for i in 1..result.len() {
            result[i] = Self::mul(result[i - 1], b);
        }    
        return result;
    }

    // RANDOMNESS
    // --------------------------------------------------------------------------------------------

    /// Generates a random field element.
    fn rand() -> T {
        let range = Uniform::from(Self::RANGE);
        let mut g = thread_rng();
        return g.sample(range);
    }

    /// Generates a vector of random field elements.
    fn rand_vector(length: usize) -> Vec<T> {
        let range = Uniform::from(Self::RANGE);
        let g = thread_rng();
        return g.sample_iter(range).take(length).collect();
    }

    /// Generates a pseudo-random field element from a given `seed`.
    fn prng(seed: [u8; 32]) -> T {
        let range = Uniform::from(Self::RANGE);
        let mut g = StdRng::from_seed(seed);
        return range.sample(&mut g);
    }

    /// Generates a vector of pseudo-random field elements from a given `seed`.
    fn prng_vector(seed: [u8; 32], length: usize) -> Vec<T> {
        let range = Uniform::from(Self::RANGE);
        let g = StdRng::from_seed(seed);
        return g.sample_iter(range).take(length).collect();
    }
}