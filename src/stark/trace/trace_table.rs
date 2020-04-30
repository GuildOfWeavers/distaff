use crate::math::{ field, fft, polynom, quartic::to_quartic_vec };
use crate::crypto::{ MerkleTree, HashFunction };
use crate::processor::opcodes;
use crate::utils::uninit_vector;
use super::{ TraceState, decoder, stack, MAX_REGISTER_COUNT };

// TYPES AND INTERFACES
// ================================================================================================
pub struct TraceTable {
    registers   : Vec<Vec<u64>>,
    polys       : Vec<Vec<u64>>,
    ext_factor  : usize,
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

        // create different segments of the trace
        let decoder_registers = decoder::process(program, extension_factor);
        let stack_registers = stack::execute(program, inputs, extension_factor);

        // move all trace registers into a single vector
        let mut registers = Vec::new();
        for register in decoder_registers.into_iter() { registers.push(register); }
        for register in stack_registers.into_iter() { registers.push(register); }

        assert!(registers.len() < MAX_REGISTER_COUNT,
            "execution trace cannot have more than {} registers", MAX_REGISTER_COUNT);

        let polys = Vec::with_capacity(registers.len());
        return TraceTable { registers, polys, ext_factor: extension_factor };
    }

    /// Returns state of the trace table at the specified `step`.
    pub fn get_state(&self, step: usize) -> TraceState {
        let mut result = TraceState::new(self.max_stack_depth());
        self.fill_state(&mut result, step);
        return result;
    }

    /// Copies trace table state at the specified `step` to the passed in `state` object.
    pub fn fill_state(&self, state: &mut TraceState, step: usize) {
        for i in 0..self.registers.len() {
            state.set_register(i, self.registers[i][step]);
        }
    }

    /// Returns the number of states in the un-extended trace table.
    pub fn unextended_length(&self) -> usize {
        return self.registers[0].capacity() / self.ext_factor;
    }

    /// Returns the number of states in the extended trace table.
    pub fn domain_size(&self) -> usize {
        return self.registers[0].capacity();
    }

    /// Returns `extension_factor` for the trace table.
    pub fn extension_factor(&self) -> usize {
        return self.ext_factor;
    }

    /// Returns the number of registers in the trace table.
    pub fn register_count(&self) -> usize {
        return self.registers.len();
    }

    /// Returns the number of registers used by the stack.
    pub fn max_stack_depth(&self) -> usize {
        return self.registers.len() - decoder::NUM_REGISTERS;
    }

    /// Returns trace of the register at the specified `index`.
    pub fn get_register_trace(&self, index: usize) -> &[u64] {
        return &self.registers[index];
    }

    /// Returns polynomial of the register at the specified `index`; can be called only
    /// after the trace table has been extended.
    pub fn get_register_poly(&self, index: usize) -> &[u64] {
        assert!(self.is_extended(), "trace table has not been extended yet");
        return &self.polys[index];
    }

    /// Returns trace of the stack register at the specified `index`.
    pub fn get_stack_register_trace(&self, index: usize) -> &[u64] {
        return &self.registers[index + decoder::NUM_REGISTERS];
    }

    /// Returns `true` if the trace table has been extended.
    pub fn is_extended(&self) -> bool {
        return self.registers[0].len() == self.registers[0].capacity();
    }

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

        // extend all registers
        for register in self.registers.iter_mut() {
            debug_assert!(register.capacity() == domain_size, "invalid capacity for register");
            // interpolate register trace into a polynomial
            polynom::interpolate_fft_twiddles(register, &inv_twiddles, true);

            // save the polynomial for later use
            self.polys.push(register.clone());

            // evaluate the polynomial over extended domain
            unsafe { register.set_len(register.capacity()); }
            polynom::eval_fft_twiddles(register, &twiddles, true);
        }
    }

    /// Puts the trace table into a Merkle tree such that each state of the table becomes
    /// a distinct leaf in the tree; all registers at a given step are hashed together to
    /// form a single leaf value.
    pub fn to_merkle_tree(&self, hash: HashFunction) -> MerkleTree {
        let mut trace_state = vec![0; self.register_count()];
        let mut hashed_states = to_quartic_vec(uninit_vector(self.domain_size() * 4));
        // TODO: this loop should be parallelized
        for i in 0..self.domain_size() {
            for j in 0..trace_state.len() {
                trace_state[j] = self.registers[j][i];
            }
            hash(&trace_state, &mut hashed_states[i]);
        }
        return MerkleTree::new(hashed_states, hash);
    }

    /// Evaluates trace polynomials at the specified point `z`; can be called only after
    /// the trace table has been extended
    pub fn eval_polys_at(&self, z: u64) -> Vec<u64> {
        assert!(self.is_extended(), "trace table has not been extended yet");

        let mut result = Vec::new();
        for poly in self.polys.iter() {
            result.push(polynom::eval(poly, z));
        }
        return result;
    }
}