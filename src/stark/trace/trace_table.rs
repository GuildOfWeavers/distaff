use crate::math::{ field, fft, polys };
use crate::processor::opcodes;
use crate::utils::zero_filled_vector;
use super::{ TraceState, stack::Stack, acc_hash };

// TYPES AND INTERFACES
// ================================================================================================
#[derive(Clone, Copy, PartialEq)]
pub enum TraceTableState {
    Initialized, Executed, Interpolated, Extended
}

pub struct TraceTable {
    state       : TraceTableState,
    op_code     : Vec<u64>,
    push_flag   : Vec<u64>,
    op_bits     : [Vec<u64>; 5],
    op_acc      : [Vec<u64>; acc_hash::STATE_WIDTH],
    stack       : Stack,
}

// TRACE TABLE IMPLEMENTATION
// ================================================================================================
impl TraceTable {

    /// Returns a trace table resulting from the execution of the specified program. Space for the
    /// trace table will be allocated in accordance with the specified `extension_factor`.
    pub fn new(program: &[u64], extension_factor: usize) -> TraceTable {
        
        let trace_length = program.len() + 1;
        let domain_size = trace_length * extension_factor;

        assert!(trace_length.is_power_of_two(), "program length must be one less than a power of 2");
        assert!(extension_factor.is_power_of_two(), "trace extension factor must be a power of 2");

        // allocate space for trace table registers. capacity of each register is set to the
        // domain size right from the beginning to avoid vector re-allocation later on.
        let state = TraceTableState::Initialized;
        let op_code = zero_filled_vector(trace_length, domain_size);
        let push_flag = zero_filled_vector(trace_length, domain_size);
        let op_bits = [
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
        ];

        // create trace table object
        let op_acc = acc_hash::digest(&program, extension_factor);
        let stack = Stack::new(trace_length, extension_factor);
        let mut trace = TraceTable { state, op_code, push_flag, op_bits, op_acc, stack };

        // copy program into the trace and set the last operation to NOOP
        trace.op_code[0..program.len()].copy_from_slice(program);
        trace.op_code[trace_length - 1] = opcodes::NOOP;
        
        // execute the program to fill out the trace and return
        trace.execute_program();
        return trace;
    }

    /// Copies trace table state at the specified `step` to the passed in `state` object.
    pub fn fill_state(&self, state: &mut TraceState, step: usize) {
        state.set_op_code(self.op_code[step]);
        state.set_push_flag(self.push_flag[step]);
        state.set_op_bits([
            self.op_bits[0][step], self.op_bits[1][step], self.op_bits[2][step],
            self.op_bits[3][step], self.op_bits[4][step]
        ]);
        state.set_op_acc([
            self.op_acc[0][step], self.op_acc[1][step], self.op_acc[2][step],  self.op_acc[3][step],
            self.op_acc[4][step], self.op_acc[5][step], self.op_acc[6][step],  self.op_acc[7][step],
            self.op_acc[8][step], self.op_acc[9][step], self.op_acc[10][step], self.op_acc[11][step]
        ]);
        self.stack.fill_state(state, step);
    }

    /// Returns the number of states in the trace table.
    pub fn len(&self) -> usize {
        return self.op_code.len();
    }

    /// Returns the number of registers in the trace table.
    pub fn register_count(&self) -> usize {
        return 1 + self.op_bits.len() + self.op_acc.len() + self.stack.max_depth();
    }

    /// Returns the number of registers used by the stack.
    pub fn max_stack_depth(&self) -> usize {
        return self.stack.max_depth();
    }

    /// Returns register trace at the specified `index`.
    pub fn get_register_trace(&self, index: usize) -> &[u64] {
        return match index {
            0     => &self.op_code,
            1..=5 => &self.op_bits[index - 1],
            _     => self.stack.get_register_trace(index - 6)
        };
    }

    /// Returns `true` if the trace table has been interpolated, but has not yet been extended.
    pub fn is_interpolated(&self) -> bool {
        return self.state == TraceTableState::Interpolated;
    }

    /// Returns `true` if the trace table has been extended.
    pub fn is_extended(&self) -> bool {
        return self.state == TraceTableState::Extended;
    }

    /// Makes a deep copy of the trace table. The cloned trace table will have the extension_factor
    /// set to the specified value.
    pub fn clone(&self, extension_factor: usize) -> TraceTable {
        assert!(extension_factor.is_power_of_two(), "trace extension factor must be a power of 2");

        let trace_length = self.len();
        let domain_size = trace_length * extension_factor;

        // clone op_code register
        let mut op_code = zero_filled_vector(trace_length, domain_size);
        op_code.copy_from_slice(&self.op_code);

        // clone was_push register
        let mut push_flag = zero_filled_vector(trace_length, domain_size);
        push_flag.copy_from_slice(&self.push_flag);

        // clone op_bits registers
        let mut op_bits = [
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
        ];
        for i in 0..op_bits.len() {
            op_bits[i].copy_from_slice(&self.op_bits[i]);
        }

        // clone op_acc registers
        let mut op_acc = [
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
        ];
        for i in 0..op_acc.len() {
            op_acc[i].copy_from_slice(&self.op_acc[i]);
        }

        // clone operation accumulator and stack
        let stack = self.stack.clone(extension_factor);

        return TraceTable { state: self.state, op_code, push_flag, op_bits, op_acc, stack };
    }

    // INTERPOLATION AND EXTENSION
    // --------------------------------------------------------------------------------------------

    /// Interpolates all registers of the trace table using FFT interpolation. Interpolation is
    /// allowed only once. That is, once the trace table is interpolated, it cannot be interpolated
    /// again.
    pub fn interpolate(&mut self) {
        assert!(!self.is_interpolated(), "trace table has already been interpolated");
        assert!(!self.is_extended(), "cannot interpolate extended trace table");

        let root = field::get_root_of_unity(self.len() as u64);
        let inv_twiddles = fft::get_inv_twiddles(root, self.len());

        polys::interpolate_fft_twiddles(&mut self.op_code, &inv_twiddles, true);
        polys::interpolate_fft_twiddles(&mut self.push_flag, &inv_twiddles, true);
        for op_bit in self.op_bits.iter_mut() {
            polys::interpolate_fft_twiddles(op_bit, &inv_twiddles, true);
        }
        for acc in self.op_acc.iter_mut() {
            polys::interpolate_fft_twiddles(acc, &inv_twiddles, true);
        }
        self.stack.interpolate_registers(&inv_twiddles);

        self.state = TraceTableState::Interpolated;
    }

    /// Extends all registers of the trace table by the extension_factor specified during
    /// trace table construction. Extension is allowed only once, right after the trace table
    /// is interpolated.
    pub fn extend(&mut self) {
        assert!(!self.is_extended(), "trace table has already been extended");
        assert!(self.is_interpolated(), "cannot extend un-interpolated trace table");

        let domain_length = self.op_code.capacity();
        let root = field::get_root_of_unity(domain_length as u64);
        let twiddles = fft::get_twiddles(root, domain_length);

        unsafe { self.op_code.set_len(domain_length); }
        polys::eval_fft_twiddles(&mut self.op_code, &twiddles, true);

        unsafe { self.push_flag.set_len(domain_length); }
        polys::eval_fft_twiddles(&mut self.push_flag, &twiddles, true);

        for op_bit in self.op_bits.iter_mut() {
            debug_assert!(op_bit.capacity() == domain_length, "invalid register capacity");
            unsafe { op_bit.set_len(domain_length); }
            polys::eval_fft_twiddles(op_bit, &twiddles, true);
        }

        for acc in self.op_acc.iter_mut() {
            debug_assert!(acc.capacity() == domain_length, "invalid register capacity");
            unsafe { acc.set_len(domain_length); }
            polys::eval_fft_twiddles(acc, &twiddles, true);
        }

        self.stack.extend_registers(&twiddles);

        self.state = TraceTableState::Extended;
    }

    // PROGRAM EXECUTION
    // --------------------------------------------------------------------------------------------

    /// Execute the program contained in the op_code register. This will fill in all other
    /// registers of the trace table.
    fn execute_program(&mut self) {
        
        // for the first operation, push_flag is always 0
        self.push_flag[0] = 0;

        for i in 0..(self.len() - 1) {
            if self.push_flag[i] == 1 {
                // if the previous operation was a PUSH, current operation must be a constant that
                // was pushed onto the stack - so, skip it and leave the stack state unchanged
                self.set_op_bits(opcodes::NOOP, i);
                self.stack.noop();

                // clear push_flag for the next operation since the current operation is not a PUSH
                self.push_flag[i + 1] = 0;
            }
            else {
                let op = self.op_code[i];
                self.set_op_bits(op, i);

                // if the current operation is a PUSH, set push_flag for the next step to 1
                self.push_flag[i + 1] = if op == opcodes::PUSH { 1 } else { 0 };
                
                // update stack state based on the op_code
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

        // set op_bits for the last step
        self.set_op_bits(self.op_code[self.len() - 1], self.len() - 1);

        self.state = TraceTableState::Executed;
    }

    /// Sets the op_bits registers at the specified `step` to a binary decomposition
    /// of the `op_code` parameter.
    fn set_op_bits(&mut self, op_code: u64, step: usize) {
        for i in 0..self.op_bits.len() {
            self.op_bits[i][step] = (op_code >> i) & 1;
        }
    }
}