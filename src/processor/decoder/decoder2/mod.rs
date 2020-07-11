use std::ops::Range;
use crate::math::{ field };
use crate::utils::accumulator::{ add_constants, apply_sbox, apply_mds, apply_inv_sbox };

mod opcodes;
use opcodes::{ FlowOps, UserOps };

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================

const BASE_CYCLE_LENGTH: usize = 16;

const SPONGE_WIDTH: usize = 4;
const NUM_CF_OP_BITS: usize = 3;
const NUM_LD_OP_BITS: usize = 5;
const NUM_HD_OP_BITS: usize = 2;

const OP_ACC_RANGE      : Range<usize> = Range { start:  0, end:  4 };
const CF_OP_BITS_RANGE  : Range<usize> = Range { start:  4, end:  7 };
const LD_OP_BITS_RANGE  : Range<usize> = Range { start:  7, end: 12 };
const HD_OP_BITS_RANGE  : Range<usize> = Range { start: 12, end: 14 };

const MAX_CONTEXT_DEPTH: usize = 16;
const MAX_LOOP_DEPTH: usize = 8;

// TYPES AND INTERFACES
// ================================================================================================
pub struct Decoder {

    step        : usize,

    op_acc      : [Vec<u128>; SPONGE_WIDTH],
    sponge      : [u128; SPONGE_WIDTH],

    cf_op_bits  : [Vec<u128>; NUM_CF_OP_BITS],
    ld_op_bits  : [Vec<u128>; NUM_LD_OP_BITS],
    hd_op_bits  : [Vec<u128>; NUM_HD_OP_BITS],

    ctx_stack   : Vec<Vec<u128>>,
    ctx_depth   : usize,

    loop_stack  : Vec<Vec<u128>>,
    loop_depth  : usize,
}

// DECODER IMPLEMENTATION
// ================================================================================================
impl Decoder {

    pub fn new(init_trace_length: usize) -> Decoder {

        // initialize instruction accumulator
        let op_acc = [
            vec![field::ZERO; init_trace_length], vec![field::ZERO; init_trace_length],
            vec![field::ZERO; init_trace_length], vec![field::ZERO; init_trace_length],
        ];
        let sponge = [field::ZERO; SPONGE_WIDTH];

        // initialize op_bits registers
        let cf_op_bits = [
            vec![field::ZERO; init_trace_length], vec![field::ZERO; init_trace_length],
            vec![field::ZERO; init_trace_length]
        ];
        let ld_op_bits = [
            vec![field::ZERO; init_trace_length], vec![field::ZERO; init_trace_length],
            vec![field::ZERO; init_trace_length], vec![field::ZERO; init_trace_length],
            vec![field::ZERO; init_trace_length]
        ];
        let hd_op_bits = [
            vec![field::ZERO; init_trace_length], vec![field::ZERO; init_trace_length]
        ];

        // initialize stacks
        let ctx_stack = vec![vec![field::ZERO; init_trace_length]];
        let ctx_depth = ctx_stack.len();

        let loop_stack = Vec::new();
        let loop_depth = loop_stack.len();

        // create and return decoder
        return Decoder {
            step: 0, op_acc, sponge, cf_op_bits, ld_op_bits, hd_op_bits,
            ctx_stack, ctx_depth, loop_stack, loop_depth,
        };
    }

    pub fn trace_length(&self) -> usize {
        return self.op_acc[0].len();
    }

    pub fn max_ctx_stack_depth(&self) -> usize {
        return self.ctx_stack.len();
    }

    pub fn max_loop_stack_depth(&self) -> usize {
        return self.loop_stack.len();
    }

    pub fn get_state(&self, step: usize) -> Vec<u128> {
        let mut state = Vec::new();

        for register in self.op_acc.iter()     { state.push(register[step]); }
        for register in self.cf_op_bits.iter() { state.push(register[step]); }
        for register in self.ld_op_bits.iter() { state.push(register[step]); }
        for register in self.hd_op_bits.iter() { state.push(register[step]); }
        for register in self.ctx_stack.iter()  { state.push(register[step]); }
        for register in self.loop_stack.iter() { state.push(register[step]); }

        return state;
    }

    pub fn print_state(&self, step: usize) {
        let state = self.get_state(step);
        let ctx_stack_start = HD_OP_BITS_RANGE.end;
        let ctx_stack_end = ctx_stack_start + self.max_ctx_stack_depth();

        println!("{}:\t{:>32X?} {:?} {:?} {:?} {:X?} {:X?}", step,
            &state[OP_ACC_RANGE], &state[CF_OP_BITS_RANGE],
            &state[LD_OP_BITS_RANGE], &state[HD_OP_BITS_RANGE],
            &state[ctx_stack_start..ctx_stack_end], &state[ctx_stack_end..],
        );
    }

    // OPERATION DECODERS
    // --------------------------------------------------------------------------------------------
    pub fn start_block(&mut self) {
        assert!(self.step % BASE_CYCLE_LENGTH == BASE_CYCLE_LENGTH - 1,
            "cannot start context block at step {}: operation alignment is not valid", self.step);

        self.advance_step();
        self.set_op_bits(FlowOps::Begin, UserOps::Noop);
        self.save_context(self.sponge[0]);
        self.copy_loop_stack();
        self.set_sponge([0, 0, 0, 0]);
    }

    pub fn end_block(&mut self, sibling_hash: u128, true_branch: bool) {
        assert!(self.step % BASE_CYCLE_LENGTH == 0,
            "cannot exit context block at step {}: operation alignment is not valid", self.step);
        self.advance_step();

        let h = self.pop_context();
        if true_branch {
            self.set_op_bits(FlowOps::Tend, UserOps::Noop);
            self.set_sponge([h, self.sponge[0], sibling_hash, 0]);
        }
        else {
            self.set_op_bits(FlowOps::Fend, UserOps::Noop);
            self.set_sponge([h, sibling_hash, self.sponge[0], 0]);
        }
        self.copy_loop_stack();
    }

    pub fn start_loop(&mut self, loop_image: u128) {
        assert!(self.step % BASE_CYCLE_LENGTH == BASE_CYCLE_LENGTH - 1,
            "cannot start a loop at step {}: operation alignment is not valid", self.step);
        self.advance_step();

        self.set_op_bits(FlowOps::Loop, UserOps::Noop);
        self.save_context(self.sponge[0]);
        self.save_loop_image(loop_image);
        self.set_sponge([0, 0, 0, 0]);
    }

    pub fn wrap_loop(&mut self) {
        assert!(self.step % BASE_CYCLE_LENGTH == BASE_CYCLE_LENGTH - 1,
            "cannot wrap a loop at step {}: operation alignment is not valid", self.step);
        self.advance_step();

        self.set_op_bits(FlowOps::Wrap, UserOps::Noop);
        let loop_image = self.peek_loop_image();
        assert!(loop_image == self.sponge[0], "TODO");
        self.copy_context_stack();
        self.set_sponge([0, 0, 0, 0]);
    }

    pub fn break_loop(&mut self) {
        assert!(self.step % BASE_CYCLE_LENGTH == BASE_CYCLE_LENGTH - 1,
            "cannot break a loop at step {}: operation alignment is not valid", self.step);
        self.advance_step();

        self.set_op_bits(FlowOps::Break, UserOps::Noop);
        let loop_image = self.pop_loop_image();
        assert!(loop_image == self.sponge[0], "TODO: wrong loop image: {}", self.sponge[0]);
        self.copy_context_stack();
        self.set_sponge(self.sponge);
    }

    pub fn decode_op(&mut self, op_code: UserOps, op_value: u128) {
        // TODO: if op_value != 0, make sure this happens on a step which is a multiple of 8
        self.advance_step();
        self.set_op_bits(FlowOps::Hacc, op_code);
        self.apply_hacc_round(op_code, op_value);
        self.copy_context_stack();
        self.copy_loop_stack();
    }

    pub fn finalize_trace(&mut self) {
        for register in self.op_acc.iter_mut()     { fill_register(register, self.step + 1, register[self.step]); }
        for register in self.cf_op_bits.iter_mut() { fill_register(register, self.step, field::ONE); }
        for register in self.ld_op_bits.iter_mut() { fill_register(register, self.step, field::ONE); }
        for register in self.hd_op_bits.iter_mut() { fill_register(register, self.step, field::ONE); }
        for register in self.ctx_stack.iter_mut()  { fill_register(register, self.step + 1, register[self.step]); }
        for register in self.loop_stack.iter_mut() { fill_register(register, self.step + 1, register[self.step]); }
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------

    fn advance_step(&mut self) {
        self.step += 1;
        if self.step >= self.trace_length() {
            let new_length = self.trace_length() * 2;

            for register in self.op_acc.iter_mut()     { register.resize(new_length, field::ZERO); }
            for register in self.cf_op_bits.iter_mut() { register.resize(new_length, field::ZERO); }
            for register in self.ld_op_bits.iter_mut() { register.resize(new_length, field::ZERO); }
            for register in self.hd_op_bits.iter_mut() { register.resize(new_length, field::ZERO); }
            for register in self.ctx_stack.iter_mut()  { register.resize(new_length, field::ZERO); }
            for register in self.loop_stack.iter_mut() { register.resize(new_length, field::ZERO); }
        }
    }
    
    fn set_op_bits(&mut self, flow_op: FlowOps, user_op: UserOps) {

        let step = self.step - 1;

        let flow_op = flow_op as u8;
        for i in 0..NUM_CF_OP_BITS {
            self.cf_op_bits[i][step] = ((flow_op >> i) & 1) as u128;
        }

        let user_op = user_op as u8;
        for i in 0..NUM_LD_OP_BITS {
            self.ld_op_bits[i][step] = ((user_op >> i) & 1) as u128;
        }

        for i in 0..NUM_HD_OP_BITS {
            self.hd_op_bits[i][step] = ((user_op >> (i + NUM_LD_OP_BITS)) & 1) as u128;
        }
    }

    fn save_context(&mut self, parent_hash: u128) {
        self.ctx_depth += 1;
        assert!(self.ctx_depth <= MAX_CONTEXT_DEPTH, "context stack overflow at step {}", self.step);

        if self.ctx_depth > self.ctx_stack.len() {
            self.ctx_stack.push(vec![field::ZERO; self.trace_length()]);
        }

        for i in 1..self.ctx_stack.len() {
            self.ctx_stack[i][self.step] = self.ctx_stack[i - 1][self.step - 1];
        }
        self.ctx_stack[0][self.step] = parent_hash;
    }

    fn pop_context(&mut self) -> u128 {
        assert!(self.ctx_depth > 0, "context stack underflow at step {}", self.step);

        let context_hash = self.ctx_stack[0][self.step - 1];

        for i in 1..self.ctx_stack.len() {
            self.ctx_stack[i - 1][self.step] = self.ctx_stack[i][self.step - 1];
        }

        self.ctx_depth -= 1;
        return context_hash;
    }

    fn copy_context_stack(&mut self) {
        for i in 0..self.ctx_stack.len() {
            self.ctx_stack[i][self.step] = self.ctx_stack[i][self.step - 1];
        }
    }

    fn save_loop_image(&mut self, loop_image: u128) {
        self.loop_depth += 1;
        assert!(self.loop_depth <= MAX_LOOP_DEPTH, "loop stack overflow at step {}", self.step);

        if self.loop_depth > self.loop_stack.len() {
            self.loop_stack.push(vec![field::ZERO; self.trace_length()]);
        }

        for i in 1..self.loop_stack.len() {
            self.loop_stack[i][self.step] = self.loop_stack[i - 1][self.step - 1];
        }
        self.loop_stack[0][self.step] = loop_image;
    }

    fn peek_loop_image(&self) -> u128 {
        // TODO: implement
        return 0;
    }

    fn pop_loop_image(&mut self) -> u128 {
        assert!(self.loop_depth > 0, "loop stack underflow at step {}", self.step);

        let loop_image = self.loop_stack[0][self.step - 1];

        for i in 1..self.loop_stack.len() {
            self.loop_stack[i - 1][self.step] = self.loop_stack[i][self.step - 1];
        }
        self.loop_stack[0][self.step] = field::ZERO;

        self.loop_depth -= 1;
        return loop_image;
    }

    fn copy_loop_stack(&mut self) {
        for i in 0..self.loop_stack.len() {
            self.loop_stack[i][self.step] = self.loop_stack[i][self.step - 1];
        }
    }

    fn set_sponge(&mut self, state: [u128; SPONGE_WIDTH]) {
        self.op_acc[0][self.step] = state[0];
        self.op_acc[1][self.step] = state[1];
        self.op_acc[2][self.step] = state[2];
        self.op_acc[3][self.step] = state[3];
        self.sponge = state;
    }

    fn apply_hacc_round(&mut self, op_code: UserOps, op_value: u128) {

        let ark_idx = (self.step - 1) % BASE_CYCLE_LENGTH;

        // apply first half of Rescue round
        add_constants(&mut self.sponge, ark_idx, 0);
        apply_sbox(&mut self.sponge);
        apply_mds(&mut self.sponge);
    
        // inject value into the state
        self.sponge[0] = field::add(self.sponge[0], op_code as u128);
        self.sponge[1] = field::add(self.sponge[1], op_value);
    
        // apply second half of Rescue round
        add_constants(&mut self.sponge, ark_idx, SPONGE_WIDTH);
        apply_inv_sbox(&mut self.sponge);
        apply_mds(&mut self.sponge);

        // copy the new sponge state into the op_acc registers
        for i in 0..SPONGE_WIDTH {
            self.op_acc[i][self.step] = self.sponge[i];
        }
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn fill_register(register: &mut Vec<u128>, from: usize, value: u128) {
    let to = register.len();
    register.resize(from, field::ZERO);
    register.resize(to, value);
}