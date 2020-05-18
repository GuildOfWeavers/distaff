use std::mem;
use crate::math::{ F64, FiniteField, quartic };
use crate::crypto::{ MerkleTree };
use crate::stark::{ ProofOptions };
use crate::utils::CopyInto;

use super::{ FriProof, FriLayer, utils, MAX_REMAINDER_LENGTH};

// PROVER FUNCTIONS
// ================================================================================================

pub fn reduce(evaluations: &[u64], domain: &[u64], options: &ProofOptions) -> Vec<MerkleTree> {

    let mut result: Vec<MerkleTree> = Vec::new();

    // transpose evaluations into a matrix with 4 columns and put its rows into a Merkle tree
    let poly_values = quartic::transpose(evaluations, 1);
    let mut p_tree = MerkleTree::new(poly_values, options.hash_function());

    // reduce the degree by 4 at each iteration until the remaining polynomial is small enough
    while p_tree.leaves().len() * 4 > MAX_REMAINDER_LENGTH {

        // build polynomials from each row of the polynomial value matrix
        let depth = result.len() as u32;
        let xs = quartic::transpose(domain, usize::pow(4, depth));
        let polys = quartic::interpolate_batch(&xs, p_tree.leaves());

        // select a pseudo-random x coordinate and evaluate each row polynomial at that x
        let special_x = F64::prng(p_tree.root().copy_into());
        let column = quartic::evaluate_batch(&polys, special_x);

        // break the column in a polynomial value matrix for the next layer
        let new_poly_values = quartic::transpose(&column, 1);

        // put the resulting matrix into a Merkle tree
        let mut c_tree = MerkleTree::new(new_poly_values, options.hash_function());

        // set p_tree = c_tree for the next iteration of the loop
        mem::swap(&mut c_tree, &mut p_tree);

        // add p_tree from this loop (which is now under c_tree) to the result
        result.push(c_tree);
    }

    // add the tree at the last layer (the remainder)
    result.push(p_tree);

    return result;
}

pub fn build_proof(trees: Vec<MerkleTree>, positions: &[usize]) -> FriProof {

    let mut positions = positions.to_vec();
    let mut domain_size = trees[0].leaves().len() * 4;

    // for all trees, except the last one, record tree root and authentication
    // paths to row evaluations
    let mut layers = Vec::with_capacity(trees.len());
    for i in 0..(trees.len() - 1) {
        let tree = &trees[i];
        positions = utils::get_augmented_positions(&positions, domain_size);
        layers.push(FriLayer {
            root    : *tree.root(),
            proof   : tree.prove_batch(&positions)
        });
        domain_size = domain_size / 4;
    }

    // use the remaining polynomial values directly as proof
    let last_tree = &trees[trees.len() - 1];
    let leaves = last_tree.leaves();
    let mut remainder = vec![0; leaves.len() * 4];
    for i in 0..leaves.len() {
        remainder[i] = leaves[i][0];
        remainder[i + leaves.len()] = leaves[i][1];
        remainder[i + leaves.len() * 2] = leaves[i][2];
        remainder[i + leaves.len() * 3] = leaves[i][3];
    }

    return FriProof { layers, rem_root: *last_tree.root(), rem_values: remainder };
}