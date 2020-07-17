use crate::math::field::{ mul };
use super::{ TraceState, is_binary, binary_not };

// CONSTRAINT EVALUATOR
// ================================================================================================

pub fn enforce_op_bits(result: &mut [u128], current: &TraceState, next: &TraceState)
{
    let mut i = 0;

    // make sure all op bits are binary and compute their product
    let mut cf_bit_prod = 1;
    for &op_bit in current.cf_op_bits() {
        result[i] = is_binary(op_bit);
        cf_bit_prod = mul(cf_bit_prod, op_bit);
        i += 1;
    }

    let mut ld_bit_prod = 1;
    for &op_bit in current.ld_op_bits() {
        result[i] = is_binary(op_bit);
        ld_bit_prod = mul(ld_bit_prod, op_bit);
        i += 1;
    }

    let mut hd_bit_prod = 1;
    for &op_bit in current.hd_op_bits() {
        result[i] = is_binary(op_bit);
        hd_bit_prod = mul(hd_bit_prod, op_bit);
        i += 1;
    }

    // ld_ops and hd_ops cannot be simultaneously set to all 0s
    result[i] = mul(binary_not(ld_bit_prod), binary_not(hd_bit_prod));
    i += 1;

    // when cf_ops are not all 0s, ld_ops and hd_ops must be all 1s
    result[i] = mul(cf_bit_prod, binary_not(mul(ld_bit_prod, hd_bit_prod)));
    i += 1;

    // TODO: PUSH is allowed only on multiples of 8
    // TODO: BEGIN, LOOP, BREAK, and WRAP are allowed only on one less than multiple of 16
    // TODO: TEND and FEND is allowed only on multiples of 16
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {

    use std::panic::catch_unwind;
    use super::{ TraceState };

    #[test]
    fn op_bits_are_binary() {

        let mut state1 = TraceState::new(1, 0, 1);
        let state2 = TraceState::new(1, 0, 1);

        let mut result = vec![0; 12];

        state1.set_op_bits([0, 0, 0, 1, 1, 1, 1, 1, 1, 1]);
        super::enforce_op_bits(&mut result, &state1, &state2);
        assert_eq!(vec![0; 12], result);

        state1.set_op_bits([2, 0, 0, 1, 1, 1, 1, 1, 1, 1]);
        let t = catch_unwind(|| {
            let mut result = vec![0; 12];
            super::enforce_op_bits(&mut result, &state1, &state2);
            assert_eq!(vec![0; 12], result);
        });
        assert_eq!(t.is_ok(), false);
    }
}