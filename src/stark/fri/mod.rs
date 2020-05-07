use crate::crypto::{ BatchMerkleProof };
use serde::{ Serialize, Deserialize };

// RE-EXPORTS
// ================================================================================================
mod prover;
pub use prover::{ reduce, build_proof };

mod verifier;
pub use verifier::{ verify };

const MAX_REMAINDER_LENGTH: usize = 256;

// TYPES AND INTERFACES
// ================================================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriProof {
    pub layers      : Vec<FriLayer>,
    pub rem_root    : [u64; 4],
    pub rem_values  : Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriLayer {
    pub root : [u64; 4],
    pub proof: BatchMerkleProof,
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    use crate::{ field, polynom, ProofOptions };
    use crate::stark::utils::compute_query_positions;

    #[test]
    fn prove_verify() {
        let degree_plus_1: usize = 64;
        let domain_size: usize = 512;
        let root = field::get_root_of_unity(domain_size as u64);
        let domain = field::get_power_series(root, domain_size);
        let options = ProofOptions::default();

        let evaluations = build_random_poly_evaluations(domain_size, degree_plus_1 - 1);

        // generate proof
        let fri_layers = super::reduce(&evaluations, &domain, &options);
        let positions = compute_query_positions(fri_layers[fri_layers.len() - 1].root(), domain_size, &options);
        let proof = super::build_proof(fri_layers, &positions);

        // verify proof
        let sampled_evaluations = positions.iter().map(|&i| evaluations[i]).collect::<Vec<u64>>();
        let result = super::verify(&proof, &sampled_evaluations, &positions, root, degree_plus_1, &options);
        assert_eq!(Ok(true), result);
    }

    #[test]
    fn verify_fail() {

        let degree_plus_1: usize = 64;
        let domain_size: usize = 512;
        let root = field::get_root_of_unity(domain_size as u64);
        let domain = field::get_power_series(root, domain_size);
        let options = ProofOptions::default();

        let evaluations = build_random_poly_evaluations(domain_size, degree_plus_1 - 1);
        
        // generate proof
        let fri_layers = super::reduce(&evaluations, &domain, &options);
        let positions = compute_query_positions(fri_layers[fri_layers.len() - 1].root(), domain_size, &options);
        let proof = super::build_proof(fri_layers, &positions);

        // degree too low
        let sampled_evaluations = positions.iter().map(|&i| evaluations[i]).collect::<Vec<u64>>();

        let result = super::verify(&proof, &sampled_evaluations, &positions, root, degree_plus_1 - 1, &options);
        let err_msg = format!("remainder is not a valid degree {} polynomial", 14);
        assert_eq!(Err(err_msg), result);

        // invalid evaluations
        let sampled_evaluations = sampled_evaluations[1..].to_vec();
        let result = super::verify(&proof, &sampled_evaluations, &positions, root, degree_plus_1, &options);
        let err_msg = format!("evaluations did not match column value at depth 0");
        assert_eq!(Err(err_msg), result);
    }

    // TODO: add more tests

    fn build_random_poly_evaluations(domain_size: usize, degree: usize) -> Vec<u64> {
        let mut evaluations = field::rand_vector(degree + 1);
        evaluations.resize(domain_size, 0);
        polynom::eval_fft(&mut evaluations, true);
        return evaluations;
    }
}