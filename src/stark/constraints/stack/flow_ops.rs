use super::{
    field, are_equal, EvaluationResult, enforce_left_shift, enforce_stack_copy,
};

/// Enforces constraints for NOOP operation. The constraints ensure that the stack state 
/// has not changed between two steps.
pub fn enforce_noop(result: &mut [u128], old_stack: &[u128], new_stack: &[u128], op_flag: u128)
{
    enforce_stack_copy(result, old_stack, new_stack, 0, op_flag);
}

/// Enforces constraints for ASSERT operation. The constraints are similar to DROP operation, but
/// have an auxiliary constraint which enforces that 1 - x = 0, where x is the top of the stack.
pub fn enforce_assert(result: &mut [u128], aux: &mut [u128], old_stack: &[u128], new_stack: &[u128], op_flag: u128)
{
    enforce_left_shift(result, old_stack, new_stack, 1, 1, op_flag);
    aux.agg_constraint(0, op_flag, are_equal(field::ONE, old_stack[0]));
}

/// Enforces constraints for ASSERTEQ operation. The stack is shifted by 2 registers the left and
/// an auxiliary constraint enforces that the first element of the stack is equal to the second.
pub fn enforce_asserteq(result: &mut [u128], aux: &mut [u128], old_stack: &[u128], new_stack: &[u128], op_flag: u128)
{
    enforce_left_shift(result, old_stack, new_stack, 2, 2, op_flag);
    aux.agg_constraint(0, op_flag, are_equal(old_stack[0], old_stack[1]));
}