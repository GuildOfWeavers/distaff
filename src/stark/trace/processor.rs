use crate::math::{ field, fft, polys };
use crate::trace::{ Stack, TraceState };
use crate::opcodes;
use crate::utils;

// TYPES AND INTERFACES
// ================================================================================================
pub struct Processor {
    op_code     : Vec<u64>,
    op_bits     : [Vec<u64>; 8],
    copy_flag   : Vec<u64>,
    stack       : Stack,
}

// PROCESSOR IMPLEMENTATION
// ================================================================================================
impl Processor {

    pub fn new(max_stack_depth: usize, trace_length: usize) -> Processor {
        let stack = Stack::new(max_stack_depth, trace_length);
        let mut op_code = utils::uninit_vector(trace_length);
        let mut op_bits = [
            utils::uninit_vector(trace_length),
            utils::uninit_vector(trace_length),
            utils::uninit_vector(trace_length),
            utils::uninit_vector(trace_length),
            utils::uninit_vector(trace_length),
            utils::uninit_vector(trace_length),
            utils::uninit_vector(trace_length),
            utils::uninit_vector(trace_length)
        ];
        let mut copy_flag = utils::uninit_vector(trace_length);

        op_code[0] = opcodes::NOOP;
        for i in 0..op_bits.len() {
            op_bits[i][0] = (op_code[0] >> i) & 1;
        }
        copy_flag[0] = 0;

        return Processor { op_code, op_bits, copy_flag, stack };
    }

    pub fn fill_state(&self, state: &mut TraceState, step: usize) {
        state.op_code = self.op_code[step];
        for i in 0..self.op_bits.len() {
            state.op_bits[i] = self.op_bits[i][step];
        }
        state.copy_flag = self.copy_flag[step];
        self.stack.fill_state(state, step);
    }

    pub fn register_count(&self) -> usize {
        return 2 + self.op_bits.len() + self.stack.max_stack_depth();
    }

    pub fn trace_length(&self) -> usize {
        return self.stack.trace_length();
    }

    pub fn current_step(&self) -> usize {
        return self.stack.current_step();
    }

    pub fn get_register_trace(&self, index: usize) -> &[u64] {
        return match index {
            0     => &self.op_code,
            1..=8 => &self.op_bits[index - 1],
            9     => &self.copy_flag,
            _     => self.stack.get_register_trace(index - 10)
        };
    }

    pub fn consume(&mut self, mut op: u64) {
        
        if self.copy_flag[self.current_step()] == 1 {
            // stack state must be updated first to advance current_step
            self.stack.push(op);

            let current_step = self.current_step();
            self.op_code[current_step] = op;
            self.copy_flag[current_step] = 0;
            op = opcodes::COPY;
        }
        else {
            // stack state must be updated first to advance current_step
            match op {
                opcodes::NOOP => self.stack.noop(),
                opcodes::PUSH => self.stack.noop(),
    
                opcodes::DUP0 => self.stack.dup0(),
                opcodes::DUP1 => self.stack.dup1(),

                opcodes::POP => self.stack.pop(),
                opcodes::ADD => self.stack.add(),
                opcodes::SUB => self.stack.sub(),
                opcodes::MUL => self.stack.mul(),

                _ => panic!("operation {} is not supported", op)
            }

            // set op_code register
            let current_step = self.current_step();
            self.op_code[current_step] = op;

            // if the current operation is push, set the copy flag
            self.copy_flag[current_step] = if op == opcodes::PUSH { 1 } else { 0 };
        }

        // update binary decomposition of the op_code
        let current_step = self.current_step();
        for i in 0..self.op_bits.len() {
            self.op_bits[i][current_step] = (op >> i) & 1;
        }
    }

    pub fn interpolate_traces(&self) -> Vec<Vec<u64>> {

        let mut result = Vec::new();

        let root = field::get_root_of_unity(self.trace_length() as u64);
        let twiddles = fft::get_inv_twiddles(root, self.trace_length());

        for i in 0..self.register_count() {
            let mut trace = self.get_register_trace(i).to_vec();
            polys::interpolate_fft_twiddles(&mut trace, &twiddles, true);
            result.push(trace);
        }
        
        return result;
    }
}