use crate::math::{ field, fft, polys };
use crate::trace::{ TraceState, Stack };
use crate::opcodes;
use crate::utils;

// TYPES AND INTERFACES
// ================================================================================================
pub enum TraceTableState {
    Initialized, Executed, Interpolated, Extended
}

pub struct TraceTable {
    state       : TraceTableState,
    op_code     : Vec<u64>,
    op_bits     : [Vec<u64>; 8],
    stack       : Stack,
}

// TRACE TABLE IMPLEMENTATION
// ================================================================================================
impl TraceTable {

    pub fn new(program: &[u64], extension_factor: usize) -> TraceTable {
        let trace_length = program.len() + 1;
        assert!(trace_length.is_power_of_two(), "program length must be one less than power of 2");

        let state = TraceTableState::Initialized;
        let op_code = utils::uninit_vector(trace_length);
        let op_bits = [
            utils::uninit_vector(trace_length),
            utils::uninit_vector(trace_length),
            utils::uninit_vector(trace_length),
            utils::uninit_vector(trace_length),
            utils::uninit_vector(trace_length),
            utils::uninit_vector(trace_length),
            utils::uninit_vector(trace_length),
            utils::uninit_vector(trace_length)
        ];

        let stack = Stack::new(trace_length, extension_factor);
        let mut trace = TraceTable { state, op_code, op_bits, stack };

        // copy program into the trace and set the last operation to NOOP
        trace.op_code[0..program.len()].copy_from_slice(program);
        trace.op_code[trace_length - 1] = opcodes::NOOP;
        
        // execute the program to fill out the trace and return
        trace.execute_program();
        return trace;
    }

    pub fn fill_state(&self, state: &mut TraceState, step: usize) {
        state.op_code = self.op_code[step];
        for i in 0..self.op_bits.len() {
            state.op_bits[i] = self.op_bits[i][step];
        }
        state.copy_flag = 0;    // TODO
        self.stack.fill_state(state, step);
    }

    pub fn len(&self) -> usize {
        return self.op_code.len();
    }

    pub fn register_count(&self) -> usize {
        return 1 + self.op_bits.len() + self.stack.max_depth();
    }

    pub fn max_stack_depth(&self) -> usize {
        return self.stack.max_depth();
    }

    pub fn get_register_trace(&self, index: usize) -> &[u64] {
        return match index {
            0     => &self.op_code,
            1..=8 => &self.op_bits[index - 1],
            _     => self.stack.get_register_trace(index - 9)
        };
    }

    // INTERPOLATION
    // --------------------------------------------------------------------------------------------
    pub fn interpolate_traces(&self) -> Vec<Vec<u64>> {

        let mut result = Vec::new();

        let root = field::get_root_of_unity(self.len() as u64);
        let twiddles = fft::get_inv_twiddles(root, self.len());

        for i in 0..self.register_count() {
            let mut trace = self.get_register_trace(i).to_vec();
            polys::interpolate_fft_twiddles(&mut trace, &twiddles, true);
            result.push(trace);
        }
        
        return result;
    }

    // PROGRAM EXECUTION
    // --------------------------------------------------------------------------------------------
    fn execute_program(&mut self) {
        
        let mut was_push = false;
        for i in 0..(self.len() - 1) {
            if was_push {
                // if the previous operation was a PUSH, current operation must be a constant that
                // was pushed onto the stack - so, skip it and leave the stack state unchanged
                self.set_op_bits(opcodes::NOOP, i);
                was_push = false;
                self.stack.noop();
            }
            else {
                let op = self.op_code[i];
                self.set_op_bits(op, i);
                was_push = op == opcodes::PUSH;

                match op {
                    opcodes::NOOP  => self.stack.noop(),

                    opcodes::PUSH  => self.stack.push(self.op_code[i + 1]),
                    opcodes::DUP0  => self.stack.dup0(),
                    opcodes::DUP1  => self.stack.dup1(),

                    opcodes::PULL1 => self.stack.pull1(),
                    opcodes::PULL2 => self.stack.pull2(),

                    opcodes::DROP  => self.stack.drop(),
                    opcodes::ADD   => self.stack.add(),
                    opcodes::SUB   => self.stack.sub(),
                    opcodes::MUL   => self.stack.mul(),

                    _ => panic!("operation {} is not supported", op)
                }
            }
        }
        self.set_op_bits(self.op_code[self.len() - 1], self.len() - 1);
        self.state = TraceTableState::Executed;
    }

    fn set_op_bits(&mut self, op_code: u64, step: usize) {
        for i in 0..self.op_bits.len() {
            self.op_bits[i][step] = (op_code >> i) & 1;
        }
    }
}