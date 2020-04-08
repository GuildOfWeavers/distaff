use crate::math::{ field, fft, polys };
use crate::trace::{ TraceState, Stack };
use crate::opcodes;
use crate::utils;

// TYPES AND INTERFACES
// ================================================================================================
#[derive(Clone, Copy, PartialEq)]
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
        assert!(trace_length.is_power_of_two(), "program length must be one less than a power of 2");
        assert!(extension_factor.is_power_of_two(), "trace extension factor must be a power of 2");
        let domain_size = trace_length * extension_factor;

        let state = TraceTableState::Initialized;
        let op_code = utils::zero_filled_vector(trace_length, domain_size);
        let op_bits = [
            utils::zero_filled_vector(trace_length, domain_size),
            utils::zero_filled_vector(trace_length, domain_size),
            utils::zero_filled_vector(trace_length, domain_size),
            utils::zero_filled_vector(trace_length, domain_size),
            utils::zero_filled_vector(trace_length, domain_size),
            utils::zero_filled_vector(trace_length, domain_size),
            utils::zero_filled_vector(trace_length, domain_size),
            utils::zero_filled_vector(trace_length, domain_size)
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

    pub fn is_interpolated(&self) -> bool {
        return self.state == TraceTableState::Interpolated;
    }

    pub fn is_extended(&self) -> bool {
        return self.state == TraceTableState::Extended;
    }

    pub fn clone(&self, extension_factor: usize) -> TraceTable {
        assert!(extension_factor.is_power_of_two(), "trace extension factor must be a power of 2");
        let trace_length = self.len();
        let domain_size = trace_length * extension_factor;

        // clone op_code register
        let mut op_code = utils::zero_filled_vector(trace_length, domain_size);
        op_code.copy_from_slice(&self.op_code);

        // clone op_bits registers
        let mut op_bits = [
            utils::zero_filled_vector(trace_length, domain_size),
            utils::zero_filled_vector(trace_length, domain_size),
            utils::zero_filled_vector(trace_length, domain_size),
            utils::zero_filled_vector(trace_length, domain_size),
            utils::zero_filled_vector(trace_length, domain_size),
            utils::zero_filled_vector(trace_length, domain_size),
            utils::zero_filled_vector(trace_length, domain_size),
            utils::zero_filled_vector(trace_length, domain_size)
        ];
        for i in 0..op_bits.len() {
            op_bits[i].copy_from_slice(&self.op_bits[i]);
        }

        // clone the stack
        let stack = self.stack.clone(extension_factor);

        return TraceTable { state: self.state, op_code, op_bits, stack };
    }

    // INTERPOLATION AND EXTENSION
    // --------------------------------------------------------------------------------------------
    pub fn interpolate(&mut self) {
        assert!(!self.is_interpolated(), "trace table has already been interpolated");
        assert!(!self.is_extended(), "cannot interpolate extended trace table");

        let root = field::get_root_of_unity(self.len() as u64);
        let inv_twiddles = fft::get_inv_twiddles(root, self.len());

        polys::interpolate_fft_twiddles(&mut self.op_code, &inv_twiddles, true);
        for op_bit in self.op_bits.iter_mut() {
            polys::interpolate_fft_twiddles(op_bit, &inv_twiddles, true);
        }
        self.stack.interpolate_registers(&inv_twiddles);

        self.state = TraceTableState::Interpolated;
    }

    pub fn extend(&mut self) {
        assert!(!self.is_extended(), "trace table has already been extended");
        assert!(self.is_interpolated(), "cannot extend un-interpolated trace table");

        let domain_length = self.op_code.capacity();
        let root = field::get_root_of_unity(domain_length as u64);
        let twiddles = fft::get_twiddles(root, domain_length);

        unsafe { self.op_code.set_len(domain_length); }
        polys::eval_fft_twiddles(&mut self.op_code, &twiddles, true);
        for op_bit in self.op_bits.iter_mut() {
            debug_assert!(op_bit.capacity() == domain_length, "invalid register capacity");
            unsafe { op_bit.set_len(domain_length); }
            polys::eval_fft_twiddles(op_bit, &twiddles, true);
        }
        self.stack.extend_registers(&twiddles);

        self.state = TraceTableState::Extended;
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