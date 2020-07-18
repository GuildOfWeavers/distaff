use super::{ TraceState, are_equal, is_zero, EvaluationResult, SPONGE_WIDTH };

// CONSTRAINT EVALUATORS
// ================================================================================================

pub fn enforce_begin(result: &mut [u128], current: &TraceState, next: &TraceState, op_flag: u128)
{
    // make sure sponge state has been cleared
    let next_sponge = next.sponge();
    result.agg_constraint(0, op_flag, is_zero(next_sponge[0]));
    result.agg_constraint(1, op_flag, is_zero(next_sponge[1]));
    result.agg_constraint(2, op_flag, is_zero(next_sponge[2]));
    result.agg_constraint(3, op_flag, is_zero(next_sponge[3]));

    // make sure hash of parent block was pushed onto the context stack
    let parent_hash = current.sponge()[0];
    let ctx_stack_end = SPONGE_WIDTH + current.ctx_stack().len();
    let ctx_result = &mut result[SPONGE_WIDTH..ctx_stack_end];
    ctx_result.agg_constraint(0, op_flag, are_equal(parent_hash, next.ctx_stack()[0]));
    enforce_stack_right_shift(ctx_result, current.ctx_stack(), next.ctx_stack(), op_flag);

    // make sure loop stack didn't change
    let loop_result = &mut result[ctx_stack_end..ctx_stack_end + current.loop_stack().len()];
    enforce_stack_copy(loop_result, current.loop_stack(), next.loop_stack(), op_flag);
}

pub fn enforce_tend(result: &mut [u128], current: &TraceState, next: &TraceState, op_flag: u128)
{
    let parent_hash = current.ctx_stack()[0];
    let block_hash = current.sponge()[0];

    let next_sponge = next.sponge();
    result.agg_constraint(0, op_flag, are_equal(parent_hash, next_sponge[0]));
    result.agg_constraint(1, op_flag, are_equal(block_hash, next_sponge[1]));
    // no constraint on the 3rd element of the sponge
    result.agg_constraint(3, op_flag, is_zero(next_sponge[3]));

    // make parent hash was popped from context stack
    let ctx_stack_end = SPONGE_WIDTH + current.ctx_stack().len();
    let ctx_result = &mut result[SPONGE_WIDTH..ctx_stack_end];
    enforce_stack_left_shift(ctx_result, current.ctx_stack(), next.ctx_stack(), op_flag);

    // make sure loop stack didn't change
    let loop_result = &mut result[ctx_stack_end..ctx_stack_end + current.loop_stack().len()];
    enforce_stack_copy(loop_result, current.loop_stack(), next.loop_stack(), op_flag);
}

pub fn enforce_fend(result: &mut [u128], current: &TraceState, next: &TraceState, op_flag: u128)
{
    let parent_hash = current.ctx_stack()[0];
    let block_hash = current.sponge()[0];

    let next_sponge = next.sponge();
    result.agg_constraint(0, op_flag, are_equal(parent_hash, next_sponge[0]));
    // no constraint on the 2nd element of the sponge
    result.agg_constraint(2, op_flag, are_equal(block_hash, next_sponge[2]));
    result.agg_constraint(3, op_flag, is_zero(next_sponge[3]));

    // make sure parent hash was popped from context stack
    let ctx_stack_end = SPONGE_WIDTH + current.ctx_stack().len();
    let ctx_result = &mut result[SPONGE_WIDTH..ctx_stack_end];
    enforce_stack_left_shift(ctx_result, current.ctx_stack(), next.ctx_stack(), op_flag);

    // make sure loop stack didn't change
    let loop_result = &mut result[ctx_stack_end..ctx_stack_end + current.loop_stack().len()];
    enforce_stack_copy(loop_result, current.loop_stack(), next.loop_stack(), op_flag);
}

pub fn enforce_loop(result: &mut [u128], current: &TraceState, next: &TraceState, op_flag: u128)
{
    // make sure sponge state has been cleared
    let next_sponge = next.sponge();
    result.agg_constraint(0, op_flag, is_zero(next_sponge[0]));
    result.agg_constraint(1, op_flag, is_zero(next_sponge[1]));
    result.agg_constraint(2, op_flag, is_zero(next_sponge[2]));
    result.agg_constraint(3, op_flag, is_zero(next_sponge[3]));

    // make sure hash of parent block was pushed onto the context stack
    let parent_hash = current.sponge()[0];
    let ctx_stack_end = SPONGE_WIDTH + current.ctx_stack().len();
    let ctx_result = &mut result[SPONGE_WIDTH..ctx_stack_end];
    ctx_result.agg_constraint(0, op_flag, are_equal(parent_hash, next.ctx_stack()[0]));
    enforce_stack_right_shift(ctx_result, current.ctx_stack(), next.ctx_stack(), op_flag);

    // make sure loop stack was shifted by 1 item to the right, but don't enforce constraints
    // on the first item of the stack (which will contain loop image)
    let loop_result = &mut result[ctx_stack_end..ctx_stack_end + current.loop_stack().len()];
    enforce_stack_right_shift(loop_result, current.loop_stack(), next.loop_stack(), op_flag);
}

pub fn enforce_wrap(result: &mut [u128], current: &TraceState, next: &TraceState, op_flag: u128)
{
    // make sure sponge state has been cleared
    let next_sponge = next.sponge();
    result.agg_constraint(0, op_flag, is_zero(next_sponge[0]));
    result.agg_constraint(1, op_flag, is_zero(next_sponge[1]));
    result.agg_constraint(2, op_flag, is_zero(next_sponge[2]));
    result.agg_constraint(3, op_flag, is_zero(next_sponge[3]));

    // make sure item at the top of loop stack is equal to loop image
    // TODO

    // make sure context stack didn't change
    let ctx_stack_end = SPONGE_WIDTH + current.ctx_stack().len();
    let ctx_result = &mut result[SPONGE_WIDTH..ctx_stack_end];
    enforce_stack_copy(ctx_result, current.ctx_stack(), next.ctx_stack(), op_flag);

    // make sure loop stack didn't change
    let loop_result = &mut result[ctx_stack_end..ctx_stack_end + current.loop_stack().len()];
    enforce_stack_copy(loop_result, current.loop_stack(), next.loop_stack(), op_flag);
}

pub fn enforce_break(result: &mut [u128], current: &TraceState, next: &TraceState, op_flag: u128)
{
    // make sure sponge state didn't change
    let old_sponge = current.sponge();
    let new_sponge = next.sponge();
    for i in 0..SPONGE_WIDTH {
        result.agg_constraint(i, op_flag, are_equal(old_sponge[i], new_sponge[i]));
    }

    // make sure item at the top of loop stack is equal to loop image
    // TODO

    // make sure context stack didn't change
    let ctx_stack_end = SPONGE_WIDTH + current.ctx_stack().len();
    let ctx_result = &mut result[SPONGE_WIDTH..ctx_stack_end];
    enforce_stack_copy(ctx_result, current.ctx_stack(), next.ctx_stack(), op_flag);

    // make loop image was popped from loop stack
    let loop_result = &mut result[ctx_stack_end..ctx_stack_end + current.loop_stack().len()];
    enforce_stack_left_shift(loop_result, current.loop_stack(), next.loop_stack(), op_flag);
}

pub fn enforce_void(result: &mut [u128], current: &TraceState, next: &TraceState, op_flag: u128)
{
    // make sure sponge state didn't change
    let old_sponge = current.sponge();
    let new_sponge = next.sponge();
    for i in 0..SPONGE_WIDTH {
        result.agg_constraint(i, op_flag, are_equal(old_sponge[i], new_sponge[i]));
    }

    // make sure context stack didn't change
    let ctx_stack_end = SPONGE_WIDTH + current.ctx_stack().len();
    let ctx_result = &mut result[SPONGE_WIDTH..ctx_stack_end];
    enforce_stack_copy(ctx_result, current.ctx_stack(), next.ctx_stack(), op_flag);

    // make sure loop stack didn't change
    let loop_result = &mut result[ctx_stack_end..ctx_stack_end + current.loop_stack().len()];
    enforce_stack_copy(loop_result, current.loop_stack(), next.loop_stack(), op_flag);
}

// HELPER FUNCTIONS
// ================================================================================================

fn enforce_stack_left_shift(result: &mut [u128], old_stack: &[u128], new_stack: &[u128], op_flag: u128)
{
    if result.len() == 0 { return; }

    let last_idx = result.len() - 1;
    for i in 0..last_idx {
        result.agg_constraint(i, op_flag, are_equal(old_stack[i + 1], new_stack[i]));
    }

    result.agg_constraint(last_idx, op_flag, is_zero(new_stack[last_idx]));
}

fn enforce_stack_right_shift(result: &mut [u128], old_stack: &[u128], new_stack: &[u128], op_flag: u128)
{
    for i in 1..result.len() {
        result.agg_constraint(i, op_flag, are_equal(old_stack[i - 1], new_stack[i]));
    }
}

fn enforce_stack_copy(result: &mut [u128], old_stack: &[u128], new_stack: &[u128], op_flag: u128)
{    
    for i in 0..result.len() {
        result.agg_constraint(i, op_flag, are_equal(old_stack[i], new_stack[i]));
    }
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    
    use crate::math::{ field };
    use super::{ TraceState };

    #[test]
    fn op_begin() {

        // correct transition, context depth = 1
        let state1 = TraceState::from_vec(1, 0, 1, &vec![3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  0,  11]);
        let state2 = TraceState::from_vec(1, 0, 1, &vec![0, 0, 0, 0,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  3,  11]);

        let mut evaluations = vec![0; 5];
        super::enforce_begin(&mut evaluations, &state1, &state2, 1);
        assert_eq!(vec![0, 0, 0, 0, 0], evaluations);

        // correct transition, context depth = 2
        let state1 = TraceState::from_vec(2, 0, 1, &vec![3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  2, 0,  11]);
        let state2 = TraceState::from_vec(2, 0, 1, &vec![0, 0, 0, 0,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  3, 2,  11]);

        let mut evaluations = vec![0; 6];
        super::enforce_begin(&mut evaluations, &state1, &state2, 1);
        assert_eq!(vec![0, 0, 0, 0, 0, 0], evaluations);

        // incorrect transition, context depth = 1
        let state1 = TraceState::from_vec(1, 0, 1, &vec![3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  0, 11]);
        let state2 = TraceState::from_vec(1, 0, 1, &vec![1, 2, 3, 4,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  5, 11]);

        let mut evaluations = vec![0; 5];
        super::enforce_begin(&mut evaluations, &state1, &state2, 1);
        assert_eq!(vec![1, 2, 3, 4, field::sub(3, 5)], evaluations);

        // incorrect transition, context depth = 2
        let state1 = TraceState::from_vec(2, 0, 1, &vec![3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  2, 0,  11]);
        let state2 = TraceState::from_vec(2, 0, 1, &vec![1, 2, 3, 4,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  5, 6,  11]);

        let mut evaluations = vec![0; 6];
        super::enforce_begin(&mut evaluations, &state1, &state2, 1);
        assert_eq!(vec![1, 2, 3, 4, field::sub(3, 5), field::sub(2, 6)], evaluations);
    }

    #[test]
    fn op_tend() {

        // correct transition, context depth = 1
        let state1 = TraceState::from_vec(1, 0, 1, &vec![3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  8,  11]);
        let state2 = TraceState::from_vec(1, 0, 1, &vec![8, 3, 4, 0,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  0,  11]);

        let mut evaluations = vec![0; 5];
        super::enforce_tend(&mut evaluations, &state1, &state2, 1);
        assert_eq!(vec![0, 0, 0, 0, 0], evaluations);

        // correct transition, context depth = 2
        let state1 = TraceState::from_vec(2, 0, 1, &vec![3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  8, 2,  11]);
        let state2 = TraceState::from_vec(2, 0, 1, &vec![8, 3, 6, 0,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  2, 0,  11]);

        let mut evaluations = vec![0; 6];
        super::enforce_tend(&mut evaluations, &state1, &state2, 1);
        assert_eq!(vec![0, 0, 0, 0, 0, 0], evaluations);

        // incorrect transition, context depth = 1
        let state1 = TraceState::from_vec(1, 0, 1, &vec![3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  8, 11]);
        let state2 = TraceState::from_vec(1, 0, 1, &vec![1, 2, 3, 4,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  8, 11]);

        let mut evaluations = vec![0; 5];
        super::enforce_tend(&mut evaluations, &state1, &state2, 1);
        assert_eq!(vec![7, 1, 0, 4, 8], evaluations);

        // incorrect transition, context depth = 2
        let state1 = TraceState::from_vec(2, 0, 1, &vec![3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  4, 6,  11]);
        let state2 = TraceState::from_vec(2, 0, 1, &vec![1, 2, 3, 4,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  5, 6,  11]);

        let mut evaluations = vec![0; 6];
        super::enforce_tend(&mut evaluations, &state1, &state2, 1);
        assert_eq!(vec![3, 1, 0, 4, 1, 6], evaluations);
    }

    #[test]
    fn op_fend() {

        // correct transition, context depth = 1
        let state1 = TraceState::from_vec(1, 0, 1, &vec![3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  8,  11]);
        let state2 = TraceState::from_vec(1, 0, 1, &vec![8, 4, 3, 0,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  0,  11]);

        let mut evaluations = vec![0; 5];
        super::enforce_fend(&mut evaluations, &state1, &state2, 1);
        assert_eq!(vec![0, 0, 0, 0, 0], evaluations);

        // correct transition, context depth = 2
        let state1 = TraceState::from_vec(2, 0, 1, &vec![3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  8, 2,  11]);
        let state2 = TraceState::from_vec(2, 0, 1, &vec![8, 6, 3, 0,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  2, 0,  11]);

        let mut evaluations = vec![0; 6];
        super::enforce_fend(&mut evaluations, &state1, &state2, 1);
        assert_eq!(vec![0, 0, 0, 0, 0, 0], evaluations);

        // incorrect transition, context depth = 1
        let state1 = TraceState::from_vec(1, 0, 1, &vec![3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  8, 11]);
        let state2 = TraceState::from_vec(1, 0, 1, &vec![1, 3, 2, 4,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  8, 11]);

        let mut evaluations = vec![0; 5];
        super::enforce_fend(&mut evaluations, &state1, &state2, 1);
        assert_eq!(vec![7, 0, 1, 4, 8], evaluations);

        // incorrect transition, context depth = 2
        let state1 = TraceState::from_vec(2, 0, 1, &vec![3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  4, 6,  11]);
        let state2 = TraceState::from_vec(2, 0, 1, &vec![1, 6, 2, 4,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  5, 6,  11]);

        let mut evaluations = vec![0; 6];
        super::enforce_fend(&mut evaluations, &state1, &state2, 1);
        assert_eq!(vec![3, 0, 1, 4, 1, 6], evaluations);
    }

    #[test]
    fn op_loop() {
        // TODO
    }

    #[test]
    fn op_wrap() {
        // TODO
    }

    #[test]
    fn op_break() {
        // TODO
    }

    #[test]
    fn op_void() {

        // correct transition, context depth = 1
        let state1 = TraceState::from_vec(1, 0, 1, &vec![3, 5, 7, 9,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  8,  11]);
        let state2 = TraceState::from_vec(1, 0, 1, &vec![3, 5, 7, 9,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  8,  11]);

        let mut evaluations = vec![0; 5];
        super::enforce_void(&mut evaluations, &state1, &state2, 1);
        assert_eq!(vec![0, 0, 0, 0, 0], evaluations);

        // correct transition, context depth = 2
        let state1 = TraceState::from_vec(2, 0, 1, &vec![3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  8, 2,  11]);
        let state2 = TraceState::from_vec(2, 0, 1, &vec![3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  8, 2,  11]);

        let mut evaluations = vec![0; 6];
        super::enforce_void(&mut evaluations, &state1, &state2, 1);
        assert_eq!(vec![0, 0, 0, 0, 0, 0], evaluations);

        // incorrect transition, context depth = 1
        let state1 = TraceState::from_vec(1, 0, 1, &vec![3, 5, 7, 9,  1, 0, 0,  1, 1, 1, 1, 1,  1, 1,  8, 11]);
        let state2 = TraceState::from_vec(1, 0, 1, &vec![2, 4, 6, 8,  1, 1, 1,  1, 1, 1, 1, 1,  1, 1,  7, 11]);

        let mut evaluations = vec![0; 5];
        super::enforce_void(&mut evaluations, &state1, &state2, 1);
        assert_eq!(vec![1, 1, 1, 1, 1], evaluations);
    }
}