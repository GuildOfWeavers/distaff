use crate::{
    math::field,
    crypto::{ MerkleTree },
    MIN_TRACE_LENGTH
};
use super::{ StarkProof, TraceState, ConstraintEvaluator, CompositionCoefficients, fri, utils };

// VERIFIER FUNCTION
// ================================================================================================

pub fn verify(program_hash: &[u8; 32], inputs: &[u128], outputs: &[u128], proof: &StarkProof) -> Result<bool, String>
{
    let options = proof.options();
    let hash_fn = options.hash_fn();

    // 1 ----- Verify proof of work and determine query positions ---------------------------------
    let degree_proof = proof.degree_proof();
    let mut fri_roots: Vec<u8> = Vec::new();
    for layer in degree_proof.layers.iter() {
        layer.root.iter().for_each(|&v| fri_roots.push(v));
    }
    degree_proof.rem_root.iter().for_each(|&v| fri_roots.push(v));

    let mut seed = [0u8; 32];
    hash_fn(&fri_roots, &mut seed);
    let seed = match utils::verify_pow_nonce(seed, proof.pow_nonce(), &options) {
        Ok(seed) => seed,
        Err(msg) => return Err(msg)
    };

    let t_positions = utils::compute_query_positions(&seed, proof.domain_size(), options);
    let c_positions = utils::map_trace_to_constraint_positions(&t_positions);

    // 2 ----- Verify number of operations in the program -----------------------------------------
    if proof.op_count() < MIN_TRACE_LENGTH as u128 {
        return Err(String::from("Verification of minimum operation count failed"));
    }

    // 3 ----- Verify trace and constraint Merkle proofs ------------------------------------------
    if !MerkleTree::verify_batch(proof.trace_root(), &t_positions, &proof.trace_proof(), hash_fn) {
        return Err(String::from("verification of trace Merkle proof failed"));
    }

    if !MerkleTree::verify_batch(proof.constraint_root(), &c_positions, &proof.constraint_proof(), hash_fn) {
        return Err(String::from("verification of constraint Merkle proof failed"));
    }

    // 4 ----- Compute constraint evaluations at DEEP point z -------------------------------------
    // derive DEEP point z from the root of the constraint tree
    let z = field::prng(*proof.constraint_root());

    // evaluate constraints at z
    let constraint_evaluation_at_z = evaluate_constraints(
        ConstraintEvaluator::from_proof(proof, program_hash, inputs, outputs),
        proof.get_state_at_z1(),
        proof.get_state_at_z2(),
        z
    );

    // 5 ----- Compute composition polynomial evaluations -----------------------------------------
    // derive coefficient for linear combination from the root of constraint tree
    let coefficients = CompositionCoefficients::new(*proof.constraint_root());

    // compute composition values separately for trace and constraints, and then add them together
    let t_composition = compose_registers(&proof, &t_positions, z, &coefficients);
    let c_composition = compose_constraints(&proof, &t_positions, &c_positions, z, constraint_evaluation_at_z, &coefficients);
    let evaluations = t_composition.iter().zip(c_composition).map(|(&t, c)| field::add(t, c)).collect::<Vec<u128>>();
    
    // 6 ----- Verify low-degree proof -------------------------------------------------------------
    let max_degree = utils::get_composition_degree(proof.trace_length());
    return match fri::verify(&degree_proof, &evaluations, &t_positions, max_degree, options) {
        Ok(result) => Ok(result),
        Err(msg) => Err(format!("verification of low-degree proof failed: {}", msg))
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn evaluate_constraints(evaluator: ConstraintEvaluator, state1: TraceState, state2: TraceState, x: u128) -> u128 {
    let (i_value, f_value) = evaluator.evaluate_boundaries(&state1, x);
    let t_value = evaluator.evaluate_transition_at(&state1, &state2, x);

    // Z(x) = x - 1
    let z = field::sub(x, field::ONE);
    let mut result = field::div(i_value, z);

    // Z(x) = x - x_at_last_step
    let z = field::sub(x, evaluator.get_x_at_last_step());
    result = field::add(result, field::div(f_value, z));

    // Z(x) = (x^steps - 1) / (x - x_at_last_step)
    let z = field::div(field::sub(field::exp(x, evaluator.trace_length() as u128), field::ONE), z);
    result = field::add(result, field::div(t_value, z));

    return result;
}

fn compose_registers(proof: &StarkProof, positions: &[usize], z: u128, cc: &CompositionCoefficients) -> Vec<u128>
{    
    let lde_root = field::get_root_of_unity(proof.domain_size());
    let trace_root = field::get_root_of_unity(proof.trace_length());
    let next_z = field::mul(z, trace_root);

    let trace_at_z1 = proof.get_state_at_z1().to_vec();
    let trace_at_z2 = proof.get_state_at_z2().to_vec();
    let evaluations = proof.trace_evaluations();

    let incremental_degree = utils::get_incremental_trace_degree(proof.trace_length()) as u128;

    let mut result = Vec::with_capacity(evaluations.len());
    for (registers, &position) in evaluations.into_iter().zip(positions) {
        let x = field::exp(lde_root, position as u128);
        
        let mut composition = field::ZERO;
        for (i, &value) in registers.iter().enumerate() {
            // compute T1(x) = (T(x) - T(z)) / (x - z)
            let t1 = field::div(field::sub(value, trace_at_z1[i]), field::sub(x, z));
            // multiply it by a pseudo-random coefficient, and combine with result
            composition = field::add(composition, field::mul(t1, cc.trace1[i]));

            // compute T2(x) = (T(x) - T(z * g)) / (x - z * g)
            let t2 = field::div(field::sub(value, trace_at_z2[i]), field::sub(x, next_z));
            // multiply it by a pseudo-random coefficient, and combine with result
            composition = field::add(composition, field::mul(t2, cc.trace2[i]));
        }

        // raise the degree to match composition degree
        let xp = field::exp(x, incremental_degree);
        let adj_composition = field::mul(field::mul(composition, xp), cc.t2_degree);
        composition = field::add(field::mul(composition, cc.t1_degree), adj_composition);

        result.push(composition);
    }

    return result;
}

fn compose_constraints(proof: &StarkProof, t_positions: &[usize], c_positions: &[usize], z: u128, evaluation_at_z: u128, cc: &CompositionCoefficients) -> Vec<u128> {
    // build constraint evaluation values from the leaves of constraint Merkle proof
    let mut evaluations: Vec<u128> = Vec::with_capacity(t_positions.len());
    let leaves = proof.constraint_proof().values;
    for &position in t_positions.iter() {
        let leaf_idx = c_positions.iter().position(|&v| v == position / 2).unwrap();
        let element_start = (position % 2) * 16;
        let element_bytes = &leaves[leaf_idx][element_start..(element_start + 16)];
        evaluations.push(field::from_bytes(element_bytes));
    }

    let lde_root = field::get_root_of_unity(proof.domain_size());

    // divide out deep point from the evaluations
    let mut result = Vec::with_capacity(evaluations.len());
    for (evaluation, &position) in evaluations.into_iter().zip(t_positions) {
        let x = field::exp(lde_root, position as u128);

        // compute C(x) = (P(x) - P(z)) / (x - z)
        let composition = field::div(field::sub(evaluation, evaluation_at_z), field::sub(x, z));
        // multiply by pseudo-random coefficient for linear combination
        result.push(field::mul(composition, cc.constraints));
    }

    return result;
}