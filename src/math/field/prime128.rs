use std::ops::Range;
use super::{ FiniteField, Field };

// CONSTANTS
// ================================================================================================

// Field modulus = 2^128 - 9 * 2^32 + 1
pub const M: u128 = 340282366920938463463374607393113505793;

// 2^32 root of unity
pub const G: u128 = 8387321423513296549;

// 128-BIT FIELD IMPLEMENTATION
// ================================================================================================
impl FiniteField<u128> for Field {

    const MODULUS: u128 = M;
    const RANGE: Range<u128> = Range { start: 0, end: M };

    const ZERO: u128 = 0;
    const ONE: u128 = 1;
    
    // BASIC ARITHMETIC
    // --------------------------------------------------------------------------------------------
    fn add(a: u128, b: u128) -> u128 {
        let z = M - b;
        return if a < z { M - z + a } else { a - z};
    }

    fn sub(a: u128, b: u128) -> u128 {
        return if a < b { M - b + a } else { a - b };
    }

    fn mul(a: u128, b: u128) -> u128 {

        let (z, x2) = mul_reduce(a, (b >> 64) as u64);

        let (y0, y1, y2) = mul_128x64(a, b as u64);                 // x = a * b_lo

        let (mut y1, carry) = adc(y1, z as u64, 0);
        let (mut y2, carry) = adc(y2, (z >> 64) as u64, carry);
        let y3 = x2 + carry;

        if y3 == 1 {
            let (t0, t1) = sub_modulus(y1, y2);
            y1 = t0; y2 = t1;
        }
        
        let (q0, q1, q2) = mul_by_mod(y2);                          // q = m * (z >> 128)

        // z = z - q
        let (mut z0, mut z1, z2) = sub_192x192(y0, y1, y2, q0, q1, q2);

        if z2 == 1 || (z1 == (M >> 64) as u64 && z0 > (M as u64)) {
            let (t0, t1) = sub_modulus(z0, z1);
            z0 = t0; z1 = t1;
        }

        return ((z1 as u128) << 64) + (z0 as u128);
    }

    fn inv(x: u128) -> u128 {
        if x == 0 { return 0 };

        // initialize a, v, u, and d variables
        let mut v = M;
        let (mut a0, mut a1, mut a2) = (0, 0, 0);
        let (mut u0, mut u1, mut u2) = if x & 1 == 1 {
            // u = x
            (x as u64, (x >> 64) as u64, 0)
        }
        else {
            // u = x + m
            add_192x192(x as u64, (x >> 64) as u64, 0, M as u64, (M >> 64) as u64, 0)
        };
        // d = m - 1
        let (mut d0, mut d1, mut d2) = ((M as u64) - 1, (M >> 64) as u64, 0);

        // compute the inverse
        while v != 1 {
            let u_lo = (u0 as u128) + ((u1 as u128) << 64);
            while u2 > 0 || u_lo > v { // u > v
                // u = u - v
                let (t0, t1, t2) = sub_192x192(u0, u1, u2, v as u64, (v >> 64) as u64, 0);
                u0 = t0; u1 = t1; u2 = t2;

                // d = d + a
                let (t0, t1, t2) = add_192x192(d0, d1, d2, a0, a1, a2);
                d0 = t0; d1 = t1; d2 = t2;

                while u0 & 1 == 0 {
                    if d0 & 1 == 1 {
                        // d = d + m
                        let (t0, t1, t2) = add_192x192(d0, d1, d2, M as u64, (M >> 64) as u64, 0);
                        d0 = t0; d1 = t1; d2 = t2;
                    }

                    // u = u >> 1
                    u0 = (u0 >> 1) | ((u1 & 1) << 63);
                    u1 = (u1 >> 1) | ((u2 & 1) << 63);
                    u2 = u2 >> 1;

                    // d = d >> 1
                    d0 = (d0 >> 1) | ((d1 & 1) << 63);
                    d1 = (d1 >> 1) | ((d2 & 1) << 63);
                    d2 = d2 >> 1;
                }
            }

            // v = v - u
            let u_lo = (u0 as u128) + ((u1 as u128) << 64);
            v = v - u_lo;
            
            // a = a + d
            let (t0, t1, t2) = add_192x192(a0, a1, a2, d0, d1, d2);
            a0 = t0; a1 = t1; a2 = t2;

            while v & 1 == 0 {
                if a0 & 1 == 1 {
                    // a = a + m
                    let (t0, t1, t2) = add_192x192(a0, a1, a2, M as u64, (M >> 64) as u64, 0);
                    a0 = t0; a1 = t1; a2 = t2;
                }

                v = v >> 1;

                // a = a >> 1
                a0 = (a0 >> 1) | ((a1 & 1) << 63);
                a1 = (a1 >> 1) | ((a2 & 1) << 63);
                a2 = a2 >> 1;
            }
        }

        // a = a mod m
        let mut a = (a0 as u128) + ((a1 as u128) << 64);
        while a2 > 0 || a >= M {
            let (t0, t1, t2) = sub_192x192(a0, a1, a2, M as u64, (M >> 64) as u64, 0);
            a0 = t0; a1 = t1; a2 = t2;
            a = (a0 as u128) + ((a1 as u128) << 64);
        }

        return a;
    }

    fn exp(b: u128, p: u128) -> u128 {
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
    fn get_root_of_unity(order: usize) -> u128 {
        assert!(order != 0, "cannot get root of unity for order 0");
        assert!(order.is_power_of_two(), "order must be a power of 2");
        assert!(order.trailing_zeros() <= 32, "order cannot exceed 2^32");
        let p = 1 << (32 - order.trailing_zeros());
        return Self::exp(G, p as u128);
    }
}

// HELPER FUNCTIONS
// ================================================================================================

fn mul_reduce(a: u128, b: u64) -> (u128, u64) {

    let (z0, z1, z2) = mul_128x64(a, b);
    let (q0, q1, q2) = mul_by_mod(z2);
    let (z0, z1, z2) = sub_192x192(z0, z1, z2, q0, q1, q2);

    return ((z0 as u128) + ((z1 as u128) << 64), z2);
}

#[inline(always)]
fn mul_128x64(a: u128, b: u64) -> (u64, u64, u64) {
    let z_lo = ((a as u64) as u128) * (b as u128);
    let z_hi = (a >> 64) * (b as u128);
    let z_hi = z_hi + (z_lo >> 64);
    return (z_lo as u64, z_hi as u64, (z_hi >> 64) as u64);
}

#[inline(always)]
fn mul_by_mod(a: u64) -> (u64, u64, u64) {
    let a_lo = (a as u128).wrapping_mul(M);
    let a_hi = if a == 0 { 0 } else { a - 1 };
    return (a_lo as u64, (a_lo >> 64) as u64, a_hi);
}

#[inline(always)]
fn sub_modulus(a_lo: u64, a_hi: u64) -> (u64, u64) {
    let z_lo = (a_lo as u128).wrapping_sub((M as u64) as u128);
    let z_hi = a_hi.wrapping_sub((M >> 64) as u64).wrapping_sub((z_lo >> 127) as u64);
    return (z_lo as u64, z_hi);
}

#[inline(always)]
fn sub_192x192(a0: u64, a1: u64, a2: u64, b0: u64, b1: u64, b2: u64) -> (u64, u64, u64) {
    let z0 = (a0 as u128).wrapping_sub(b0 as u128);
    let z1 = (a1 as u128).wrapping_sub((b1 as u128) + (z0 >> 127));
    let z2 = (a2 as u128).wrapping_sub((b2 as u128) + (z1 >> 127));
    return (z0 as u64, z1 as u64, z2 as u64);
}

#[inline(always)]
fn add_192x192(a0: u64, a1: u64, a2: u64, b0: u64, b1: u64, b2: u64) -> (u64, u64, u64) {
    let z0 = (a0 as u128) + (b0 as u128);
    let z1 = (a1 as u128) + (b1 as u128) + (z0 >> 64);
    let z2 = (a2 as u128) + (b2 as u128) + (z1 >> 64);
    return (z0 as u64, z1 as u64, z2 as u64);
}

#[inline(always)]
pub const fn adc(a: u64, b: u64, carry: u64) -> (u64, u64) {
    let ret = (a as u128) + (b as u128) + (carry as u128);
    return (ret as u64, (ret >> 64) as u64);
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {

    use std::convert::TryInto;
    use num_bigint::{ BigUint };
    use super::{ Field, FiniteField };

    #[test]
    fn add() {
        // identity
        let r: u128 = Field::rand();
        assert_eq!(r, Field::add(r, 0));

        // test addition within bounds
        assert_eq!(5, Field::add(2u64, 3));

        // test overflow
        let m: u128 = Field::MODULUS;
        let t = m - 1;
        assert_eq!(0, Field::add(t, 1));
        assert_eq!(1, Field::add(t, 2));

        // test random values
        let r1: u128 = Field::rand();
        let r2: u128 = Field::rand();

        let expected = (BigUint::from(r1) + BigUint::from(r2)) % BigUint::from(super::M);
        let expected = u128::from_le_bytes((expected.to_bytes_le()[..]).try_into().unwrap());
        assert_eq!(expected, Field::add(r1, r2));
    }

    #[test]
    fn sub() {
        // identity
        let r: u128 = Field::rand();
        assert_eq!(r, Field::sub(r, 0));

        // test subtraction within bounds
        assert_eq!(2, Field::sub(5u128, 3));

        // test underflow
        let m: u128 = Field::MODULUS;
        assert_eq!(m - 2, Field::sub(3u128, 5));
    }

    #[test]
    fn mul() {
        // identity
        let r: u128 = Field::rand();
        assert_eq!(0, Field::mul(r, 0));
        assert_eq!(r, Field::mul(r, 1));

        // test multiplication within bounds
        assert_eq!(15, Field::mul(5u128, 3));

        // test overflow
        let m: u128 = Field::MODULUS;
        let t = m - 1;
        assert_eq!(1, Field::mul(t, t));
        assert_eq!(m - 2, Field::mul(t, 2));
        assert_eq!(m - 4, Field::mul(t, 4));

        let t = (m + 1) / 2;
        assert_eq!(1, Field::mul(t, 2));

        // test random values
        let v1: Vec<u128> = Field::rand_vector(1000);
        let v2: Vec<u128> = Field::rand_vector(1000);
        for i in 0..v1.len() {
            let r1 = v1[i];
            let r2 = v2[i];

            let result = (BigUint::from(r1) * BigUint::from(r2)) % BigUint::from(super::M);
            let result = result.to_bytes_le();
            let mut expected = [0u8; 16];
            expected[0..result.len()].copy_from_slice(&result);
            let expected = u128::from_le_bytes(expected);

            if expected != Field::mul(r1, 32) {
                println!("failed for: {} * {}", r1, r2);
                assert_eq!(expected, Field::mul(r1, r2));
            }
        }
    }

    #[test]
    fn inv() {
        // identity
        assert_eq!(1, Field::inv(1u128));
        assert_eq!(0, Field::inv(0u128));

        // test random values
        let x: u64 = Field::rand();
        let y = Field::inv(x);
        assert_eq!(1, Field::mul(x, y));
    }
}