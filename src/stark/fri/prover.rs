use std::mem;
use crate::math::{ F64, FiniteField, quartic };
use crate::crypto::{ MerkleTree };
use crate::stark::{ ProofOptions };
use crate::utils::{ CopyInto };

use super::{ FriProof, FriLayer, utils, MAX_REMAINDER_LENGTH};

// PROVER FUNCTIONS
// ================================================================================================

pub fn reduce(evaluations: &[u64], domain: &[u64], options: &ProofOptions) -> (Vec<MerkleTree>, Vec<Vec<[u64; 4]>>) {

    let mut tree_results: Vec<MerkleTree> = Vec::new();
    let mut value_results: Vec<Vec<[u64; 4]>> = Vec::new();

    // transpose evaluations into a matrix with 4 columns and put its rows into a Merkle tree
    let mut p_values = quartic::transpose(evaluations, 1);
    let hashed_values = utils::hash_values(&p_values, options.hash_function());
    let mut p_tree = MerkleTree::new(hashed_values, options.hash_function());

    // reduce the degree by 4 at each iteration until the remaining polynomial is small enough
    while p_tree.leaves().len() * 4 > MAX_REMAINDER_LENGTH {

        // build polynomials from each row of the polynomial value matrix
        let depth = tree_results.len() as u32;
        let xs = quartic::transpose(domain, usize::pow(4, depth));
        let polys = quartic::interpolate_batch(&xs, &p_values);

        // select a pseudo-random x coordinate and evaluate each row polynomial at that x
        let special_x = F64::prng(p_tree.root().copy_into());
        let column = quartic::evaluate_batch(&polys, special_x);

        // break the column in a polynomial value matrix for the next layer
        let mut c_values = quartic::transpose(&column, 1);

        // put the resulting matrix into a Merkle tree
        let hashed_values = utils::hash_values(&c_values, options.hash_function());
        let mut c_tree = MerkleTree::new(hashed_values, options.hash_function());

        // set p_tree = c_tree and p_values = c_values for the next iteration of the loop
        mem::swap(&mut c_tree, &mut p_tree);
        mem::swap(&mut c_values, &mut p_values);

        // add p_tree and p_values from this loop (which is now under c_tree and c_values) to the result
        tree_results.push(c_tree);
        value_results.push(c_values);
    }

    // add the tree at the last layer (the remainder)
    tree_results.push(p_tree);
    value_results.push(p_values);

    return (tree_results, value_results);
}

pub fn build_proof(trees: Vec<MerkleTree>, values: Vec<Vec<[u64; 4]>>, positions: &[usize]) -> FriProof {

    let mut positions = positions.to_vec();
    let mut domain_size = trees[0].leaves().len() * 4;

    // for all trees, except the last one, record tree root, authentication paths
    // to row evaluations, and values for row evaluations
    let mut layers = Vec::with_capacity(trees.len());
    for i in 0..(trees.len() - 1) {
        
        positions = utils::get_augmented_positions(&positions, domain_size);

        let tree = &trees[i];
        let proof = tree.prove_batch(&positions);
        
        let mut queried_values: Vec<[u64; 4]> = Vec::with_capacity(positions.len());
        for &position in positions.iter() {
            queried_values.push(values[i][position]);
        }

        layers.push(FriLayer {
            root    : *tree.root(),
            values  : queried_values,
            nodes   : proof.nodes,
            depth   : proof.depth
        });
        domain_size = domain_size / 4;
    }

    // use the remaining polynomial values directly as proof
    let last_tree = &trees[trees.len() - 1];
    let last_values = &values[values.len() - 1];
    let n = last_values.len();
    let mut remainder = vec![0; n * 4];
    for i in 0..last_values.len() {
        remainder[i] = last_values[i][0];
        remainder[i + n] = last_values[i][1];
        remainder[i + n * 2] = last_values[i][2];
        remainder[i + n * 3] = last_values[i][3];
    }

    return FriProof { layers, rem_root: *last_tree.root(), rem_values: remainder };
}