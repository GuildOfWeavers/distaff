use crate::crypto::{ HashFunction, MerkleTree };
use crate::math::{ field, fft, polynom, quartic::to_quartic_vec };
use crate::stark::{ utils::CompositionCoefficients, MAX_CONSTRAINT_DEGREE };

// TYPES AND INTERFACES
// ================================================================================================
pub struct ConstraintPoly {
    domain      : Vec<u64>,
    poly        : Vec<u64>,
    degree      : usize,
}

// CONSTRAINT POLY IMPLEMENTATION
// ================================================================================================
impl ConstraintPoly {

    pub fn new(poly: Vec<u64>, domain: Vec<u64>) -> ConstraintPoly {

        assert!(poly.len().is_power_of_two(), "poly length must be a power of two");
        assert!(domain.len().is_power_of_two(), "domain size must be a power of two");
        assert!(domain.len() > poly.len(), "domain size must be greater than poly length");

        let trace_length = poly.len() / MAX_CONSTRAINT_DEGREE;
        let degree = poly.len() - trace_length;
        assert!(degree == polynom::degree_of(&poly),
            "expected polynomial of degree {} but received degree {}",
            degree, polynom::degree_of(&poly));

        return ConstraintPoly { domain, poly, degree };
    }

    pub fn domain(&self) -> &[u64] {
        return &self.domain;
    }

    pub fn degree(&self) -> usize {
        return self.degree;
    }

    pub fn to_merkle_tree(&self, hash: HashFunction) -> MerkleTree {

        let domain_root = field::get_root_of_unity(self.domain.len() as u64);
        let twiddles = fft::get_twiddles(domain_root, self.domain.len());   // TODO: don't re-build twiddles
    
        // evaluate constraint polynomial over the evaluation domain
        let mut evaluations = vec![0; self.domain.len()];
        evaluations[..self.poly.len()].copy_from_slice(&self.poly);
        polynom::eval_fft_twiddles(&mut evaluations, &twiddles, true);

        // put evaluations into a Merkle tree; 4 evaluations per leaf
        let evaluations = to_quartic_vec(evaluations);
        return MerkleTree::new(evaluations, hash);
    }

    pub fn eval_at(&self, z: u64) -> u64 {
        return polynom::eval(&self.poly, z);
    }

    pub fn get_composition_poly(&self, z: u64, cc: &CompositionCoefficients) -> Vec<u64> {
        
        let mut composition_poly = self.poly.clone();

        // compute C(x) = (A(x) - A(z)) / (x - z)
        let z_value = polynom::eval(&self.poly, z);
        composition_poly[0] = field::sub(composition_poly[0], z_value);
        polynom::syn_div_in_place(&mut composition_poly, z);

        // TODO: parallelize
        for i in 0..composition_poly.len() {
            composition_poly[i] = field::mul(composition_poly[i], cc.constraints);
        }

        return composition_poly;
    }
}