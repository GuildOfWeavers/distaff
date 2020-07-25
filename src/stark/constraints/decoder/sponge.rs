use crate::{
    math::field::{ self, add, sub, mul },
    utils::sponge::{ apply_sbox, apply_mds, apply_inv_mds },
};
use super::{ TraceState, UserOps, are_equal, EvaluationResult, SPONGE_WIDTH };

// CONSTRAINT EVALUATOR
// ================================================================================================

pub fn enforce_hacc(result: &mut [u128], current: &TraceState, next: &TraceState, ark: &[u128], op_flag: u128)
{
    // determine current op_value
    let stack_top = next.user_stack()[0];
    let push_flag = current.hd_op_flags()[UserOps::Push.hd_index()];
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
#[cfg(test)]
mod tests {
    
    use crate::{ SPONGE_WIDTH, BASE_CYCLE_LENGTH };
    use crate::utils::sponge::{ apply_round as apply_hacc_round, ARK };
    use super::{ TraceState, super::transpose_ark_constants };

    #[test]
    fn op_hacc() {

        let ark: Vec<Vec<u128>> = ARK.iter().map(|row| row.to_vec()).collect();
        let ark = transpose_ark_constants(ark, BASE_CYCLE_LENGTH);

        // correct transition, push.7
        let push_value = 7;
        let state1 = TraceState::from_vec(1, 0, 1, &vec![0,  1, 2, 3, 4,  0, 0, 0,  1, 1, 1, 1, 1,  0, 0,  0,  0]);

        let mut sponge = [1, 2, 3, 4];
        apply_hacc_round(&mut sponge, state1.op_code(), push_value, 0);
        
        let state2 = build_state(&sponge, push_value);

        let mut evaluations = vec![0; 4];
        super::enforce_hacc(&mut evaluations, &state1, &state2, &ark[0], 1);
        assert_eq!(vec![0, 0, 0, 0], evaluations);

        // correct transition, non-push op
        let state1 = TraceState::from_vec(1, 0, 1, &vec![0,  1, 2, 3, 4,  0, 0, 0,  0, 0, 0, 0, 0,  1, 1,  0,  0]);

        let mut sponge = [1, 2, 3, 4];
        apply_hacc_round(&mut sponge, state1.op_code(), 0, 0);

        let state2 = build_state(&sponge, 9);
        let mut evaluations = vec![0; 4];
        super::enforce_hacc(&mut evaluations, &state1, &state2, &ark[0], 1);
        assert_eq!(vec![0, 0, 0, 0], evaluations);

        // incorrect transition, push.7
        let push_value = 7;
        let state1 = TraceState::from_vec(1, 0, 1, &vec![0,  1, 2, 3, 4,  0, 0, 0,  1, 1, 1, 1, 1,  0, 0,  0,  0]);

        let mut sponge = [1, 2, 3, 4];
        apply_hacc_round(&mut sponge, state1.op_code(), push_value, 0);
        
        let state2 = build_state(&sponge, push_value - 1);

        let mut evaluations = vec![0; 4];
        super::enforce_hacc(&mut evaluations, &state1, &state2, &ark[0], 1);
        assert_eq!(vec![0, 340282366920938463463374557953744961536, 0, 0], evaluations);

        // incorrect transition, non-push op
        let state1 = TraceState::from_vec(1, 0, 1, &vec![0,  1, 2, 3, 4,  0, 0, 0,  0, 0, 0, 0, 0,  1, 1,  0,  0]);

        let mut sponge = [1, 2, 3, 4];
        apply_hacc_round(&mut sponge, state1.op_code(), 9, 0);

        let state2 = build_state(&sponge, 9);
        let mut evaluations = vec![0; 4];
        super::enforce_hacc(&mut evaluations, &state1, &state2, &ark[0], 1);
        assert_eq!(vec![0, 340282366920938463463374557953744961528, 0, 0], evaluations);
    }

    // HELPER FUNCTIONS
    // --------------------------------------------------------------------------------------------
    fn build_state(sponge: &[u128; SPONGE_WIDTH], push_value: u128) -> TraceState {
        let state = vec![
            0, sponge[0], sponge[1], sponge[2], sponge[3],  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  0,  push_value
        ];
        return TraceState::from_vec(1, 0, 1, &state);
    }
}