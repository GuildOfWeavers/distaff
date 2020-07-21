use std::{ fmt, cmp };
use crate::{
    math::field,
    OpCode,
    MIN_STACK_DEPTH,
    PROGRAM_DIGEST_SIZE,
    OP_COUNTER_IDX, SPONGE_WIDTH, SPONGE_RANGE,
    NUM_CF_OPS, NUM_LD_OPS, NUM_HD_OPS,
    NUM_CF_OP_BITS, NUM_LD_OP_BITS, NUM_HD_OP_BITS,
    CF_OP_BITS_RANGE, LD_OP_BITS_RANGE, HD_OP_BITS_RANGE,
};

// CONSTANTS
// ================================================================================================
const NUM_OP_BITS: usize = NUM_CF_OP_BITS + NUM_LD_OP_BITS + NUM_HD_OP_BITS;
const NUM_STATIC_DECODER_REGISTERS: usize = 1 + SPONGE_WIDTH + NUM_OP_BITS; // 1 is for op_counter

// TYPES AND INTERFACES
// ================================================================================================
#[derive(PartialEq)]
pub struct TraceState {
    op_counter  : u128,
    sponge      : [u128; SPONGE_WIDTH],
    cf_op_bits  : [u128; NUM_CF_OP_BITS],
    ld_op_bits  : [u128; NUM_LD_OP_BITS],
    hd_op_bits  : [u128; NUM_HD_OP_BITS],
    ctx_stack   : Vec<u128>,
    loop_stack  : Vec<u128>,
    user_stack  : Vec<u128>,
    stack_depth : usize,

    cf_op_flags : [u128; NUM_CF_OPS],
    ld_op_flags : [u128; NUM_LD_OPS],
    hd_op_flags : [u128; NUM_HD_OPS],
    op_flags_set: bool,
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
            cf_op_bits  : [0; NUM_CF_OP_BITS],
            ld_op_bits  : [0; NUM_LD_OP_BITS],
            hd_op_bits  : [0; NUM_HD_OP_BITS],
            ctx_stack   : vec![0; ctx_depth],
            loop_stack  : vec![0; loop_depth],
            user_stack  : vec![0; cmp::max(stack_depth, MIN_STACK_DEPTH)],
            stack_depth : stack_depth,
            cf_op_flags : [0; NUM_CF_OPS],
            ld_op_flags : [0; NUM_LD_OPS],
            hd_op_flags : [0; NUM_HD_OPS],
            op_flags_set: false,
        };
    }

    pub fn from_vec(ctx_depth: usize, loop_depth: usize, stack_depth: usize, state: &Vec<u128>) -> TraceState {

        let op_counter = state[OP_COUNTER_IDX];

        let mut sponge = [0; SPONGE_WIDTH];
        sponge.copy_from_slice(&state[SPONGE_RANGE]);

        let mut cf_op_bits = [0; NUM_CF_OP_BITS];
        cf_op_bits.copy_from_slice(&state[CF_OP_BITS_RANGE]);

        let mut ld_op_bits = [0; NUM_LD_OP_BITS];
        ld_op_bits.copy_from_slice(&state[LD_OP_BITS_RANGE]);

        let mut hd_op_bits = [0; NUM_HD_OP_BITS];
        hd_op_bits.copy_from_slice(&state[HD_OP_BITS_RANGE]);

        let mut ctx_stack = vec![0; ctx_depth];
        let ctx_stack_end = HD_OP_BITS_RANGE.end + ctx_depth;
        ctx_stack.copy_from_slice(&state[HD_OP_BITS_RANGE.end..ctx_stack_end]);

        let mut loop_stack = vec![0; loop_depth];
        let loop_stack_end = ctx_stack_end + loop_depth;
        loop_stack.copy_from_slice(&state[ctx_stack_end..loop_stack_end]);

        let mut user_stack = vec![0; cmp::max(stack_depth, MIN_STACK_DEPTH)];
        user_stack[..stack_depth].copy_from_slice(&state[loop_stack_end..]);

        return TraceState {
            op_counter, sponge,
            cf_op_bits, ld_op_bits, hd_op_bits,
            ctx_stack, loop_stack, user_stack, stack_depth,
            cf_op_flags : [0; NUM_CF_OPS],
            ld_op_flags : [0; NUM_LD_OPS],
            hd_op_flags : [0; NUM_HD_OPS],
            op_flags_set: false,
        };
    }

    // STATIC FUNCTIONS
    // --------------------------------------------------------------------------------------------
    pub fn compute_decoder_width(ctx_depth: usize, loop_depth: usize) -> usize {
        return NUM_STATIC_DECODER_REGISTERS + ctx_depth + loop_depth;
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------
    pub fn width(&self) -> usize {
        return HD_OP_BITS_RANGE.end + self.ctx_stack.len() + self.loop_stack.len() + self.stack_depth;
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
    pub fn cf_op_bits(&self) -> &[u128] {
        return &self.cf_op_bits;
    }

    pub fn ld_op_bits(&self) -> &[u128] {
        return &self.ld_op_bits;
    }

    pub fn hd_op_bits(&self) -> &[u128] {
        return &self.hd_op_bits;
    }

    pub fn op_code(&self) -> u128 {
        let mut result = self.ld_op_bits[0];
        result = field::add(result, field::mul(self.ld_op_bits[1], 2));
        result = field::add(result, field::mul(self.ld_op_bits[2], 4));
        result = field::add(result, field::mul(self.ld_op_bits[3], 8));
        result = field::add(result, field::mul(self.ld_op_bits[4], 16));
        result = field::add(result, field::mul(self.hd_op_bits[0], 32));
        result = field::add(result, field::mul(self.hd_op_bits[1], 64));
        return result;
    }

    pub fn set_op_bits(&mut self, bits: [u128; NUM_OP_BITS]) {
        self.cf_op_bits.copy_from_slice(&bits[..3]);
        self.ld_op_bits.copy_from_slice(&bits[3..8]);
        self.hd_op_bits.copy_from_slice(&bits[8..]);
    }

    // OP FLAGS
    // --------------------------------------------------------------------------------------------
    pub fn cf_op_flags(&self) -> [u128; NUM_CF_OPS] {
        if !self.op_flags_set {
            unsafe {
                let mutable_self = &mut *(self as *const _ as *mut TraceState);
                mutable_self.set_op_flags();
            }
        }
        return self.cf_op_flags;
    }

    pub fn ld_op_flags(&self) -> [u128; NUM_LD_OPS] {
        if !self.op_flags_set {
            unsafe {
                let mutable_self = &mut *(self as *const _ as *mut TraceState);
                mutable_self.set_op_flags();
            }
        }
        return self.ld_op_flags;
    }

    pub fn hd_op_flags(&self) -> [u128; NUM_HD_OPS] {
        if !self.op_flags_set {
            unsafe {
                let mutable_self = &mut *(self as *const _ as *mut TraceState);
                mutable_self.set_op_flags();
            }
        }
        return self.hd_op_flags;
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
        result.extend_from_slice(&self.cf_op_bits);
        result.extend_from_slice(&self.ld_op_bits);
        result.extend_from_slice(&self.hd_op_bits);
        result.extend_from_slice(&self.ctx_stack);
        result.extend_from_slice(&self.loop_stack);
        result.extend_from_slice(&self.user_stack[..self.stack_depth]);
        return result;
    }

    pub fn update_from_trace(&mut self, trace: &Vec<Vec<u128>>, step: usize) {

        self.op_counter = trace[OP_COUNTER_IDX][step];

        for (i, j) in SPONGE_RANGE.enumerate()     { self.sponge[i] = trace[j][step]; }
        for (i, j) in CF_OP_BITS_RANGE.enumerate() { self.cf_op_bits[i] = trace[j][step]; }
        for (i, j) in LD_OP_BITS_RANGE.enumerate() { self.ld_op_bits[i] = trace[j][step]; }
        for (i, j) in HD_OP_BITS_RANGE.enumerate() { self.hd_op_bits[i] = trace[j][step]; }

        let ctx_stack_start = HD_OP_BITS_RANGE.end;
        let ctx_stack_end = ctx_stack_start + self.ctx_stack.len();
        for (i, j) in (ctx_stack_start..ctx_stack_end).enumerate() {
            self.ctx_stack[i] = trace[j][step];
        }

        let loop_stack_end = ctx_stack_end + self.loop_stack.len();
        for (i, j) in (ctx_stack_end..loop_stack_end).enumerate() {
            self.loop_stack[i] = trace[j][step];
        }

        let user_stack_end = loop_stack_end + self.stack_depth;
        for (i, j) in (loop_stack_end..user_stack_end).enumerate() {
            self.user_stack[i] = trace[j][step];
        }
        
        self.op_flags_set = false;
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------
    fn set_op_flags(&mut self) {

        // set control flow flags
        let not_0 = binary_not(self.cf_op_bits[0]);
        let not_1 = binary_not(self.cf_op_bits[1]);
        self.cf_op_flags[0] = field::mul(not_0, not_1);
        self.cf_op_flags[1] = field::mul(self.cf_op_bits[0], not_1);
        self.cf_op_flags[2] = field::mul(not_0, self.cf_op_bits[1]);
        self.cf_op_flags[3] = field::mul(self.cf_op_bits[0], self.cf_op_bits[1]);
        self.cf_op_flags.copy_within(0..4, 4);

        let not_2 = binary_not(self.cf_op_bits[2]);
        for i in 0..4 { self.cf_op_flags[i] = field::mul(self.cf_op_flags[i], not_2); }
        for i in 4..8 { self.cf_op_flags[i] = field::mul(self.cf_op_flags[i], self.cf_op_bits[2]); }

        // set low-degree operation flags
        let not_0 = binary_not(self.ld_op_bits[0]);
        let not_1 = binary_not(self.ld_op_bits[1]);
        self.ld_op_flags[0] = field::mul(not_0, not_1);
        self.ld_op_flags[1] = field::mul(self.ld_op_bits[0], not_1);
        self.ld_op_flags[2] = field::mul(not_0, self.cf_op_bits[1]);
        self.ld_op_flags[3] = field::mul(self.ld_op_bits[0], self.ld_op_bits[1]);
        self.ld_op_flags.copy_within(0..4, 4);

        let not_2 = binary_not(self.ld_op_bits[2]);
        for i in 0..4 { self.ld_op_flags[i] = field::mul(self.ld_op_flags[i], not_2); }
        for i in 4..8 { self.ld_op_flags[i] = field::mul(self.ld_op_flags[i], self.ld_op_bits[2]); }
        self.ld_op_flags.copy_within(0..8, 8);

        let not_3 = binary_not(self.ld_op_bits[3]);
        for i in 0..8  { self.ld_op_flags[i] = field::mul(self.ld_op_flags[i], not_3); }
        for i in 8..16 { self.ld_op_flags[i] = field::mul(self.ld_op_flags[i], self.ld_op_bits[3]); }
        self.ld_op_flags.copy_within(0..16, 16);

        let not_4 = binary_not(self.ld_op_bits[4]);
        for i in 0..16  { self.ld_op_flags[i] = field::mul(self.ld_op_flags[i], not_4); }
        for i in 16..32 { self.ld_op_flags[i] = field::mul(self.ld_op_flags[i], self.ld_op_bits[4]); }

        // set high-degree operation flags
        let not_0 = binary_not(self.hd_op_bits[0]);
        let not_1 = binary_not(self.hd_op_bits[1]);
        self.hd_op_flags[0] = field::mul(not_0, not_1);
        self.hd_op_flags[1] = field::mul(self.hd_op_bits[0], not_1);
        self.hd_op_flags[2] = field::mul(not_0, self.hd_op_bits[1]);
        self.hd_op_flags[3] = field::mul(self.hd_op_bits[0], self.hd_op_bits[1]);

        // we need to make special adjustments for PUSH and ASSERT op flags so that they
        // don't coincide with BEGIN operation; we do this by multiplying each flag by a
        // single op_bit from another bank; this increases degree of each flag by 1
        debug_assert!(OpCode::Push.hd_index() == 0, "PUSH index is not 0!");
        self.hd_op_flags[0] = field::mul(self.hd_op_flags[0], self.ld_op_bits[0]);

        debug_assert!(OpCode::Assert.ld_index() == 0, "ASSERT index is not 0!");
        self.ld_op_flags[0] = field::mul(self.ld_op_flags[0], self.hd_op_bits[0]);

        // mark flags as set
        self.op_flags_set = true;
    }
}

impl fmt::Debug for TraceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:>4}] {:>32X?} {:?} {:?} {:?} {:>32X?} {:>32X?} {:?}",
            self.op_counter,
            self.sponge, 
            self.cf_op_bits,
            self.ld_op_bits,
            self.hd_op_bits,
            self.ctx_stack,
            self.loop_stack,
            self.user_stack
        )
    }
}

impl fmt::Display for TraceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:>4}] {:>16X?} {:?} {:?} {:?} {:>16X?} {:>16X?} {:?}",
            self.op_counter,
            self.sponge.iter().map(|x| x >> 64).collect::<Vec<u128>>(),
            self.cf_op_bits,
            self.ld_op_bits,
            self.hd_op_bits,
            self.ctx_stack.iter().map(|x| x >> 64).collect::<Vec<u128>>(),
            self.loop_stack.iter().map(|x| x >> 64).collect::<Vec<u128>>(),
            &self.user_stack[..self.stack_depth]
        )
    }
}

// HELPER FUNCTIONS
// ================================================================================================
#[inline(always)]
fn binary_not(v: u128) -> u128 {
    return field::sub(field::ONE, v);
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {

    use super::{ TraceState };

    #[test]
    fn from_vec() {

        // empty loop stack
        let state = TraceState::from_vec(1, 0, 2, &vec![
            101, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17
        ]);

        let empty_loop_stack: [u128; 0] = [];
        assert_eq!(101, state.op_counter());
        assert_eq!([1, 2, 3, 4], state.sponge());
        assert_eq!([5, 6, 7], state.cf_op_bits());
        assert_eq!([8, 9, 10, 11, 12], state.ld_op_bits());
        assert_eq!([13, 14], state.hd_op_bits());
        assert_eq!([15], state.ctx_stack());
        assert_eq!(empty_loop_stack, state.loop_stack());
        assert_eq!([16, 17, 0, 0, 0, 0, 0, 0], state.user_stack());
        assert_eq!(18, state.width());
        assert_eq!(2, state.stack_depth());
        assert_eq!(vec![
            101, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17
        ], state.to_vec());

        // non-empty loop stack
        let state = TraceState::from_vec(2, 1, 9, &vec![
            101, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
            18, 19, 20, 21, 22, 23, 24, 25, 26,
        ]);

        assert_eq!(101, state.op_counter());
        assert_eq!([1, 2, 3, 4], state.sponge());
        assert_eq!([5, 6, 7], state.cf_op_bits());
        assert_eq!([8, 9, 10, 11, 12], state.ld_op_bits());
        assert_eq!([13, 14], state.hd_op_bits());
        assert_eq!([15, 16], state.ctx_stack());
        assert_eq!([17], state.loop_stack());
        assert_eq!([18, 19, 20, 21, 22, 23, 24, 25, 26], state.user_stack());
        assert_eq!(27, state.width());
        assert_eq!(9, state.stack_depth());
        assert_eq!(vec![
            101, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
            18, 19, 20, 21, 22, 23, 24, 25, 26,
        ], state.to_vec());
    }

    #[test]
    fn update_from_trace() {
        let data = vec![101, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20];
        let mut trace = Vec::with_capacity(data.len());
        for i in 0..data.len() {
            trace.push(vec![0, data[i], 0]);
        }

        // first row
        let mut state = TraceState::new(2, 1, 3);
        state.update_from_trace(&trace, 0);

        assert_eq!(0, state.op_counter());
        assert_eq!([0, 0, 0, 0], state.sponge());
        assert_eq!([0, 0, 0], state.cf_op_bits());
        assert_eq!([0, 0, 0, 0, 0], state.ld_op_bits());
        assert_eq!([0, 0], state.hd_op_bits());
        assert_eq!([0, 0], state.ctx_stack());
        assert_eq!([0], state.loop_stack());
        assert_eq!([0, 0, 0, 0, 0, 0, 0, 0], state.user_stack());
        assert_eq!(21, state.width());
        assert_eq!(3, state.stack_depth());

        // second row
        state.update_from_trace(&trace, 1);

        assert_eq!(101, state.op_counter());
        assert_eq!([1, 2, 3, 4], state.sponge());
        assert_eq!([5, 6, 7], state.cf_op_bits());
        assert_eq!([8, 9, 10, 11, 12], state.ld_op_bits());
        assert_eq!([13, 14], state.hd_op_bits());
        assert_eq!([15, 16], state.ctx_stack());
        assert_eq!([17], state.loop_stack());
        assert_eq!([18, 19, 20, 0, 0, 0, 0, 0], state.user_stack());
        assert_eq!(21, state.width());
        assert_eq!(3, state.stack_depth());
    }

    #[test]
    fn op_flags() {
        // all ones
        let state = TraceState::from_vec(1, 0, 2, &vec![
            101, 1, 2, 3, 4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 15, 16, 17
        ]);

        assert_eq!([0, 0, 0, 0, 0, 0, 0, 1], state.cf_op_flags());
        assert_eq!([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        ], state.ld_op_flags());
        assert_eq!([0, 0, 0, 1], state.hd_op_flags());

        // all zeros
        let state = TraceState::from_vec(1, 0, 2, &vec![
            101, 1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 16, 17
        ]);

        assert_eq!([1, 0, 0, 0, 0, 0, 0, 0], state.cf_op_flags());
        assert_eq!([
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ], state.ld_op_flags());
        assert_eq!([1, 0, 0, 0], state.hd_op_flags());

        // mixed 1
        let state = TraceState::from_vec(1, 0, 2, &vec![
            101, 1, 2, 3, 4, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 15, 16, 17
        ]);

        assert_eq!([0, 1, 0, 0, 0, 0, 0, 0], state.cf_op_flags());
        assert_eq!([
            0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ], state.ld_op_flags());
        assert_eq!([0, 1, 0, 0], state.hd_op_flags());

        // mixed 2
        let state = TraceState::from_vec(1, 0, 2, &vec![
            101, 1, 2, 3, 4, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 15, 16, 17
        ]);

        assert_eq!([0, 0, 0, 1, 0, 0, 0, 0], state.cf_op_flags());
        assert_eq!([
            0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ], state.ld_op_flags());
        assert_eq!([0, 0, 1, 0], state.hd_op_flags());
    }

    #[test]
    fn op_code() {
        let state = TraceState::from_vec(1, 0, 2, &vec![
            101, 1, 2, 3, 4, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 15, 16, 17
        ]);
        assert_eq!(0, state.op_code());

        let state = TraceState::from_vec(1, 0, 2, &vec![
            101, 1, 2, 3, 4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 15, 16, 17
        ]);
        assert_eq!(127, state.op_code());

        let state = TraceState::from_vec(1, 0, 2, &vec![
            101, 1, 2, 3, 4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 15, 16, 17
        ]);
        assert_eq!(63, state.op_code());

        let state = TraceState::from_vec(1, 0, 2, &vec![
            101, 1, 2, 3, 4, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 15, 16, 17
        ]);
        assert_eq!(97, state.op_code());
    }
}