use std::ops::Range;
use super::{ FiniteField, Field };

// CONSTANTS
// ================================================================================================

// Field modulus = 2^64 - 45 * 2^32 + 1
pub const M: u64 = 18446743880436023297;

// 2^32 root of unity
pub const G: u64 = 8387321423513296549;

// 64-BIT FIELD IMPLEMENTATION
// ================================================================================================
impl FiniteField<u64> for Field {

    const MODULUS: u64 = M;
    const RANGE: Range<u64> = Range { start: 0, end: M };

    const ZERO: u64 = 0;
    const ONE: u64 = 1;
    
    // BASIC ARITHMETIC
    // --------------------------------------------------------------------------------------------
    fn add(a: u64, b: u64) -> u64 {
        let mut z = (a as u128) + (b as u128);
        if z >= (M as u128) {
            z = z - (M as u128);
        }
        return z as u64;
    }

    fn sub(a: u64, b: u64) -> u64 {
        if a < b { M - b + a } else { a - b }
    }

    fn mul(a: u64, b: u64) -> u64 {
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

    fn inv(x: u64) -> u64 {
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

    fn exp(b: u64, p: u64) -> u64 {
        if b == 0 { return 0; }
        else if p == 0 { return 1; }

        let mut r = 1;
        let mut b = b;
        let mut p = p;

        // TODO: optimize
        while p > 0 {
            if p & 1 == 1 {
                r = Self::mul(r, b);
            }
            p = p >> 1;
            b = Self::mul(b, b);
        }

        return r;
    }

    // ROOT OF UNITY
    // --------------------------------------------------------------------------------------------
    fn get_root_of_unity(order: usize) -> u64 {
        assert!(order != 0, "cannot get root of unity for order 0");
        assert!(order.is_power_of_two(), "order must be a power of 2");
        assert!(order.trailing_zeros() <= 32, "order cannot exceed 2^32");
        let p = 1 << (32 - order.trailing_zeros());
        return Self::exp(G, p as u64);
    }
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    
    use super::{ Field, FiniteField };

    #[test]
    fn add() {
        // identity
        let r = Field::rand();
        assert_eq!(r, Field::add(r, 0));

        // test addition within bounds
        assert_eq!(5, Field::add(2, 3));

        // test overflow
        let t = Field::MODULUS - 1;
        assert_eq!(0, Field::add(t, 1));
        assert_eq!(1, Field::add(t, 2));

        // test random values
        let r1 = Field::rand();
        let r2 = Field::rand();
        assert_eq!(test_add(r1, r2), Field::add(r1, r2));
    }

    #[test]
    fn sub() {
        // identity
        let r = Field::rand();
        assert_eq!(r, Field::sub(r, 0));

        // test subtraction within bounds
        assert_eq!(2, Field::sub(5, 3));

        // test underflow
        assert_eq!(Field::MODULUS - 2, Field::sub(3, 5));
    }

    #[test]
    fn neg() {
        let r = Field::rand();
        let nr = Field::neg(r);
        assert_eq!(0, Field::add(r, nr));
    }

    #[test]
    fn mul() {
        // identity
        let r = Field::rand();
        assert_eq!(0, Field::mul(r, 0));
        assert_eq!(r, Field::mul(r, 1));

        // test multiplication within bounds
        assert_eq!(15, Field::mul(5, 3));

        // test overflow
        let t = Field::MODULUS - 1;
        assert_eq!(1, Field::mul(t, t));
        assert_eq!(Field::MODULUS - 2, Field::mul(t, 2));
        assert_eq!(Field::MODULUS - 4, Field::mul(t, 4));

        let t = (Field::MODULUS + 1) / 2;
        assert_eq!(1, Field::mul(t, 2));

        // test random values
        let r1 = Field::rand();
        let r2 = Field::rand();
        assert_eq!(test_mul(r1, r2), Field::mul(r1, r2));
    }

    #[test]
    fn mul_acc() {
        let mut a = vec![1u64, 2, 3, 4];
        let b = vec![5u64, 6, 7, 8];
        let c = 3u64;

        Field::mul_acc(&mut a, &b, c);
        assert_eq!(vec![16, 20, 24, 28], a);
    }

    #[test]
    fn inv() {
        // identity
        assert_eq!(1, Field::inv(1));
        assert_eq!(0, Field::inv(0));

        // test random values
        let x = Field::rand();
        let y = Field::inv(x);
        assert_eq!(1, Field::mul(x, y));
    }

    #[test]
    fn inv_many() {
        let v = Field::rand_vector(1024);
        let inv_v = Field::inv_many(&v);
        for i in 0..inv_v.len() {
            assert_eq!(Field::inv(v[i]), inv_v[i]);
        }
    }

    #[test]
    fn exp() {
        // identity
        let r = Field::rand();
        assert_eq!(1, Field::exp(r, 0));
        assert_eq!(r, Field::exp(r, 1));
        assert_eq!(0, Field::exp(0, r));

        // test exponentiation within bounds
        assert_eq!(125, Field::exp(5, 3));

        // test overflow
        let t = Field::MODULUS - 1;
        assert_eq!(test_mul(t, t), Field::exp(t, 2));
        assert_eq!(test_mul(test_mul(t, t), t), Field::exp(t, 3));
    }

    #[test]
    fn rand() {
        assert!(Field::rand() < Field::MODULUS);
    }

    #[test]
    fn rand_vector() {
        let v = Field::rand_vector(1024);
        assert_eq!(1024, v.len());
        for i in 0..v.len() {
            assert!(v[i] < Field::MODULUS);
        }
    }

    #[test]
    fn prng() {
        assert_eq!(1585975022918167114u64, Field::prng([42u8; 32]));
    }
    #[test]
    fn prng_vector() {
        let expected = vec![
            1585975022918167114u64,  8820585137952568641, 15299160011266138131,  6866407899083796441,
              10162285885306164082,  7867471008095992463, 13555280605728288753,   188511605104900532,
               2199779508986021858, 14291627743304465931,   279098277252367170, 13691721925447740205,
              10211632385674463860,  3308819557792802457, 16148052607759843745, 10046899211138939420
        ];
        assert_eq!(expected, Field::prng_vector([42u8; 32], 16));
    }

    // controller methods
    fn test_add(a: u64, b: u64) -> u64 {
        ((a as u128 + b as u128) % (super::M as u128)) as u64
    }

    fn test_mul(a: u64, b: u64) -> u64 {
        ((a as u128 * b as u128) % (super::M as u128)) as u64
    }
}