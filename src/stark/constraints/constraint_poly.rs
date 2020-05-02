use crate::crypto::{ HashFunction, MerkleTree };
use crate::math::{ field, fft, polynom, quartic::to_quartic_vec };
use crate::stark::{ utils::CompositionCoefficients };

// TYPES AND INTERFACES
// ================================================================================================
pub struct ConstraintPoly {
    domain      : Vec<u64>,
    poly        : Vec<u64>,
}

// CONSTRAINT POLY IMPLEMENTATION
// ================================================================================================
impl ConstraintPoly {

    pub fn new(poly: Vec<u64>, domain: Vec<u64>) -> ConstraintPoly {

        assert!(poly.len().is_power_of_two(), "poly length must be a power of two");
        assert!(domain.len().is_power_of_two(), "domain size must be a power of two");
        assert!(domain.len() > poly.len(), "domain size must be greater than poly length");

        return ConstraintPoly { domain, poly };
    }

    pub fn domain(&self) -> &[u64] {
        return &self.domain;
    }

    pub fn degree(&self) -> usize {
        return polynom::degree_of(&self.poly);
    }

    pub fn to_merkle_tree(&self, hash: HashFunction) -> MerkleTree {

        let domain_root = field::get_root_of_unity(self.domain.len() as u64);
        let twiddles = fft::get_twiddles(domain_root, self.domain.len());
    
        // evaluate constraint polynomial over the evaluation domain
        let mut evaluations = vec![0; self.domain.len()];
        evaluations[..self.poly.len()].copy_from_slice(&self.poly);
        polynom::eval_fft_twiddles(&mut evaluations, &twiddles, true);

        // put evaluations into a Merkle tree; 4 evaluations per leaf
        let evaluations = to_quartic_vec(evaluations);
        return MerkleTree::new(evaluations, hash);
    }

    pub fn get_composition_poly(&self, z: u64, cc: &CompositionCoefficients) -> Vec<u64> {
        // TODO: implement
        return vec![];
    }
}