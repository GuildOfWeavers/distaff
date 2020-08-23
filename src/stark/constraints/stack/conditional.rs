use super::{
    field, 
    are_equal, is_binary, binary_not, EvaluationResult, enforce_left_shift
};

// CONSTRAINT EVALUATORS
// ================================================================================================

/// Enforces constraints for CHOOSE operation. These constraints work with top 3 registers of the
/// stack and enforce that when condition = 1, x remains at the top of the stack; when
/// condition = 0, y remains at the top of the stack. Otherwise the operation fails.
pub fn enforce_choose(result: &mut [u128], aux: &mut [u128], old_stack: &[u128], new_stack: &[u128], op_flag: u128)
{
    let x = old_stack[0];
    let y = old_stack[1];
    let condition = old_stack[2];

    // the constraint is: (condition * x) + ((1 - condition) * y)
    let not_condition = binary_not(condition);
    let op_result = field::add(field::mul(condition, x), field::mul(not_condition, y));
    result.agg_constraint(0, op_flag, are_equal(new_stack[0], op_result));

    // registers beyond the 3rd are shifted left by 2 slots
    enforce_left_shift(result, old_stack, new_stack, 3, 2, op_flag);
    
    // make sure the condition was a binary value
    aux.agg_constraint(0, op_flag, is_binary(condition));
}

/// Enforces constraints for CHOOSE2 operation. These constraints work with top 6 registers of
/// the stack and enforce that when condition = 1, (x0, x1) remain at the top of the stack; when
/// condition = 0, (y0, y1) remains at the top of the stack. Otherwise the operation fails.
pub fn enforce_choose2(result: &mut [u128], aux: &mut [u128], old_stack: &[u128], new_stack: &[u128], op_flag: u128) 
{
    let x0 = old_stack[0];
    let x1 = old_stack[1];
    let y0 = old_stack[2];
    let y1 = old_stack[3];
    let condition = old_stack[4];

    // the constraints are: (condition * x0) + ((1 - condition) * y0)
    // and (condition * x1) + ((1 - condition) * y1)
    let not_condition = binary_not(condition);
    let op_result1 = field::add(field::mul(condition, x0), field::mul(not_condition, y0));
    let op_result2 = field::add(field::mul(condition, x1), field::mul(not_condition, y1));
    result.agg_constraint(0, op_flag, are_equal(new_stack[0], op_result1));
    result.agg_constraint(1, op_flag, are_equal(new_stack[1], op_result2));

    // registers beyond the 6th are shifted left by 4 slots
    enforce_left_shift(result, old_stack, new_stack, 6, 4, op_flag);

    // make sure the condition was a binary value
    aux.agg_constraint(0, op_flag, is_binary(condition));
}

/// Enforces constraints for CSWAP2 operation. These constraints work with top 6 registers of the
/// stack and enforce that when condition = 1, (v2, v3) move to the top of the stack; when
/// condition = 0, top 4 elements of the stack remain unchanged. 
pub fn enforce_cswap2(result: &mut [u128], aux: &mut [u128], old_stack: &[u128], new_stack: &[u128], op_flag: u128) 
{
    let v0 = old_stack[0];
    let v1 = old_stack[1];
    let v2 = old_stack[2];
    let v3 = old_stack[3];
    let condition = old_stack[4];


    let not_condition = binary_not(condition);
    let op_result0 = field::add(field::mul(condition, v2), field::mul(not_condition, v0));
    let op_result1 = field::add(field::mul(condition, v3), field::mul(not_condition, v1));
    let op_result2 = field::add(field::mul(condition, v0), field::mul(not_condition, v2));
    let op_result3 = field::add(field::mul(condition, v1), field::mul(not_condition, v3));
    result.agg_constraint(0, op_flag, are_equal(new_stack[0], op_result0));
    result.agg_constraint(1, op_flag, are_equal(new_stack[1], op_result1));
    result.agg_constraint(2, op_flag, are_equal(new_stack[2], op_result2));
    result.agg_constraint(3, op_flag, are_equal(new_stack[3], op_result3));

    // registers beyond the 6th are shifted left by 2 slots
    enforce_left_shift(result, old_stack, new_stack, 6, 2, op_flag);

    // make sure the condition was a binary value
    aux.agg_constraint(0, op_flag, is_binary(condition));
}