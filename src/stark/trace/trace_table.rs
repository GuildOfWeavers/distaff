use crate::math::{ FiniteField, fft, polynom, parallel };
use crate::crypto::{ MerkleTree, HashFunction };
use crate::stark::{ CompositionCoefficients, utils };
use crate::utils::{ uninit_vector, filled_vector, as_bytes };
use crate::{ MAX_REGISTER_COUNT, DECODER_WIDTH, PROG_HASH_RANGE, MIN_EXTENSION_FACTOR };
use super::{ TraceState };

// TYPES AND INTERFACES
// ================================================================================================
pub struct TraceTable {
    registers       : Vec<Vec<u128>>,
    polys           : Vec<Vec<u128>>,
    trace_length    : usize,
    extension_factor: usize,
}

// TRACE TABLE IMPLEMENTATION
// ================================================================================================
impl TraceTable {
    /// Returns a trace table constructed from the specified register traces.
    pub fn new(registers: Vec<Vec<u128>>, extension_factor: usize) -> TraceTable {

        // validate extension factor
        assert!(extension_factor.is_power_of_two(), "trace extension factor must be a power of 2");
        assert!(extension_factor >= MIN_EXTENSION_FACTOR,
            "extension factor must be at least {}", MIN_EXTENSION_FACTOR);

        // validate register traces
        assert!(registers.len() < MAX_REGISTER_COUNT,
            "execution trace cannot have more than {} registers", MAX_REGISTER_COUNT);
        let trace_length = registers[0].len();
        assert!(trace_length.is_power_of_two(), "execution trace length must be a power of 2");
        for register in registers.iter() {
            assert!(register.len() == trace_length, "all register traces must have the same length");
        }

        let polys = Vec::with_capacity(registers.len());
        return TraceTable { registers, polys, trace_length, extension_factor };
    }

    /// Returns hash value of the executed program.
    pub fn get_program_hash(&self) -> Vec<u128> {
        let last_step = if self.is_extended() {
            self.domain_size() - self.extension_factor()
        }
        else {
            self.unextended_length() - 1
        };

        let mut result = vec![u128::ZERO; PROG_HASH_RANGE.end - PROG_HASH_RANGE.start];
        for (i, j) in PROG_HASH_RANGE.enumerate() {
            result[i] = self.registers[j][last_step];
        }
        return result;
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
        return self.trace_length;
    }

    /// Returns the number of states in the extended trace table.
    pub fn domain_size(&self) -> usize {
        return self.trace_length * self.extension_factor;
    }

    /// Returns `extension_factor` for the trace table.
    pub fn extension_factor(&self) -> usize {
        return self.extension_factor;
    }

    /// Returns the number of registers in the trace table.
    pub fn register_count(&self) -> usize {
        return self.registers.len();
    }

    /// Returns the number of registers used by the stack.
    pub fn max_stack_depth(&self) -> usize {
        return self.registers.len() - DECODER_WIDTH;
    }

    /// Returns polynomial of the register at the specified `index`; can be called only
    /// after the trace table has been extended.
    #[cfg(test)]
    pub fn get_register_poly(&self, index: usize) -> &[u128] {
        assert!(self.is_extended(), "trace table has not been extended yet");
        return &self.polys[index];
    }

    /// Returns values of all registers at the specified `positions`.
    pub fn get_register_values_at(&self, positions: &[usize]) -> Vec<Vec<u128>> {
        let mut result = Vec::with_capacity(positions.len());
        for &i in positions.iter() {
            let row = self.registers.iter().map(|r| r[i]).collect();
            result.push(row);
        }
        return result;
    }

    /// Returns `true` if the trace table has been extended.
    pub fn is_extended(&self) -> bool {
        return self.registers[0].len() > self.trace_length;
    }

    /// Extends all registers of the trace table by the `extension_factor` specified during
    /// trace table construction. A trace table can be extended only once.
    pub fn extend(&mut self, twiddles: &[u128]) {
        assert!(!self.is_extended(), "trace table has already been extended");
        assert!(twiddles.len() * 2 == self.domain_size(), "invalid number of twiddles");

        // build inverse twiddles needed for FFT interpolation
        let root = u128::get_root_of_unity(self.unextended_length());
        let inv_twiddles = fft::get_inv_twiddles(root, self.unextended_length());
        
        // move register traces into polys
        std::mem::swap(&mut self.registers, &mut self.polys);

        // extend all registers
        let domain_size = self.domain_size();
        for poly in self.polys.iter_mut() {

            // interpolate register trace into a polynomial
            polynom::interpolate_fft_twiddles(poly, &inv_twiddles, true);
            
            // allocate space to hold extended evaluations and copy the polynomial into it
            let mut register = vec![u128::ZERO; domain_size];
            register[..poly.len()].copy_from_slice(&poly);
            
            // evaluate the polynomial over extended domain
            polynom::eval_fft_twiddles(&mut register, &twiddles, true);
            self.registers.push(register);
        }
    }

    /// Puts the trace table into a Merkle tree such that each state of the table becomes
    /// a distinct leaf in the tree; all registers at a given step are hashed together to
    /// form a single leaf value.
    pub fn build_merkle_tree(&self, hash: HashFunction) -> MerkleTree {
        let mut trace_state = vec![u128::ZERO; self.register_count()];
        let mut hashed_states = uninit_vector::<[u8; 32]>(self.domain_size());
        // TODO: this loop should be parallelized
        for i in 0..self.domain_size() {
            for j in 0..trace_state.len() {
                trace_state[j] = self.registers[j][i];
            }
            hash(as_bytes(&trace_state), &mut hashed_states[i]);
        }
        return MerkleTree::new(hashed_states, hash);
    }

    /// Evaluates trace polynomials at the specified point `z`; can be called only after
    /// the trace table has been extended
    pub fn eval_polys_at(&self, z: u128) -> Vec<u128> {
        assert!(self.is_extended(), "trace table has not been extended yet");

        let mut result = Vec::new();
        for poly in self.polys.iter() {
            result.push(polynom::eval(poly, z));
        }
        return result;
    }

    /// Combines trace polynomials for all registers into a single composition polynomial.
    /// The combination is done as follows:
    /// 1. First, state of trace registers at deep points z and z * g are computed;
    /// 2. Then, polynomials T1_i(x) = (T_i(x) - T_i(z)) / (x - z) and 
    /// T2_i(x) = (T_i(x) - T_i(z * g)) / (x - z * g) are computed for all i and combined
    /// together into a single polynomial using a pseudo-random linear combination;
    /// 3. Then the degree of the polynomial is adjusted to match the specified degree
    pub fn get_composition_poly(&self, z: u128, cc: &CompositionCoefficients) -> (Vec<u128>, Vec<u128>, Vec<u128>) {

        let trace_length = self.unextended_length();
        assert!(self.is_extended(), "trace table has not been extended yet");
        
        let g = u128::get_root_of_unity(trace_length);
        let next_z = u128::mul(z, g);

        // compute state of registers at deep points z and z * g
        let trace_state1 = self.eval_polys_at(z);
        let trace_state2 = self.eval_polys_at(next_z);

        let mut t1_composition = vec![u128::ZERO; trace_length];
        let mut t2_composition = vec![u128::ZERO; trace_length];

        // combine trace polynomials into 2 composition polynomials T1(x) and T2(x)
        for i in 0..self.polys.len() {
            // compute T1(x) = (T(x) - T(z)), multiply it by a pseudo-random coefficient,
            // and add the result into composition polynomial
            parallel::mul_acc(&mut t1_composition, &self.polys[i], cc.trace1[i], 1);
            let adjusted_tz = u128::mul(trace_state1[i], cc.trace1[i]);
            t1_composition[0] = u128::sub(t1_composition[0], adjusted_tz);

            // compute T2(x) = (T(x) - T(z * g)), multiply it by a pseudo-random
            // coefficient, and add the result into composition polynomial
            parallel::mul_acc(&mut t2_composition, &self.polys[i], cc.trace2[i], 1);
            let adjusted_tz = u128::mul(trace_state2[i], cc.trace2[i]);
            t2_composition[0] = u128::sub(t2_composition[0], adjusted_tz);
        }

        // divide the two composition polynomials by (x - z) and (x - z * g)
        // respectively and add the resulting polynomials together
        polynom::syn_div_in_place(&mut t1_composition, z);
        polynom::syn_div_in_place(&mut t2_composition, next_z);
        parallel::add_in_place(&mut t1_composition, &t2_composition, 1);

        // adjust the degree of the polynomial to match the degree parameter by computing
        // C(x) = T(x) * k_1 + T(x) * x^incremental_degree * k_2
        let poly_size = utils::get_composition_degree(trace_length).next_power_of_two();
        let mut composition_poly = filled_vector(poly_size, self.domain_size(), u128::ZERO);
        let incremental_degree = utils::get_incremental_trace_degree(trace_length);
        // this is equivalent to T(x) * k_1
        parallel::mul_acc(
            &mut composition_poly[..trace_length],
            &t1_composition,
            cc.t1_degree,
            1);
        // this is equivalent to T(x) * x^incremental_degree * k_2
        parallel::mul_acc(
            &mut composition_poly[incremental_degree..(incremental_degree + trace_length)],
            &t1_composition,
            cc.t2_degree,
            1);
        
        return (composition_poly, trace_state1, trace_state2);
    }
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {

    use crate::math::{ F128, FiniteField, polynom, parallel, fft };
    use crate::{ crypto::hash::blake3, processor::opcodes::f128 as opcodes };
    use crate::programs::{ Program, ProgramInputs };
    use crate::processor::{ execute };
    use crate::stark::{ TraceTable, CompositionCoefficients, MAX_CONSTRAINT_DEGREE };
    
    const EXT_FACTOR: usize = 32;

    #[test]
    fn eval_polys_at() {
        let mut trace = build_trace_table();
        let lde_root = F128::get_root_of_unity(trace.domain_size());
        trace.extend(&fft::get_twiddles(lde_root, trace.domain_size()));

        let g = F128::get_root_of_unity(trace.unextended_length());

        let v1 = trace.eval_polys_at(g);
        let s1 = trace.get_state(1 * EXT_FACTOR);
        assert_eq!(v1, s1.registers());

        let v2 = trace.eval_polys_at(F128::exp(g, 2));
        let s2 = trace.get_state(2 * EXT_FACTOR);
        assert_eq!(v2, s2.registers());
    }

    #[test]
    fn get_composition_poly() {

        let mut trace = build_trace_table();
        let lde_root = F128::get_root_of_unity(trace.domain_size());
        trace.extend(&fft::get_twiddles(lde_root, trace.domain_size()));

        // compute trace composition polynomial
        let t_tree = trace.build_merkle_tree(blake3);
        let z = F128::prng(*t_tree.root());
        let cc = CompositionCoefficients::new(*t_tree.root());
        let target_degree = (trace.unextended_length() - 2) * MAX_CONSTRAINT_DEGREE - 1;

        let g = F128::get_root_of_unity(trace.unextended_length());
        let zg = F128::mul(z, g);

        let (composition_poly, ..) = trace.get_composition_poly(z, &cc);
        let mut actual_evaluations = composition_poly.clone();
        polynom::eval_fft(&mut actual_evaluations, true);
        assert_eq!(target_degree, polynom::infer_degree(&actual_evaluations));

        // compute expected evaluations
        let domain_size = target_degree.next_power_of_two();
        let domain_root = F128::get_root_of_unity(domain_size);
        let domain = F128::get_power_series(domain_root, domain_size);

        let mut expected_evaluations = vec![0; domain_size];

        let tz = trace.eval_polys_at(z);
        let tzg = trace.eval_polys_at(zg);

        for i in 0..trace.register_count() {
            // add T1(x) to expected evaluations
            let mut trace_poly = trace.get_register_poly(i).to_vec();
            trace_poly.resize(domain_size, 0);
            polynom::eval_fft(&mut trace_poly, true);
            parallel::sub_const_in_place(&mut trace_poly, tz[i], 1);
            for j in 0..trace_poly.len() {
                trace_poly[j] = F128::div(trace_poly[j], F128::sub(domain[j], z));
            }
            parallel::mul_acc(&mut expected_evaluations, &trace_poly, cc.trace1[i], 1);

            // add T2(x) to expected evaluations
            let mut trace_poly = trace.get_register_poly(i).to_vec();
            trace_poly.resize(domain_size, 0);
            polynom::eval_fft(&mut trace_poly, true);
            parallel::sub_const_in_place(&mut trace_poly, tzg[i], 1);
            for j in 0..trace_poly.len() {
                trace_poly[j] = F128::div(trace_poly[j], F128::sub(domain[j], zg));
            }
            parallel::mul_acc(&mut expected_evaluations, &trace_poly, cc.trace2[i], 1);
        }

        // raise degree
        let incremental_degree = target_degree - (trace.unextended_length() - 2);
        for i in 0..domain.len() {
            let y = expected_evaluations[i];
            let y1 = F128::mul(y, cc.t1_degree);

            let xp = F128::exp(domain[i], F128::from_usize(incremental_degree));
            let y2 = F128::mul(F128::mul(y, xp), cc.t2_degree);
            expected_evaluations[i] = F128::add(y1, y2);
        }

        assert_eq!(expected_evaluations, actual_evaluations);
    }

    fn build_trace_table() -> TraceTable {
        let program = Program::from_path(vec![
            opcodes::BEGIN, opcodes::SWAP, opcodes::DUP2, opcodes::DROP,
            opcodes::ADD,   opcodes::SWAP, opcodes::DUP2, opcodes::DROP,
            opcodes::ADD,   opcodes::SWAP, opcodes::DUP2, opcodes::DROP,
            opcodes::ADD,   opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
        ]);
        let inputs = ProgramInputs::from_public(&[1, 0]);
        let registers = execute(&program, &inputs);
        return TraceTable::new(registers, EXT_FACTOR);
    }
}