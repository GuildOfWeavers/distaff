use crate::math::{ field, fft, polys };
use crate::processor::opcodes;
use crate::utils::zero_filled_vector;
use super::{ TraceState, stack::Stack, hash_acc };

// TYPES AND INTERFACES
// ================================================================================================
pub struct TraceTable {
    op_code     : Vec<u64>,
    push_flag   : Vec<u64>,
    op_bits     : [Vec<u64>; 5],
    op_acc      : [Vec<u64>; hash_acc::STATE_WIDTH],
    stack       : Stack,

    extension_factor: usize
}

// TRACE TABLE IMPLEMENTATION
// ================================================================================================
impl TraceTable {

    /// Returns a trace table resulting from the execution of the specified program. Space for the
    /// trace table is allocated in accordance with the specified `extension_factor`.
    pub fn new(program: &[u64], inputs: &[u64], extension_factor: usize) -> TraceTable {
        
        let trace_length = program.len();
        let domain_size = trace_length * extension_factor;

        assert!(trace_length.is_power_of_two(), "program length must be a power of 2");
        assert!(extension_factor.is_power_of_two(), "trace extension factor must be a power of 2");
        assert!(program[trace_length - 1] == opcodes::NOOP, "last operation in a program must be NOOP");

        // allocate space for trace table registers. capacity of each register is set to the
        // domain size right from the beginning to avoid vector re-allocation later on.
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
        let op_acc = hash_acc::digest(&program, extension_factor);
        let stack = Stack::new(trace_length, inputs, extension_factor);
        let mut trace = TraceTable { op_code, push_flag, op_bits, op_acc, stack, extension_factor };

        // copy program into the trace
        trace.op_code[0..program.len()].copy_from_slice(program);
        
        // execute the program to fill out the trace and return
        trace.execute_program();
        return trace;
    }

    /// Returns state of the trace table at the specified `step`.
    pub fn get_state(&self, step: usize) -> TraceState {
        let mut result = TraceState::new(self.max_stack_depth());
        self.fill_state(&mut result, step);
        return result;
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

    /// Returns `extension_factor` for the trace table.
    pub fn extension_factor(&self) -> usize {
        return self.extension_factor;
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

    /// Returns `true` if the trace table has been extended.
    pub fn is_extended(&self) -> bool {
        return self.op_code.len() == self.op_code.capacity();
    }

    // INTERPOLATION AND EXTENSION
    // --------------------------------------------------------------------------------------------

    /// Extends all registers of the trace table by the `extension_factor` specified during
    /// trace table construction. A trace table can be extended only once.
    pub fn extend(&mut self) {
        assert!(!self.is_extended(), "trace table has already been extended");
        let domain_size = self.len() * self.extension_factor();

        // build vectors of twiddles and inv_twiddles needed for FFT
        let root = field::get_root_of_unity(self.len() as u64);
        let inv_twiddles = fft::get_inv_twiddles(root, self.len());
        let root = field::get_root_of_unity(domain_size as u64);
        let twiddles = fft::get_twiddles(root, domain_size);

        // extend op_code
        polys::interpolate_fft_twiddles(&mut self.op_code, &inv_twiddles, true);
        unsafe { self.op_code.set_len(domain_size); }
        polys::eval_fft_twiddles(&mut self.op_code, &twiddles, true);

        // extend push_flag
        polys::interpolate_fft_twiddles(&mut self.push_flag, &inv_twiddles, true);
        unsafe { self.push_flag.set_len(domain_size); }
        polys::eval_fft_twiddles(&mut self.push_flag, &twiddles, true);

        // extend op_bits
        for op_bit in self.op_bits.iter_mut() {
            polys::interpolate_fft_twiddles(op_bit, &inv_twiddles, true);
            unsafe { op_bit.set_len(domain_size); }
            polys::eval_fft_twiddles(op_bit, &twiddles, true);
        }

        // extend op_acc
        for acc in self.op_acc.iter_mut() {
            polys::interpolate_fft_twiddles(acc, &inv_twiddles, true);
            unsafe { acc.set_len(domain_size); }
            polys::eval_fft_twiddles(acc, &twiddles, true);
        }

        // extend stack registers
        self.stack.extend_registers(&twiddles, &inv_twiddles);
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
    }

    /// Sets the op_bits registers at the specified `step` to a binary decomposition
    /// of the `op_code` parameter.
    fn set_op_bits(&mut self, op_code: u64, step: usize) {
        for i in 0..self.op_bits.len() {
            self.op_bits[i][step] = (op_code >> i) & 1;
        }
    }
}