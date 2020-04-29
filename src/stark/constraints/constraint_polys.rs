use crate::math::{ field, polynom, fft, quartic::to_quartic_vec };
use crate::crypto::{ MerkleTree, HashFunction };
use crate::utils::{ zero_filled_vector, uninit_vector };
use super::{ MAX_CONSTRAINT_DEGREE };

// TYPES AND INTERFACES
// ================================================================================================
pub struct ConstraintPolys {
    domain      : Vec<u64>,
    polys       : [Vec<u64>; MAX_CONSTRAINT_DEGREE],
    evaluations : [Vec<u64>; MAX_CONSTRAINT_DEGREE],
}

// CONSTRAINT POLYS IMPLEMENTATION
// ================================================================================================
impl ConstraintPolys {

    pub fn new(polys: [Vec<u64>; MAX_CONSTRAINT_DEGREE], domain: Vec<u64>) -> ConstraintPolys {

        let trace_length = polys[0].len();
        let domain_size = domain.len();

        // allocate space for polynomial evaluations
        let mut evaluations = [
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
            zero_filled_vector(trace_length, domain_size),
        ];

        // copy polynomials into the evaluations
        for i in 0..polys.len() {
            evaluations[i].copy_from_slice(&polys[i]);
        }

        return ConstraintPolys { domain, polys, evaluations };
    }

    pub fn poly_count(&self) -> usize {
        return self.polys.len();
    }

    pub fn domain(&self) -> &[u64] {
        return &self.domain;
    }

    pub fn is_evaluated(&self) -> bool {
        return self.evaluations[0].len() == self.evaluations[0].capacity();
    }

    pub fn evaluate(&mut self) {
        assert!(!self.is_evaluated(), "constraint polynomials have already been evaluated");

        let domain_size = self.domain.len();
        let root = field::get_root_of_unity(domain_size as u64);
        let twiddles = fft::get_twiddles(root, domain_size);

        for poly in self.evaluations.iter_mut() {
            debug_assert!(poly.capacity() == domain_size, "invalid capacity constraint polynomial evaluation");
            unsafe { poly.set_len(poly.capacity()); }
            polynom::eval_fft_twiddles(poly, &twiddles, true);
        }
    }

    pub fn evaluate_at(&self, z: u64) -> Vec<u64> {
        let mut result = Vec::new();
        for poly in self.polys.iter() {
            result.push(polynom::eval(poly, z));
        }
        return result;
    }

    pub fn to_merkle_tree(&self, hash: HashFunction) -> MerkleTree {
        assert!(self.is_evaluated(), "constraint polynomials haven't been evaluated yet");
        let domain_size = self.domain.len();
        let mut values = [0; MAX_CONSTRAINT_DEGREE];
        let mut hashed_values = to_quartic_vec(uninit_vector(domain_size * 4));
        // TODO: this loop should be parallelized
        for i in 0..self.domain.len() {
            for j in 0..values.len() {
                values[j] = self.evaluations[j][i];
            }
            hash(&values, &mut hashed_values[i]);
        }
        return MerkleTree::new(hashed_values, hash);
    }
}