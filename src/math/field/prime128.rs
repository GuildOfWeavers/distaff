use std::ops::Range;
use super::{ FiniteField, Field };

// CONSTANTS
// ================================================================================================

// Field modulus = 2^64 - 45 * 2^32 + 1
pub const M: u128 = 18446743880436023297;

// 2^32 root of unity
pub const G: u128 = 8387321423513296549;

// 64-BIT FIELD IMPLEMENTATION
// ================================================================================================
impl FiniteField<u128> for Field {

    const MODULUS: u128 = M;
    const RANGE: Range<u128> = Range { start: 0, end: M };

    const ZERO: u128 = 0;
    const ONE: u128 = 1;
    
    // BASIC ARITHMETIC
    // --------------------------------------------------------------------------------------------
    fn add(a: u128, b: u128) -> u128 {
        let mut z = a + b;
        if z >= M {
            z = z - M;
        }
        return z;
    }

    fn sub(a: u128, b: u128) -> u128 {
        // TODO
        return 0;
    }

    fn mul(a: u128, b: u128) -> u128 {
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