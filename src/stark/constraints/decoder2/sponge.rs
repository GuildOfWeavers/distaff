use crate::math::{ field::{ self, add, sub, mul } };
use crate::utils::accumulator::{ apply_sbox, apply_mds, apply_inv_mds };
use super::{ TraceState, are_equal, EvaluationResult };

// TODO: move to global constants
const SPONGE_WIDTH: usize = 4;
const SPONGE_CYCLE_LENGTH: usize = 16;

// CONSTRAINT EVALUATOR
// ================================================================================================

pub fn enforce_hacc(result: &mut [u128], current: &TraceState, next: &TraceState, ark: &[u128], op_flag: u128)
{
    // determine current op_value
    let stack_top = next.user_stack()[0];
    let push_flag = current.hd_op_flags()[0];   // TODO: use constant for lookup
    let op_value = mul(stack_top, push_flag);

    // evaluate the first half of Rescue round
    let mut old_sponge = [field::ZERO; SPONGE_WIDTH];
    old_sponge.copy_from_slice(current.sponge());
    for i in 0..SPONGE_WIDTH {
        old_sponge[i] = add(old_sponge[i], ark[i]);
    }
    apply_sbox(&mut old_sponge);
    apply_mds(&mut old_sponge);

    // op_code injection
    old_sponge[0] = add(old_sponge[0], current.op_code());
    old_sponge[1] = add(old_sponge[1], op_value);
    
    // evaluate inverse of the second half of Rescue round
    let mut new_sponge = [field::ZERO; SPONGE_WIDTH];
    new_sponge.copy_from_slice(next.sponge());
    apply_inv_mds(&mut new_sponge);
    apply_sbox(&mut new_sponge);
    for i in 0..SPONGE_WIDTH {
        new_sponge[i] = sub(new_sponge[i], ark[SPONGE_WIDTH + i]);
    }

    // add the constraints to the result
    for i in 0..SPONGE_WIDTH {
        result.agg_constraint(i, op_flag, are_equal(old_sponge[i], new_sponge[i]));
    }
}

// TESTS
// ================================================================================================