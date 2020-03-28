use rand::prelude::*;
use rand::distributions::{Distribution, Uniform};

pub const M: u64 = 18446743880436023297; // 2^64 - 45 * 2^32 + 1
pub const G: u64 = 8387321423513296549;  // 2^32 root of unity

// BASIC ARITHMETIC
// ------------------------------------------------------------------------------------------------

/// Computes (a + b) % m; a and b are assumed to be valid field elements.
pub fn add(a: u64, b: u64) -> u64 {
    let mut z = (a as u128) + (b as u128);
    if z >= (M as u128) {
        z = z - (M as u128);
    }
    return z as u64;
}

/// Computes (a - b) % m; a and b are assumed to be valid field elements.
pub fn sub(a: u64, b: u64) -> u64 {
    if a < b { M - b + a } else { a - b }
}

/// Computes (a * b) % m; a and b are assumed to be valid field elements.
pub fn mul(a: u64, b: u64) -> u64 {
    let mut z = (a as u128) * (b as u128);

    // TODO: prove that 3 shifts are enough
    let mut q = (z >> 64) * (M as u128);
    z = z - q;

    q = (z >> 64) * (M as u128);
    z = z - q;

    q = (z >> 64) * (M as u128);
    z = z - q;

    if z >= (M as u128) {
        z = z - (M as u128);
    }
    
    return z as u64;
}

/// Computes y such that (x * y) % m = 1; x is assumed to be a valid field element.
pub fn inv(x: u64) -> u64 {
    if x == 0 { return 0 };

    let mut a: u128 = 0;
    let mut u: u128 = if x & 1 == 1 { x as u128 } else { (x as u128) + (M as u128) };
    let mut v: u128 = M as u128;
    let mut d = (M as u128) - 1;

    while v != 1 {
        while v < u {
            u = u - v;
            d = d + a;
            while u & 1 == 0 {
                if d & 1 == 1 {
                    d = d + (M as u128);
                }
                u = u >> 1;
                d = d >> 1;
            }
        }

        v = v - u;
        a = a + d;

        while v & 1 == 0 {
            if a & 1 == 1 {
                a = a + (M as u128);
            }
            v = v >> 1;
            a = a >> 1;
        }
    }

    while a > (M as u128) {
        a = a - (M as u128);
    }

    return a as u64;
}

pub fn inv_many(v: &[u64]) -> Vec<u64> {
    let mut result = Vec::with_capacity(v.len());
    unsafe { result.set_len(v.len()); }

    let mut last = 1u64;
    for i in 0..v.len() {
        result[i] = last;
        if v[i] != 0 {
            last = mul(last, v[i]);
        }
    }

    last = inv(last);
    for i in (0..v.len()).rev() {
        if v[i] == 0 {
            result[i] = 0;
        }
        else {
            result[i] = mul(last, result[i]);
            last = mul(last, v[i]);
        }
    }

    return result;
}

/// Computes y = (a / b) such that b * y = a; a and b are assumed to be valid field elements.
pub fn div(a: u64, b: u64) -> u64 {
    let b = inv(b);
    return mul(a, b);
}

/// Computes (b^p) % m; b and p are assumed to be valid field elements.
pub fn exp(b: u64, p: u64) -> u64 {
    if b == 0 { return 0; }
    else if p == 0 { return 1; }

    let mut r = 1;
    let mut b = b;
    let mut p = p;

    // TODO: optimize
    while p > 0 {
        if p & 1 == 1 {
            r = mul(r, b);
        }
        p = p >> 1;
        b = mul(b, b);
    }

    return r;
}

/// Computes (0 - x) % m; x is assumed to be a valid field element.
pub fn neg(x: u64) -> u64 {
    return sub(0, x);
}

// ROOT OF UNITY
// ------------------------------------------------------------------------------------------------
pub fn get_root_of_unity(order: u64) -> u64 {
    // TODO: add error handling is_power_of_two
    let p = 1 << (32 - order.trailing_zeros());
    return exp(G, p as u64);
}

pub fn fill_power_series(base: u64, dest: &mut [u64]) {
    let mut p = 1u64;
    for i in 0..dest.len() {
        dest[i] = p;
        p = mul(p, base);
    }
}

pub fn get_power_series(base: u64, length: usize) -> Vec<u64> {
    return (0..length).map(|i| exp(base, i as u64)).collect::<Vec<u64>>();
}

// RANDOMNESS
// ------------------------------------------------------------------------------------------------
pub fn rand() -> u64 {
    let range = Uniform::from(0..M);
    let mut g = thread_rng();
    return range.sample(&mut g);
}

pub fn rand_fill(dest: &mut [u64]) {
    let mut g = thread_rng();
    g.fill(dest);
}

pub fn prng(seed: [u8; 32]) -> u64 {
    let range = Uniform::from(0..M);
    let mut g = StdRng::from_seed(seed);
    return range.sample(&mut g);
}

pub fn prng_fill(seed: [u8; 32], dest: &mut [u64]) {
    // TODO: implement
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    
    #[test]
    fn add() {
        // identity
        let r = super::rand();
        assert_eq!(r, super::add(r, 0));

        // test addition within bounds
        assert_eq!(5, super::add(2, 3));

        // test overflow
        let t = super::M - 1;
        assert_eq!(0, super::add(t, 1));
        assert_eq!(1, super::add(t, 2));

        // test random values
        let r1 = super::rand();
        let r2 = super::rand();
        assert_eq!(test_add(r1, r2), super::add(r1, r2));
    }

    #[test]
    fn sub() {
        // identity
        let r = super::rand();
        assert_eq!(r, super::sub(r, 0));

        // test subtraction within bounds
        assert_eq!(2, super::sub(5, 3));

        // test underflow
        assert_eq!(super::M - 2, super::sub(3, 5));
    }

    #[test]
    fn mul() {
        // identity
        let r = super::rand();
        assert_eq!(0, super::mul(r, 0));
        assert_eq!(r, super::mul(r, 1));

        // test multiplication within bounds
        assert_eq!(15, super::mul(5, 3));

        // test overflow
        let t = super::M - 1;
        assert_eq!(1, super::mul(t, t));
        assert_eq!(super::M - 2, super::mul(t, 2));
        assert_eq!(super::M - 4, super::mul(t, 4));

        let t = (super::M + 1) / 2;
        assert_eq!(1, super::mul(t, 2));

        // test random values
        let r1 = super::rand();
        let r2 = super::rand();
        assert_eq!(test_mul(r1, r2), super::mul(r1, r2));
    }

    #[test]
    fn inv() {
        // identity
        assert_eq!(1, super::inv(1));
        assert_eq!(0, super::inv(0));

        // test random values
        let x = super::rand();
        let y = super::inv(x);
        assert_eq!(1, super::mul(x, y));
    }

    #[test]
    fn exp() {
        // identity
        let r = super::rand();
        assert_eq!(1, super::exp(r, 0));
        assert_eq!(r, super::exp(r, 1));
        assert_eq!(0, super::exp(0, r));

        // test exponentiation within bounds
        assert_eq!(125, super::exp(5, 3));

        // test overflow
        let t = super::M - 1;
        assert_eq!(test_mul(t, t), super::exp(t, 2));
        assert_eq!(test_mul(test_mul(t, t), t), super::exp(t, 3));
    }

    // controller methods
    fn test_add(a: u64, b: u64) -> u64 {
        ((a as u128 + b as u128) % (super::M as u128)) as u64
    }

    fn test_mul(a: u64, b: u64) -> u64 {
        ((a as u128 * b as u128) % (super::M as u128)) as u64
    }
}