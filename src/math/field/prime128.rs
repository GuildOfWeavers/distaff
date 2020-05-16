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
        let (z0, z1, z2) = mul_128x64(a, (b >> 64) as u64);
        let (q0, q1, q2) = mul_by_mod(z2);
        
        let (z0, z1, z2) = sub_192x192(z0, z1, z2, q0, q1, q2);

        if z2 > 0 {
            // z = z - m
        }

        let (a0, a1, a2) = mul_128x64(a, b as u64);

        // z = z << 64 -> 192 bit value
        // z = z + a -> 193? bit value

        // q = m * (z >> 128)
        // z = z - q

        if z2 > 0 {
            // z = z - m
        }

        // TODO
        return 0;
    }

    fn inv(x: u128) -> u128 {
        if x == 0 { return 0 };

        // TODO
        return 0;
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


fn mul_128x64(a: u128, b: u64) -> (u64, u64, u64) {
    let z_lo = ((a as u64) as u128) * (b as u128);
    let z_hi = (a >> 64) * (b as u128);
    let z_hi = z_hi + (z_lo >> 64);
    return (z_lo as u64, z_hi as u64, (z_hi >> 64) as u64);
}

fn mul_by_mod(a: u64) -> (u64, u64, u64) {
    // TODO
    return (0, 0, 0);
}

fn sub_192x192(a0: u64, a1: u64, a2: u64, b0: u64, b1: u64, b2: u64) -> (u64, u64, u64) {
    // TODO
    return (0, 0, 0);
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
}