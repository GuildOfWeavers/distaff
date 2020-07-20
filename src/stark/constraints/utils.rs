use crate::math::{ field, polynom, fft };
use crate::utils::{ filled_vector };
use crate::{ BASE_CYCLE_LENGTH };

// BASIC CONSTRAINTS OPERATORS
// ================================================================================================

#[inline(always)]
pub fn is_zero(v: u128) -> u128 {
    return v;
}

#[inline(always)]
pub fn is_binary(v: u128) -> u128 {
    return field::sub(field::mul(v, v), v);
}

#[inline(always)]
pub fn binary_not(v: u128) -> u128 {
    return field::sub(field::ONE, v);
}

#[inline(always)]
pub fn are_equal(v1: u128, v2: u128) -> u128 {
    return field::sub(v1, v2);
}

// COMMON STACK CONSTRAINTS
// ================================================================================================

/// Enforces that stack values starting from `from_slot` haven't changed. All constraints in the
/// `result` slice are filled in.
pub fn enforce_stack_copy(result: &mut [u128], old_stack: &[u128], new_stack: &[u128], from_slot: usize, op_flag: u128)
{
    for i in from_slot..result.len() {
        result.agg_constraint(i, op_flag, are_equal(old_stack[i], new_stack[i]));
    }
}

/// Enforces that values in the stack were shifted to the right by `num_slots`. Constraints in 
/// the `result` slice are filled in starting from `num_slots` index.
pub fn enforce_right_shift(result: &mut [u128], old_stack: &[u128], new_stack: &[u128], num_slots: usize, op_flag: u128)
{
    for i in num_slots..result.len() {
        result.agg_constraint(i, op_flag, are_equal(old_stack[i - num_slots], new_stack[i]));
    }
}

/// Enforces that values in the stack were shifted to the left by `num_slots` starting from
/// `from_slots`. All constraints in the `result` slice are filled in.
pub fn enforce_left_shift(result: &mut [u128], old_stack: &[u128], new_stack: &[u128], from_slot: usize, num_slots: usize, op_flag: u128)
{
    // make sure values in the stack were shifted by `num_slots` to the left
    let start_idx = from_slot - num_slots;
    let remainder_idx = result.len() - num_slots;
    for i in start_idx..remainder_idx {
        result.agg_constraint(i, op_flag, are_equal(old_stack[i + num_slots], new_stack[i]));
    }

    // also make sure that remaining slots were filled in with 0s
    for i in remainder_idx..result.len() {
        result.agg_constraint(i, op_flag, is_zero(new_stack[i]));
    }
}

// TRAIT TO SIMPLIFY CONSTRAINT AGGREGATION
// ================================================================================================

pub trait EvaluationResult {
    fn agg_constraint(&mut self, index: usize, flag: u128, value: u128);
}

impl EvaluationResult for [u128] {
    fn agg_constraint(&mut self, index: usize, flag: u128, value: u128) {
        self[index] = field::add(self[index], field::mul(flag, value));
    }
}

impl EvaluationResult for Vec<u128> {
    fn agg_constraint(&mut self, index: usize, flag: u128, value: u128) {
        self[index] = field::add(self[index], field::mul(flag, value));
    }
}

// CONSTANT INTERPOLATION AND EXTENSIONS
// ================================================================================================
pub fn extend_constants(constants: &[[u128; BASE_CYCLE_LENGTH]], extension_factor: usize) -> (Vec<Vec<u128>>, Vec<Vec<u128>>)
{
    let root = field::get_root_of_unity(BASE_CYCLE_LENGTH);
    let inv_twiddles = fft::get_inv_twiddles(root, BASE_CYCLE_LENGTH);

    let domain_size = BASE_CYCLE_LENGTH * extension_factor;
    let domain_root = field::get_root_of_unity(domain_size);
    let twiddles = fft::get_twiddles(domain_root, domain_size);

    let mut polys = Vec::with_capacity(constants.len());
    let mut evaluations = Vec::with_capacity(constants.len());

    for constant in constants.iter() {
        let mut extended_constant = filled_vector(BASE_CYCLE_LENGTH, domain_size, field::ZERO);
        extended_constant.copy_from_slice(constant);

        polynom::interpolate_fft_twiddles(&mut extended_constant, &inv_twiddles, true);
        polys.push(extended_constant.clone());

        unsafe { extended_constant.set_len(extended_constant.capacity()); }
        polynom::eval_fft_twiddles(&mut extended_constant, &twiddles, true);

        evaluations.push(extended_constant);
    }

    return (polys, evaluations);
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {

    #[test]
    fn enforce_left_shift() {

        let op_flag = 1;

        // sift left by 1 starting from 1
        let mut result = vec![0; 8];
        super::enforce_left_shift(&mut result,
            &[1, 2, 3, 4, 5, 6, 7, 8],
            &[1, 2, 3, 4, 5, 6, 7, 8],
            1, 1,
            op_flag);
        assert_eq!(vec![1, 1, 1, 1, 1, 1, 1, 8], result);

        // sift left by 2 starting from 2
        let mut result = vec![0; 8];
        super::enforce_left_shift(&mut result,
            &[1, 2, 3, 4, 5, 6, 7, 8],
            &[1, 2, 3, 4, 5, 6, 7, 8],
            2, 2,
            op_flag);
        assert_eq!(vec![2, 2, 2, 2, 2, 2, 7, 8], result);

        // sift left by 1 starting from 2
        let mut result = vec![0; 8];
        super::enforce_left_shift(&mut result,
            &[1, 2, 3, 4, 5, 6, 7, 8],
            &[1, 2, 3, 4, 5, 6, 7, 8],
            2, 1,
            op_flag);
        assert_eq!(vec![0, 1, 1, 1, 1, 1, 1, 8], result);

        // sift left by 4 starting from 6
        let mut result = vec![0; 8];
        super::enforce_left_shift(&mut result,
            &[1, 2, 3, 4, 5, 6, 7, 8],
            &[1, 2, 3, 4, 5, 6, 7, 8],
            6, 4,
            op_flag);
        assert_eq!(vec![0, 0, 4, 4, 5, 6, 7, 8], result);
    }

}