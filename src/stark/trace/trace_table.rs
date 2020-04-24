use crate::math::{ field, fft, polys };
use crate::processor::opcodes;
use crate::stark::{ hash_acc::STATE_WIDTH as ACC_STATE_WIDTH };
use super::{ TraceState, decoder, stack, NUM_OP_BITS };

// TYPES AND INTERFACES
// ================================================================================================
pub struct TraceTable {
    op_code     : Vec<u64>,
    push_flag   : Vec<u64>,
    op_bits     : [Vec<u64>; NUM_OP_BITS],
    op_acc      : [Vec<u64>; ACC_STATE_WIDTH],
    stack       : Vec<Vec<u64>>,

    extension_factor: usize
}

// TRACE TABLE IMPLEMENTATION
// ================================================================================================
impl TraceTable {

    /// Returns a trace table resulting from the execution of the specified program. Space for the
    /// trace table is allocated in accordance with the specified `extension_factor`.
    pub fn new(program: &[u64], inputs: &[u64], extension_factor: usize) -> TraceTable {
        
        assert!(program.len().is_power_of_two(), "program length must be a power of 2");
        assert!(extension_factor.is_power_of_two(), "trace extension factor must be a power of 2");
        assert!(program[program.len() - 1] == opcodes::NOOP, "last operation of a program must be NOOP");

        // create trace table object
        let (op_code, push_flag, op_bits, op_acc) = decoder::process(program, extension_factor);
        let stack = stack::execute(program, inputs, extension_factor);
        return TraceTable { op_code, push_flag, op_bits, op_acc, stack, extension_factor };
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
        for i in 0..self.stack.len() {
            state.set_stack_value(i, self.stack[i][step]);
        }
    }

    /// Returns the number of states in the un-extended trace table.
    pub fn unextended_length(&self) -> usize {
        return self.op_code.capacity() / self.extension_factor();
    }

    /// Returns the number of states in the extended trace table.
    pub fn domain_size(&self) -> usize {
        return self.op_code.capacity();
    }

    /// Returns `extension_factor` for the trace table.
    pub fn extension_factor(&self) -> usize {
        return self.extension_factor;
    }

    /// Returns the number of registers in the trace table.
    pub fn register_count(&self) -> usize {
        return 1 + self.op_bits.len() + self.op_acc.len() + self.stack.len();
    }

    /// Returns the number of registers used by the stack.
    pub fn max_stack_depth(&self) -> usize {
        return self.stack.len();
    }

    /// Returns register trace at the specified `index`.
    pub fn get_register_trace(&self, index: usize) -> &[u64] {
        return match index {
            0     => &self.op_code,
            1..=5 => &self.op_bits[index - 1],
            _     => &self.stack[index - 6]
        };
    }

    /// Returns register trace for stack register specified by the `index`.
    pub fn get_stack_register_trace(&self, index: usize) -> &[u64] {
        return &self.stack[index];
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
        let domain_size = self.domain_size();

        // build vectors of twiddles and inv_twiddles needed for FFT
        let root = field::get_root_of_unity(self.unextended_length() as u64);
        let inv_twiddles = fft::get_inv_twiddles(root, self.unextended_length());
        let root = field::get_root_of_unity(domain_size as u64);
        let twiddles = fft::get_twiddles(root, domain_size);

        // extend op_code
        debug_assert!(self.op_code.capacity() == domain_size, "invalid op_code register capacity");
        polys::interpolate_fft_twiddles(&mut self.op_code, &inv_twiddles, true);
        unsafe { self.op_code.set_len(self.op_code.capacity()); }
        polys::eval_fft_twiddles(&mut self.op_code, &twiddles, true);

        // extend push_flag
        debug_assert!(self.push_flag.capacity() == domain_size, "invalid push_flag register capacity");
        polys::interpolate_fft_twiddles(&mut self.push_flag, &inv_twiddles, true);
        unsafe { self.push_flag.set_len(self.push_flag.capacity()); }
        polys::eval_fft_twiddles(&mut self.push_flag, &twiddles, true);

        // extend op_bits
        for register in self.op_bits.iter_mut() {
            debug_assert!(register.capacity() == domain_size, "invalid op_bits register capacity");
            polys::interpolate_fft_twiddles(register, &inv_twiddles, true);
            unsafe { register.set_len(register.capacity()); }
            polys::eval_fft_twiddles(register, &twiddles, true);
        }

        // extend op_acc
        for register in self.op_acc.iter_mut() {
            debug_assert!(register.capacity() == domain_size, "invalid op_acc register capacity");
            polys::interpolate_fft_twiddles(register, &inv_twiddles, true);
            unsafe { register.set_len(register.capacity()); }
            polys::eval_fft_twiddles(register, &twiddles, true);
        }

        // extend stack registers
        for register in self.stack.iter_mut() {
            debug_assert!(register.capacity() == domain_size, "invalid stack register capacity");
            polys::interpolate_fft_twiddles(register, &inv_twiddles, true);
            unsafe { register.set_len(register.capacity()); }
            polys::eval_fft_twiddles(register, &twiddles, true);
        }
    }
}