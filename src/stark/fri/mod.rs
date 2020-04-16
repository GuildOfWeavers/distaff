use std::mem;
use crate::math::{ field, polys, quartic };
use crate::crypto::{ MerkleTree, BatchMerkleProof };
use crate::stark::{ ProofOptions, utils::QueryIndexGenerator };

mod proof;
pub use proof::{ FriProof, FriLayer };

// CONSTANTS
// ================================================================================================
const MAX_REMAINDER_LENGTH: usize = 256;

// PROVER
// ================================================================================================

/// Uses FRI protocol to prove that the polynomial which evaluates to `evaluations` over the provided
/// `domain` has degree at most `max_degree_plus_1` - 1. 
pub fn prove(evaluations: &[u64], domain: &[u64], max_degree_plus_1: usize, options: &ProofOptions) -> FriProof {

    assert!(domain.len().is_power_of_two(), "domain length must be a power of 2");
    assert!(evaluations.len() == domain.len(), "evaluations and domain slices must have the same length");
    assert!(max_degree_plus_1.is_power_of_two(), "max_degree_plus_1 must be a power of 2");
    assert!(max_degree_plus_1 < domain.len(), "domain length must be greater than max_degree_plus_1");
    assert!(domain.len() / max_degree_plus_1 < MAX_REMAINDER_LENGTH, "degree is too big for the domain");

    let idx_generator = QueryIndexGenerator::new(options);
    
    // 1 ----- initialize the proof object --------------------------------------------------------

    // transpose evaluations into a matrix with 4 columns and
    // put its rows as leaves into a Merkle tree
    let poly_values = quartic::transpose(evaluations, 1);
    let mut p_tree = MerkleTree::new(poly_values, options.hash_function());

    // build a Merkle proof against this tree choosing indexes like so:
    // first, generate indexes at the positions which will be later used to query the trace tree;
    // then, map these indexes to corresponding indexes of p_tree (the indexes need to be mapped
    // because p_tree is build against transposed evaluations)
    let positions = idx_generator.get_trace_indexes(p_tree.root(), evaluations.len());
    let augmented_positions = get_augmented_positions(&positions, evaluations.len());
    let ev_proof = p_tree.prove_batch(&augmented_positions);

    // initialize the proof object with the root of the tree and the proof
    let mut proof = FriProof::new(p_tree.root(), ev_proof);

    // 2 ----- generate recursive components of FRI proof -----------------------------------------
    // reduce the degree by 4 at each iteration until the remaining polynomial is small enough
    let mut max_degree_plus_1 = max_degree_plus_1;
    while p_tree.leaves().len() * 4 > MAX_REMAINDER_LENGTH {

        // build polynomials from each row of the polynomial value matrix
        let depth = proof.layers.len() as u32;
        let xs = quartic::transpose(domain, usize::pow(4, depth));
        let polys = quartic::interpolate_batch(&xs, p_tree.leaves());

        // select a pseudo-random x coordinate and evaluate each row polynomial at that coordinate
        let special_x = field::prng(to_bytes(p_tree.root()));
        let column = quartic::evaluate_batch(&polys, special_x);

        // break the column in a polynomial value matrix for the next layer
        let new_poly_values = quartic::transpose(&column, 1);

        // put the resulting matrix into a Merkle tree
        let mut c_tree = MerkleTree::new(new_poly_values, options.hash_function());

        // compute query positions in the column and corresponding positions in the original values
        let positions = idx_generator.get_fri_indexes(c_tree.root(), column.len());
        let augmented_positions = get_augmented_positions(&positions, column.len());

        // add FRI layer to the proof
        proof.layers.push(FriLayer { 
            column_root : *c_tree.root(),
            column_proof: c_tree.prove_batch(&augmented_positions),
            poly_proof  : p_tree.prove_batch(&positions)
        });

        // update variables for the next iteration of the loop
        max_degree_plus_1 = max_degree_plus_1 / 4;
        mem::swap(&mut c_tree, &mut p_tree);
    }

    // 3 ----- use the remaining polynomial values directly as proof ------------------------------
    // first "un-transpose" the values of the remainder
    let remainder = p_tree.leaves();
    proof.remainder.resize(remainder.len() * 4, 0);
    for i in 0..remainder.len() {
        proof.remainder[i] = remainder[i][0];
        proof.remainder[i + remainder.len()] = remainder[i][1];
        proof.remainder[i + remainder.len() * 2] = remainder[i][2];
        proof.remainder[i + remainder.len() * 3] = remainder[i][3];
    }

    // make sure that the remainder polynomial actually satisfies the degree
    let depth = proof.layers.len() as u32;
    let root = field::exp(domain[1], u64::pow(4, depth));
    verify_remainder(&proof.remainder, max_degree_plus_1, root, options.extension_factor()).unwrap();

    return proof;
}

// VERIFIER
// ================================================================================================

/// Verifies that a polynomial which evaluates to a small number of the provided `evaluations`
/// has degree at most `max_degree_plus_1` - 1.
pub fn verify(proof: &FriProof, evaluations: &[u64], root: u64, max_degree_plus_1: usize, options: &ProofOptions) -> Result<bool, String>
{

    let domain_size = get_root_of_unity_degree(root);
    let idx_generator = QueryIndexGenerator::new(options);

    // powers of the given root of unity 1, p, p^2, p^3 such that p^4 = 1
    let quartic_roots = [
        1,
        field::exp(root, (domain_size / 4) as u64),
        field::exp(root, (domain_size / 2) as u64),
        field::exp(root, (domain_size * 3 / 4) as u64),
    ];

    // 1 ----- check correctness of evaluation tree ----------------------------------------------
    let positions = idx_generator.get_trace_indexes(&proof.ev_root, domain_size);
    let augmented_positions = get_augmented_positions(&positions, domain_size);
    if !MerkleTree::verify_batch(&proof.ev_root, &augmented_positions, &proof.ev_proof, options.hash_function()) {
        return Err(String::from("Verification of evaluation Merkle proof failed"));
    }

    let ev_checks = get_column_values(&proof.ev_proof, &positions, &augmented_positions, domain_size);
    if evaluations != &ev_checks[..] {
        return Err(String::from("Verification of evaluation values failed"));
    }

    // 2 ----- verify the recursive components of the FRI proof -----------------------------------
    // make the variables mutable
    let mut root = root;
    let mut p_root = proof.ev_root;
    let mut column_length = domain_size / 4;
    let mut max_degree_plus_1 = max_degree_plus_1;

    for (depth, FriLayer { column_root, column_proof, poly_proof }) in (&proof.layers).into_iter().enumerate() {

        // calculate pseudo-random indexes for column and poly values
        let positions = idx_generator.get_fri_indexes(column_root, column_length);
        let augmented_positions = get_augmented_positions(&positions, column_length);

        // verify Merkle proof for the column
        if !MerkleTree::verify_batch(&column_root, &augmented_positions, &column_proof, options.hash_function()) {
            return Err(format!("Verification of column Merkle proof failed at depth {}", depth));
        }

        // verify Merkle proof for polynomials
        if !MerkleTree::verify_batch(&p_root, &positions, &poly_proof, options.hash_function()) {
            return Err(format!("Verification of polynomial Merkle proof failed at depth {}", depth));
        }

        // build a set of x and y coordinates for each row polynomial
        let mut xs = Vec::with_capacity(positions.len());
        for &i in positions.iter() {
            let xe = field::exp(root, i as u64);
            xs.push([
                field::mul(quartic_roots[0], xe),
                field::mul(quartic_roots[1], xe),
                field::mul(quartic_roots[2], xe),
                field::mul(quartic_roots[3], xe)
            ]);
        }
        let ys = poly_proof.values();

        // interpolate x and y values into row polynomials
        let row_polys = quartic::interpolate_batch(&xs, ys);

        // calculate the pseudo-random x coordinate
        let special_x = field::prng(to_bytes(&p_root));

        // check that when the polynomials are evaluated at x, the result is equal to the corresponding column value
        let p_evaluations = quartic::evaluate_batch(&row_polys, special_x);
        let column_values = get_column_values(&column_proof, &positions, &augmented_positions, column_length);
        if p_evaluations != column_values {
            return Err(format!("Row polynomial didn't evaluate to column value at depth {}", depth));
        }

        // update variables for the next iteration of the loop
        p_root = *column_root;
        root = field::exp(root, 4);
        max_degree_plus_1 = max_degree_plus_1 / 4;
        column_length = column_length / 4;
    }

    // 3 ----- verify the remainder of the FRI proof ----------------------------------------------
    // check that Merkle root matches up
    let c_tree = MerkleTree::new(quartic::transpose(&proof.remainder, 1), options.hash_function());
    if *c_tree.root() != p_root {
        return Err(String::from("Remainder values do not match Merkle root of the last column"));
    }

    // make sure the remainder values satisfy the degree
    return verify_remainder(&proof.remainder, max_degree_plus_1, root, options.extension_factor());
}

fn verify_remainder(remainder: &[u64], max_degree_plus_1: usize, root: u64, extension_factor: usize) -> Result<bool, String> {

    if max_degree_plus_1 > remainder.len() {
        return Err(String::from("Remainder degree is greater than number of remainder values"));
    }

    // exclude points which should be skipped during evaluation
    let mut positions = Vec::new();
    for i in 0..remainder.len() {
        if i % extension_factor != 0 {
            positions.push(i);
        }
    }

    // pick a subset of points from the remainder and interpolate them into a polynomial
    let domain = field::get_power_series(root, remainder.len());
    let mut xs = Vec::with_capacity(max_degree_plus_1);
    let mut ys = Vec::with_capacity(max_degree_plus_1);
    for i in 0..max_degree_plus_1 {
        let p = positions[i];
        xs.push(domain[p]);
        ys.push(remainder[p]);
    }
    let poly = polys::interpolate(&xs, &ys);

    // check that polynomial evaluates correctly for all other points in the remainder
    for i in max_degree_plus_1..positions.len() {
        let p = positions[i];
        if polys::eval(&poly, domain[p]) != remainder[p] {
            return Err(format!("Remainder is not a valid degree {} polynomial", max_degree_plus_1 - 1));
        }
    }

    return Ok(true);
}

// HELPER FUNCTIONS
// ================================================================================================
fn get_augmented_positions(positions: &[usize], column_length: usize) -> Vec<usize> {
    let row_length = column_length / 4;
    let mut result = Vec::new();
    for i in 0..positions.len() {
        let ap = positions[i] % row_length;
        if !result.contains(&ap) {
            result.push(ap);
        }
    }    
    return result;
}

fn get_column_values(proof: &BatchMerkleProof, positions: &[usize], augmented_positions: &[usize], column_length: usize) -> Vec<u64> {
    let row_length = column_length / 4;

    let values = proof.values();
    let mut result = Vec::new();
    for position in positions {
        let idx = augmented_positions.iter().position(|&v| v == position % row_length).unwrap();
        let value = values[idx][position / row_length];
        result.push(value);
    }

    return result;
}

fn get_root_of_unity_degree(root: u64) -> usize {
    let mut result = 1;
    let mut root = root;
    while root != 1 {
        result = result * 2;
        root = field::mul(root, root);
    }
    return result;
}

fn to_bytes(value: &[u64; 4]) -> [u8; 32] {
    return unsafe { *(value as *const _ as *const [u8; 32]) };
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    
    use crate::{ field, polys };
    use crate::stark::{ ProofOptions, utils::QueryIndexGenerator };

    #[test]
    fn verify_remainder() {
        let degree_plus_1: usize = 32;
        let root = field::get_root_of_unity((degree_plus_1 * 2) as u64);
        let extension_factor = 16;

        let mut remainder = field::rand_vector(degree_plus_1);
        remainder.resize(degree_plus_1 * 2, 0);
        polys::eval_fft(&mut remainder, true);

        // check against exact degree
        let result = super::verify_remainder(&remainder, degree_plus_1, root, extension_factor);
        assert_eq!(Ok(true), result);

        // check against higher degree
        let result = super::verify_remainder(&remainder, degree_plus_1 + 1, root, extension_factor);
        assert_eq!(Ok(true), result);

        // check against lower degree
        let degree_plus_1 = degree_plus_1 - 1;
        let result = super::verify_remainder(&remainder, degree_plus_1, root, extension_factor);
        let err_msg = format!("Remainder is not a valid degree {} polynomial", degree_plus_1 - 1);
        assert_eq!(Err(err_msg), result);
    }

    #[test]
    fn prove_verify() {
        let degree_plus_1: usize = 64;
        let domain_size: usize = 512;
        let root = field::get_root_of_unity(domain_size as u64);
        let domain = field::get_power_series(root, domain_size);
        let options = ProofOptions::default();

        // generate proof
        let mut evaluations = field::rand_vector(degree_plus_1);
        evaluations.resize(domain_size, 0);
        polys::eval_fft(&mut evaluations, true);
        let proof = super::prove(&evaluations, &domain, degree_plus_1, &options);

        // verify proof
        let idx_generator = QueryIndexGenerator::new(&options);
        let trace_positions = idx_generator.get_trace_indexes(&proof.ev_root, domain_size);
        let sampled_evaluations = trace_positions.into_iter().map(|i| evaluations[i]).collect::<Vec<u64>>();

        let result = super::verify(&proof, &sampled_evaluations, root, degree_plus_1, &options);
        assert_eq!(Ok(true), result);
    }

    #[test]
    fn verify_fail() {
        let degree_plus_1: usize = 64;
        let domain_size: usize = 512;
        let root = field::get_root_of_unity(domain_size as u64);
        let domain = field::get_power_series(root, domain_size);
        let options = ProofOptions::default();

        // generate proof
        let mut evaluations = field::rand_vector(degree_plus_1);
        evaluations.resize(domain_size, 0);
        polys::eval_fft(&mut evaluations, true);
        let proof = super::prove(&evaluations, &domain, degree_plus_1, &options);

        // degree too low
        let idx_generator = QueryIndexGenerator::new(&options);
        let trace_positions = idx_generator.get_trace_indexes(&proof.ev_root, domain_size);
        let sampled_evaluations = trace_positions.into_iter().map(|i| evaluations[i]).collect::<Vec<u64>>();

        let result = super::verify(&proof, &sampled_evaluations, root, degree_plus_1 - 1, &options);
        let err_msg = format!("Remainder is not a valid degree {} polynomial", 14);
        assert_eq!(Err(err_msg), result);

        // invalid evaluations
        let sampled_evaluations = sampled_evaluations[1..].to_vec();
        let result = super::verify(&proof, &sampled_evaluations, root, degree_plus_1, &options);
        let err_msg = format!("Verification of evaluation values failed");
        assert_eq!(Err(err_msg), result);

        // invalid ev_root
        let mut proof2 = proof.clone();
        proof2.ev_root = [1, 2, 3, 4];
        let result = super::verify(&proof2, &sampled_evaluations, root, degree_plus_1, &options);
        let err_msg = format!("Verification of evaluation Merkle proof failed");
        assert_eq!(Err(err_msg), result);

        // TODO: add more tests
    }
}