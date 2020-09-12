use std::{ fmt, cmp };
use crate::{
    math::field,
    processor::opcodes::{ FlowOps, UserOps },
    PROGRAM_DIGEST_SIZE,
    MIN_STACK_DEPTH, MIN_CONTEXT_DEPTH, MIN_LOOP_DEPTH,
    OP_COUNTER_IDX, SPONGE_WIDTH, SPONGE_RANGE,
    NUM_FLOW_OP_BITS, NUM_USER_OP_BITS,
    FLOW_OP_BITS_RANGE, USER_OP_BITS_RANGE,
};
use super::op_flags::OpFlags;

// CONSTANTS
// ================================================================================================
const NUM_OP_BITS: usize = NUM_FLOW_OP_BITS + NUM_USER_OP_BITS;
const NUM_STATIC_DECODER_REGISTERS: usize = 1 + SPONGE_WIDTH + NUM_OP_BITS; // 1 is for op_counter

// TYPES AND INTERFACES
// ================================================================================================
#[derive(PartialEq)]
pub struct TraceState {
    op_counter  : u128,
    sponge      : [u128; SPONGE_WIDTH],
    flow_op_bits: [u128; NUM_FLOW_OP_BITS],
    user_op_bits: [u128; NUM_USER_OP_BITS],
    ctx_stack   : Vec<u128>,
    loop_stack  : Vec<u128>,
    user_stack  : Vec<u128>,

    ctx_depth   : usize,
    loop_depth  : usize,
    stack_depth : usize,

    op_flags    : OpFlags,
}

// TRACE STATE IMPLEMENTATION
// ================================================================================================
impl TraceState {

    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------

    pub fn new(ctx_depth: usize, loop_depth: usize, stack_depth: usize) -> TraceState {
        
        return TraceState {
            op_counter  : 0,
            sponge      : [0; SPONGE_WIDTH],
            flow_op_bits: [0; NUM_FLOW_OP_BITS],
            user_op_bits: [0; NUM_USER_OP_BITS],
            ctx_stack   : vec![0; cmp::max(ctx_depth, MIN_CONTEXT_DEPTH)],
            loop_stack  : vec![0; cmp::max(loop_depth, MIN_LOOP_DEPTH)],
            user_stack  : vec![0; cmp::max(stack_depth, MIN_STACK_DEPTH)],
            ctx_depth   : ctx_depth,
            loop_depth  : loop_depth,
            stack_depth : stack_depth,
            op_flags    : OpFlags::new(),
        };
    }

    pub fn from_vec(ctx_depth: usize, loop_depth: usize, stack_depth: usize, state: &Vec<u128>) -> TraceState {

        let op_counter = state[OP_COUNTER_IDX];

        let mut sponge = [0; SPONGE_WIDTH];
        sponge.copy_from_slice(&state[SPONGE_RANGE]);

        let mut flow_op_bits = [0; NUM_FLOW_OP_BITS];
        flow_op_bits.copy_from_slice(&state[FLOW_OP_BITS_RANGE]);

        let mut user_op_bits = [0; NUM_USER_OP_BITS];
        user_op_bits.copy_from_slice(&state[USER_OP_BITS_RANGE]);

        let mut ctx_stack = vec![0; cmp::max(ctx_depth, MIN_CONTEXT_DEPTH)];
        let ctx_stack_end = USER_OP_BITS_RANGE.end + ctx_depth;
        ctx_stack[..ctx_depth].copy_from_slice(&state[USER_OP_BITS_RANGE.end..ctx_stack_end]);

        let mut loop_stack = vec![0; cmp::max(loop_depth, MIN_LOOP_DEPTH)];
        let loop_stack_end = ctx_stack_end + loop_depth;
        loop_stack[..loop_depth].copy_from_slice(&state[ctx_stack_end..loop_stack_end]);

        let mut user_stack = vec![0; cmp::max(stack_depth, MIN_STACK_DEPTH)];
        user_stack[..stack_depth].copy_from_slice(&state[loop_stack_end..]);

        let mut state = TraceState {
            op_counter, sponge,
            flow_op_bits, user_op_bits,
            ctx_stack, loop_stack, user_stack,
            ctx_depth, loop_depth, stack_depth,
            op_flags: OpFlags::new()
        };
        state.op_flags.update(&state.flow_op_bits, &state.user_op_bits);

        return state;
    }

    // STATIC FUNCTIONS
    // --------------------------------------------------------------------------------------------
    pub fn compute_decoder_width(ctx_depth: usize, loop_depth: usize) -> usize {
        return NUM_STATIC_DECODER_REGISTERS + ctx_depth + loop_depth;
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------
    pub fn width(&self) -> usize {
        return USER_OP_BITS_RANGE.end + self.ctx_depth + self.loop_depth + self.stack_depth;
    }

    pub fn stack_depth(&self) -> usize {
        return self.stack_depth;
    }

    // OPERATION COUNTER
    // --------------------------------------------------------------------------------------------
    pub fn op_counter(&self) -> u128 {
        return self.op_counter;
    }

    #[cfg(test)]
    pub fn set_op_counter(&mut self, value: u128) {
        self.op_counter = value;
    }

    // SPONGE
    // --------------------------------------------------------------------------------------------
    pub fn sponge(&self) -> &[u128] {
        return &self.sponge;
    }

    pub fn program_hash(&self) -> &[u128] {
        return &self.sponge[..PROGRAM_DIGEST_SIZE];
    }

    // OP BITS
    // --------------------------------------------------------------------------------------------
    pub fn flow_op_bits(&self) -> &[u128] {
        return &self.flow_op_bits;
    }

    pub fn user_op_bits(&self) -> &[u128] {
        return &self.user_op_bits;
    }

    pub fn op_code(&self) -> u128 {
        let mut result = self.user_op_bits[0];
        result = field::add(result, field::mul(self.user_op_bits[1], 2));
        result = field::add(result, field::mul(self.user_op_bits[2], 4));
        result = field::add(result, field::mul(self.user_op_bits[3], 8));
        result = field::add(result, field::mul(self.user_op_bits[4], 16));
        result = field::add(result, field::mul(self.user_op_bits[5], 32));
        return result;
    }

    pub fn set_op_bits(&mut self, bits: [u128; NUM_OP_BITS]) {
        self.flow_op_bits.copy_from_slice(&bits[..3]);
        self.user_op_bits.copy_from_slice(&bits[3..9]);
        self.op_flags.update(&self.flow_op_bits, &self.user_op_bits);
    }

    // OP FLAGS
    // --------------------------------------------------------------------------------------------
    pub fn get_flow_op_flags(&self, opcode: FlowOps) -> u128 {
        return self.op_flags.get_flow_op_flag(opcode);
    }

    pub fn get_user_op_flag(&self, opcode: UserOps) -> u128 {
        return self.op_flags.get_user_op_flag(opcode);
    }

    // STACKS
    // --------------------------------------------------------------------------------------------
    pub fn ctx_stack(&self) -> &[u128] {
        return &self.ctx_stack;
    }

    pub fn loop_stack(&self) -> &[u128] {
        return &self.loop_stack;
    }

    pub fn user_stack(&self) -> &[u128] {
        return &self.user_stack;
    }

    // RAW STATE
    // --------------------------------------------------------------------------------------------
    pub fn to_vec(&self) -> Vec<u128> {
        let mut result = Vec::with_capacity(self.width());
        result.push(self.op_counter);
        result.extend_from_slice(&self.sponge);
        result.extend_from_slice(&self.flow_op_bits);
        result.extend_from_slice(&self.user_op_bits);
        result.extend_from_slice(&self.ctx_stack[..self.ctx_depth]);
        result.extend_from_slice(&self.loop_stack[..self.loop_depth]);
        result.extend_from_slice(&self.user_stack[..self.stack_depth]);
        return result;
    }

    pub fn update_from_trace(&mut self, trace: &Vec<Vec<u128>>, step: usize) {

        self.op_counter = trace[OP_COUNTER_IDX][step];

        for (i, j) in SPONGE_RANGE.enumerate()       { self.sponge[i] = trace[j][step]; }
        for (i, j) in FLOW_OP_BITS_RANGE.enumerate() { self.flow_op_bits[i] = trace[j][step]; }
        for (i, j) in USER_OP_BITS_RANGE.enumerate() { self.user_op_bits[i] = trace[j][step]; }

        let ctx_stack_start = USER_OP_BITS_RANGE.end;
        let ctx_stack_end = ctx_stack_start + self.ctx_depth;
        for (i, j) in (ctx_stack_start..ctx_stack_end).enumerate() {
            self.ctx_stack[i] = trace[j][step];
        }

        let loop_stack_end = ctx_stack_end + self.loop_depth;
        for (i, j) in (ctx_stack_end..loop_stack_end).enumerate() {
            self.loop_stack[i] = trace[j][step];
        }

        let user_stack_end = loop_stack_end + self.stack_depth;
        for (i, j) in (loop_stack_end..user_stack_end).enumerate() {
            self.user_stack[i] = trace[j][step];
        }
        
        self.op_flags.update(&self.flow_op_bits, &self.user_op_bits);
    }
}

impl fmt::Debug for TraceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:>4}] {:>32X?} {:?} {:?} {:>32X?} {:>32X?} {:?}",
            self.op_counter,
            self.sponge, 
            self.flow_op_bits,
            self.user_op_bits,
            self.ctx_stack,
            self.loop_stack,
            self.user_stack
        )
    }
}

impl fmt::Display for TraceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:>4}] {:>16X?} {:?} {:?} {:>16X?} {:>16X?} {:?}",
            self.op_counter,
            self.sponge.iter().map(|x| x >> 64).collect::<Vec<u128>>(),
            self.flow_op_bits,
            self.user_op_bits,
            self.ctx_stack.iter().map(|x| x >> 64).collect::<Vec<u128>>(),
            self.loop_stack.iter().map(|x| x >> 64).collect::<Vec<u128>>(),
            &self.user_stack[..self.stack_depth]
        )
    }
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {

    use super::{ TraceState };

    #[test]
    fn from_vec() {

        // empty context and loop stacks
        let state = TraceState::from_vec(0, 0, 2, &vec![
            101,  1, 2, 3, 4,  5, 6, 7,  8, 9, 10, 11, 12, 13,  14, 15
        ]);

        assert_eq!(101, state.op_counter());
        assert_eq!([1, 2, 3, 4], state.sponge());
        assert_eq!([5, 6, 7], state.flow_op_bits());
        assert_eq!([8, 9, 10, 11, 12, 13], state.user_op_bits());
        assert_eq!([0], state.ctx_stack());
        assert_eq!([0], state.loop_stack());
        assert_eq!([14, 15, 0, 0, 0, 0, 0, 0], state.user_stack());
        assert_eq!(16, state.width());
        assert_eq!(2, state.stack_depth());
        assert_eq!(vec![
            101, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
        ], state.to_vec());

        // 1 item on context stack, empty loop stack
        let state = TraceState::from_vec(1, 0, 2, &vec![
            101,  1, 2, 3, 4,  5, 6, 7,  8, 9, 10, 11, 12, 13,  14,  15, 16
        ]);

        assert_eq!(101, state.op_counter());
        assert_eq!([1, 2, 3, 4], state.sponge());
        assert_eq!([5, 6, 7], state.flow_op_bits());
        assert_eq!([8, 9, 10, 11, 12, 13], state.user_op_bits());
        assert_eq!([14], state.ctx_stack());
        assert_eq!([0], state.loop_stack());
        assert_eq!([15, 16, 0, 0, 0, 0, 0, 0], state.user_stack());
        assert_eq!(17, state.width());
        assert_eq!(2, state.stack_depth());
        assert_eq!(vec![
            101, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
        ], state.to_vec());

        // non-empty loop stack
        let state = TraceState::from_vec(2, 1, 9, &vec![
            101,  1, 2, 3, 4,  5, 6, 7,  8, 9, 10, 11, 12, 13,  14, 15,  16,
            17, 18, 19, 20, 21, 22, 23, 24, 25
        ]);

        assert_eq!(101, state.op_counter());
        assert_eq!([1, 2, 3, 4], state.sponge());
        assert_eq!([5, 6, 7], state.flow_op_bits());
        assert_eq!([8, 9, 10, 11, 12, 13], state.user_op_bits());
        assert_eq!([14, 15], state.ctx_stack());
        assert_eq!([16], state.loop_stack());
        assert_eq!([17, 18, 19, 20, 21, 22, 23, 24, 25], state.user_stack());
        assert_eq!(26, state.width());
        assert_eq!(9, state.stack_depth());
        assert_eq!(vec![
            101, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20, 21, 22, 23, 24, 25,
        ], state.to_vec());
    }

    #[test]
    fn update_from_trace() {
        let data = vec![
            101,  1, 2, 3, 4,  5, 6, 7,  8, 9, 10, 11, 12, 13,  14, 15,  16,  17, 18, 19
        ];
        let mut trace = Vec::with_capacity(data.len());
        for i in 0..data.len() {
            trace.push(vec![0, data[i], 0]);
        }

        // first row
        let mut state = TraceState::new(2, 1, 3);
        state.update_from_trace(&trace, 0);

        assert_eq!(0, state.op_counter());
        assert_eq!([0, 0, 0, 0], state.sponge());
        assert_eq!([0, 0, 0], state.flow_op_bits());
        assert_eq!([0, 0, 0, 0, 0, 0], state.user_op_bits());
        assert_eq!([0, 0], state.ctx_stack());
        assert_eq!([0], state.loop_stack());
        assert_eq!([0, 0, 0, 0, 0, 0, 0, 0], state.user_stack());
        assert_eq!(20, state.width());
        assert_eq!(3, state.stack_depth());

        // second row
        state.update_from_trace(&trace, 1);

        assert_eq!(101, state.op_counter());
        assert_eq!([1, 2, 3, 4], state.sponge());
        assert_eq!([5, 6, 7], state.flow_op_bits());
        assert_eq!([8, 9, 10, 11, 12, 13], state.user_op_bits());
        assert_eq!([14, 15], state.ctx_stack());
        assert_eq!([16], state.loop_stack());
        assert_eq!([17, 18, 19, 0, 0, 0, 0, 0], state.user_stack());
        assert_eq!(20, state.width());
        assert_eq!(3, state.stack_depth());
    }

    #[test]
    fn op_code() {
        let state = TraceState::from_vec(1, 0, 2, &vec![
            101,  1, 2, 3, 4,  1, 1, 1,  0, 0, 0, 0, 0, 0,  14, 15, 16
        ]);
        assert_eq!(0, state.op_code());

        let state = TraceState::from_vec(1, 0, 2, &vec![
            101,  1, 2, 3, 4,  1, 1, 1,  1, 1, 1, 1, 1, 1,  14, 15, 16
        ]);
        assert_eq!(63, state.op_code());

        let state = TraceState::from_vec(1, 0, 2, &vec![
            101,  1, 2, 3, 4,  1, 1, 1,  1, 1, 1, 1, 0, 0,  14, 15, 16
        ]);
        assert_eq!(15, state.op_code());
    }
}