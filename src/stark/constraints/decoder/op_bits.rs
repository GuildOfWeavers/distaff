use super::{
    field::{ self, mul, add },
    TraceState, FlowOps, UserOps, is_binary, binary_not, are_equal, EvaluationResult,
    CYCLE_MASK_IDX, PREFIX_MASK_IDX, PUSH_MASK_IDX,
};

// CONSTRAINT EVALUATOR
// ================================================================================================

pub fn enforce_op_bits(result: &mut [u128], current: &TraceState, next: &TraceState, masks: &[u128; 3])
{
    let mut i = 0;

    // make sure all op bits are binary and compute their product/sum
    let mut flow_op_bit_sum = 0;
    for &op_bit in current.flow_op_bits() {
        result[i] = is_binary(op_bit);
        flow_op_bit_sum = add(flow_op_bit_sum, op_bit);
        i += 1;
    }

    let mut user_op_bit_prod = 1;
    for &op_bit in current.user_op_bits() {
        result[i] = is_binary(op_bit);
        user_op_bit_prod = mul(user_op_bit_prod, binary_not(op_bit));
        i += 1;
    }

    // when flow_op = HACC, operation counter should be incremented by 1;
    // otherwise, operation counter should remain the same
    let op_counter = current.op_counter();
    let is_hacc = current.get_flow_op_flags(FlowOps::Hacc);
    let hacc_transition = mul(add(op_counter, field::ONE), is_hacc);
    let rest_transition = mul(op_counter, binary_not(is_hacc));
    result[i] = are_equal(add(hacc_transition, rest_transition), next.op_counter());
    i += 1;

    // unless flow_op is HACC, user_op must be NOOP
    result[i] = mul(flow_op_bit_sum, binary_not(user_op_bit_prod));
    i += 1;

    // TODO: add comment
    let mut low_user_op_bit_sum = current.user_op_bits()[0];
    for j in 1..4 {
        low_user_op_bit_sum = add(current.user_op_bits()[j], low_user_op_bit_sum);
    }
    let is_one_bit = binary_not(mul(low_user_op_bit_sum, low_user_op_bit_sum));
    let is_one_bit = add(is_one_bit, current.user_op_bits()[4]);
    result[i] = mul(mul(is_one_bit, current.user_op_bits()[5]), op_counter);
    i += 1;

    // TODO: add comment
    let mut low_user_op_bit_prod = current.user_op_bits()[0];
    for j in 1..5 {
        low_user_op_bit_prod = mul(low_user_op_bit_prod, current.user_op_bits()[j]);
    }
    result[i] = mul(binary_not(current.user_op_bits()[5]), low_user_op_bit_prod);
    i += 1;
    
    // VOID can be followed only by VOID
    let current_void_flag = current.get_flow_op_flags(FlowOps::Void);
    let next_void_flag = next.get_flow_op_flags(FlowOps::Void);
    result[i] = mul(current_void_flag, binary_not(next_void_flag));
    i += 1;

    // BEGIN, LOOP, BREAK, and WRAP are allowed only on one less than multiple of 16
    let prefix_mask = masks[PREFIX_MASK_IDX];
    result.agg_constraint(i, current.get_flow_op_flags(FlowOps::Begin), prefix_mask);
    result.agg_constraint(i, current.get_flow_op_flags(FlowOps::Loop),  prefix_mask);
    result.agg_constraint(i, current.get_flow_op_flags(FlowOps::Wrap),  prefix_mask);
    result.agg_constraint(i, current.get_flow_op_flags(FlowOps::Break), prefix_mask);

    // TEND and FEND is allowed only on multiples of 16
    let base_cycle_mask = masks[CYCLE_MASK_IDX];
    result.agg_constraint(i, current.get_flow_op_flags(FlowOps::Tend), base_cycle_mask);
    result.agg_constraint(i, current.get_flow_op_flags(FlowOps::Fend), base_cycle_mask);

    // PUSH is allowed only on multiples of 8
    let push_cycle_mask = masks[PUSH_MASK_IDX];
    result.agg_constraint(i, current.get_user_op_flag(UserOps::Push), push_cycle_mask);
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {

    use super::{ TraceState, FlowOps, UserOps, super::NUM_OP_CONSTRAINTS };

    #[test]
    fn op_bits_are_binary() {

        let success_result = vec![0; NUM_OP_CONSTRAINTS];

        // all bits are 1s: success
        let state = new_state(FlowOps::Void as u8, UserOps::Noop as u8, 1);
        assert_eq!(success_result, evaluate_state(&state, [0, 0, 0], false));

        // control flow bits are not binary
        for i in 0..3 {
            let mut op_bits = [1; 3];
            op_bits[i] = 3;
            let mut expected_evaluations = vec![0; 10];
            expected_evaluations[i] = 3 * 3 - 3;

            let state = new_state_from_bits(op_bits, [1, 1, 1, 1, 1, 1, 1]);
            assert_eq!(expected_evaluations, &evaluate_state(&state, [0, 0, 0], false)[..10]);
        }

        // user bits are not binary
        for i in 0..7 {
            let mut op_bits = [1, 1, 1, 1, 1, 1, 1];
            op_bits[i] = 3;
            let mut expected_evaluations = vec![0; 10];
            expected_evaluations[i + 3] = 3 * 3 - 3;

            let state = new_state_from_bits([0, 0, 0], op_bits);
            assert_eq!(expected_evaluations, &evaluate_state(&state, [0, 0, 0], false)[..10]);
        }
    }

    #[test]
    fn invalid_op_combinations() {

        let success_result = vec![0; NUM_OP_CONSTRAINTS];

        // user op bits cannot be all 0s
        for cf_op in 0..8 {
            let state = new_state(cf_op, 0, 1);
            assert_ne!(success_result, evaluate_state(&state, [0, 0, 0], false));
        }

        // when cf_ops are not all 0s, user_ops must be all 1s
        for cf_op in 1..8 {
            for user_op in 0..127 {
                let state = new_state(cf_op as u8, user_op as u8, 1);
                assert_ne!(success_result, evaluate_state(&state, [0, 0, 0], false));
            }

            let state = new_state(cf_op as u8, UserOps::Noop as u8, 1);
            assert_eq!(success_result, evaluate_state(&state, [0, 0, 0], false));
        }
    }

    #[test]
    fn invalid_op_alignment() {

        let success_result = vec![0; NUM_OP_CONSTRAINTS];
        
        // TEND and FEND are allowed only on multiples of 16
        let state = new_state(FlowOps::Tend as u8, UserOps::Noop as u8, 1);
        assert_eq!(success_result, evaluate_state(&state, [0, 0, 0], false));
        assert_ne!(success_result, evaluate_state(&state, [1, 0, 0], false));

        let state = new_state(FlowOps::Fend as u8, UserOps::Noop as u8, 1);
        assert_eq!(success_result, evaluate_state(&state, [0, 0, 0], false));
        assert_ne!(success_result, evaluate_state(&state, [1, 0, 0], false));

        // BEGIN, LOOP, WRAP, and BREAK are allowed only on one less than multiples of 16
        let state = new_state(FlowOps::Begin as u8, UserOps::Noop as u8, 1);
        assert_eq!(success_result, evaluate_state(&state, [0, 0, 0], false));
        assert_ne!(success_result, evaluate_state(&state, [0, 1, 0], false));

        let state = new_state(FlowOps::Loop as u8, UserOps::Noop as u8, 1);
        assert_eq!(success_result, evaluate_state(&state, [0, 0, 0], false));
        assert_ne!(success_result, evaluate_state(&state, [0, 1, 0], false));

        let state = new_state(FlowOps::Wrap as u8, UserOps::Noop as u8, 1);
        assert_eq!(success_result, evaluate_state(&state, [0, 0, 0], false));
        assert_ne!(success_result, evaluate_state(&state, [0, 1, 0], false));

        let state = new_state(FlowOps::Break as u8, UserOps::Noop as u8, 1);
        assert_eq!(success_result, evaluate_state(&state, [0, 0, 0], false));
        assert_ne!(success_result, evaluate_state(&state, [0, 1, 0], false));

        // PUSH is allowed only on multiples of 8
        let state = new_state(FlowOps::Hacc as u8, UserOps::Push as u8, 1);
        assert_eq!(success_result, evaluate_state(&state, [0, 0, 0], true));
        assert_ne!(success_result, evaluate_state(&state, [0, 0, 1], true));
    }

    #[test]
    fn invalid_op_sequence() {
        let success_result = vec![0; NUM_OP_CONSTRAINTS];

        // void can follow non-void
        let state1 = new_state(FlowOps::Hacc as u8, UserOps::Add as u8, 1);
        let state2 = new_state(FlowOps::Void as u8, UserOps::Noop as u8, 2);
        let mut evaluations = vec![0; NUM_OP_CONSTRAINTS];
        super::enforce_op_bits(&mut evaluations, &state1, &state2, &[0, 0, 0]);
        assert_eq!(success_result, evaluations);

        // void can follow void
        let state1 = new_state(FlowOps::Void as u8, UserOps::Noop as u8, 1);
        let state2 = new_state(FlowOps::Void as u8, UserOps::Noop as u8, 1);
        let mut evaluations = vec![0; NUM_OP_CONSTRAINTS];
        super::enforce_op_bits(&mut evaluations, &state1, &state2, &[0, 0, 0]);
        assert_eq!(success_result, evaluations);

        // non-void cannot follow void
        let state1 = new_state(FlowOps::Void as u8, UserOps::Noop as u8, 1);
        let state2 = new_state(FlowOps::Hacc as u8, UserOps::Add as u8, 1);
        let mut evaluations = vec![0; NUM_OP_CONSTRAINTS];
        super::enforce_op_bits(&mut evaluations, &state1, &state2, &[0, 0, 0]);
        assert_ne!(success_result, evaluations);
    }

    // HELPER FUNCTIONS
    // --------------------------------------------------------------------------------------------
    fn new_state(flow_op: u8, user_op: u8, op_counter: u128) -> TraceState {
        let mut state = TraceState::new(1, 0, 1);
    
        let mut op_bits = [0; 9];
        for i in 0..3 {
            op_bits[i] = ((flow_op as u128) >> i) & 1;
        }
    
        for i in 0..6 {
            op_bits[i + 3] = ((user_op as u128) >> i) & 1;
        }

        state.set_op_bits(op_bits);
        state.set_op_counter(op_counter);
        return state;
    }

    fn new_state_from_bits(cf_bits: [u128; 3], u_bits: [u128; 7]) -> TraceState {
        let mut state = TraceState::new(1, 0, 1);
        state.set_op_bits([
            cf_bits[0], cf_bits[1], cf_bits[2],
            u_bits[0], u_bits[1], u_bits[2], u_bits[3], u_bits[4], u_bits[5]
        ]);
        return state;
    }

    fn evaluate_state(state: &TraceState, masks: [u128; 3], inc_counter: bool) -> Vec<u128> {
        let op_counter = if inc_counter { state.op_counter() + 1 } else { state.op_counter() };
        let next_state = new_state(FlowOps::Void as u8, UserOps::Noop as u8, op_counter);
        let mut evaluations = vec![0; NUM_OP_CONSTRAINTS];
        super::enforce_op_bits(&mut evaluations, &state, &next_state, &masks);
        return evaluations;
    }
}