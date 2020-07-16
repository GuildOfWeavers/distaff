use crate::math::{ field::{ mul } };
use crate::processor::opcodes2::{ FlowOps };
use crate::stark::trace::{ trace_state2::TraceState };
use super::utils::{ are_equal, is_zero, is_binary, binary_not, EvaluationResult };

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================
const NUM_OP_CONSTRAINTS: usize = 20;
const OP_CONSTRAINT_DEGREES: [usize; NUM_OP_CONSTRAINTS] = [
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2,   // all op bits are binary
    7,                              // ld_ops and hd_ops cannot be all 0s
    8,                              // when cf_ops are not all 0s, ld_ops and hd_ops must be all 1s
    3,                              // PUSH is allowed only on multiples of 8
    2,                              // VOID can be followed only by VOID
    4, 4, 4, 4, 4, 4,               // cf_ops are aligned correctly
];

const NUM_SPONGE_CONSTRAINTS: usize = 4;
const SPONGE_CONSTRAINT_DEGREES: [usize; NUM_SPONGE_CONSTRAINTS] = [
    6, 7, 6, 6,                     // sponge transition constraints
];

const STACK_CONSTRAINT_DEGREE: usize = 4;


// TYPES AND INTERFACES
// ================================================================================================
pub struct Decoder {
    constraint_degrees  : Vec<usize>
}

// DECODER CONSTRAINT EVALUATOR IMPLEMENTATION
// ================================================================================================
impl Decoder {

    fn new(ctx_depth: usize, loop_depth: usize) -> Decoder {

        let mut degrees = Vec::from(&OP_CONSTRAINT_DEGREES[..]);
        degrees.extend_from_slice(&SPONGE_CONSTRAINT_DEGREES[..]);
        degrees.resize(degrees.len() + ctx_depth + loop_depth, STACK_CONSTRAINT_DEGREE);

        return Decoder {
            constraint_degrees: degrees,
        };
    }

    pub fn constraint_degrees(&self) -> &[usize] {
        return &self.constraint_degrees;
    }

    // EVALUATOR FUNCTIONS
    // --------------------------------------------------------------------------------------------

    pub fn evaluate(&self, current: &TraceState, next: &TraceState, step: usize, result: &mut [u128]) {

        // evaluate constraints for decoding op codes
        self.check_op_bits(current, next, &mut result[..NUM_OP_CONSTRAINTS]);

        let result = &mut result[NUM_OP_CONSTRAINTS..];
        let op_flags = current.cf_op_flags();
        enforce_begin(result, current, next, op_flags[FlowOps::Begin as usize]);
        enforce_tend (result, current, next, op_flags[FlowOps::Tend as usize]);
        enforce_fend (result, current, next, op_flags[FlowOps::Fend as usize]);
    }


    fn check_op_bits(&self, current: &TraceState, next: &TraceState, result: &mut [u128]) {

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
}

// CONTROL FLOW OPERATIONS
// ================================================================================================
fn enforce_begin(result: &mut [u128], current: &TraceState, next: &TraceState, op_flag: u128) {

    // make sure sponge state has been cleared
    let next_sponge = next.sponge();
    result.agg_constraint(0, op_flag, is_zero(next_sponge[0]));
    result.agg_constraint(1, op_flag, is_zero(next_sponge[1]));
    result.agg_constraint(2, op_flag, is_zero(next_sponge[2]));
    result.agg_constraint(3, op_flag, is_zero(next_sponge[3]));

    // make sure hash of parent block was pushed onto the context stack
    let parent_hash = current.sponge()[0];
    let ctx_stack_end = 4 + current.ctx_stack().len();
    let ctx_result = &mut result[4..ctx_stack_end];
    enforce_stack_push(ctx_result, current.ctx_stack(), next.ctx_stack(), parent_hash, op_flag);

    // make sure loop stack didn't change
    let loop_result = &mut result[ctx_stack_end..ctx_stack_end + current.loop_stack().len()];
    enforce_stack_copy(loop_result, current.loop_stack(), next.loop_stack(), op_flag);
}

fn enforce_tend(result: &mut [u128], current: &TraceState, next: &TraceState, op_flag: u128)
{
    let parent_hash = current.ctx_stack()[0];
    let block_hash = current.sponge()[0];

    let next_sponge = next.sponge();
    result.agg_constraint(0, op_flag, are_equal(parent_hash, next_sponge[0]));
    result.agg_constraint(1, op_flag, are_equal(block_hash, next_sponge[1]));
    // no constraint on the 3rd element of the sponge
    result.agg_constraint(3, op_flag, is_zero(next_sponge[3]));

    // make parent hash was popped from context stack
    let ctx_stack_end = 4 + current.ctx_stack().len();
    let ctx_result = &mut result[4..ctx_stack_end];
    enforce_stack_pop(ctx_result, current.ctx_stack(), next.ctx_stack(), op_flag);

    // make sure loop stack didn't change
    let loop_result = &mut result[ctx_stack_end..ctx_stack_end + current.loop_stack().len()];
    enforce_stack_copy(loop_result, current.loop_stack(), next.loop_stack(), op_flag);
}

fn enforce_fend(result: &mut [u128], current: &TraceState, next: &TraceState, op_flag: u128)
{
    let parent_hash = current.ctx_stack()[0];
    let block_hash = current.sponge()[0];

    let next_sponge = next.sponge();
    result.agg_constraint(0, op_flag, are_equal(parent_hash, next_sponge[0]));
    // no constraint on the 2nd element of the sponge
    result.agg_constraint(2, op_flag, are_equal(block_hash, next_sponge[2]));
    result.agg_constraint(3, op_flag, is_zero(next_sponge[3]));

    // make sure parent hash was popped from context stack
    let ctx_stack_end = 4 + current.ctx_stack().len();
    let ctx_result = &mut result[4..ctx_stack_end];
    enforce_stack_pop(ctx_result, current.ctx_stack(), next.ctx_stack(), op_flag);

    // make sure loop stack didn't change
    let loop_result = &mut result[ctx_stack_end..ctx_stack_end + current.loop_stack().len()];
    enforce_stack_copy(loop_result, current.loop_stack(), next.loop_stack(), op_flag);
}

// HELPER FUNCTIONS
// ================================================================================================

fn enforce_stack_pop(result: &mut [u128], old_stack: &[u128], new_stack: &[u128], op_flag: u128)
{
    let last_idx = result.len() - 1;
    for i in 0..last_idx {
        result.agg_constraint(i, op_flag, are_equal(old_stack[i + 1], new_stack[i]));
    }

    result.agg_constraint(last_idx, op_flag, is_zero(new_stack[last_idx]));
}

fn enforce_stack_push(result: &mut [u128], old_stack: &[u128], new_stack: &[u128], push_value: u128, op_flag: u128)
{
    result.agg_constraint(0, op_flag, are_equal(push_value, new_stack[0]));
    
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