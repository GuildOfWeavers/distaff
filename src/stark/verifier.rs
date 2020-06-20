use std::mem;
use crate::{ math::{ FiniteField }, crypto::{ MerkleTree }, utils::{ Hasher, Accumulator}, Program };
use super::{ StarkProof, TraceState, ConstraintEvaluator, CompositionCoefficients, fri, utils };

// VERIFIER FUNCTION
// ================================================================================================

pub fn verify<T>(program_hash: &[u8; 32], inputs: &[T], outputs: &[T], proof: &StarkProof<T>) -> Result<bool, String>
    where T: FiniteField + Accumulator + Hasher
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
    let c_positions = utils::map_trace_to_constraint_positions::<T>(&t_positions);

    // 2 ----- Verify program execution path ------------------------------------------------------
    let auth_path = proof.auth_path();
    if !Program::verify_auth_path(program_hash, proof.auth_path_index(), auth_path, hash_fn) {
        return Err(String::from("verification of program execution path failed"));
    }
    let execution_path_hash = auth_path[0];

    // 3 ----- Verify trace and constraint Merkle proofs ------------------------------------------
    if !MerkleTree::verify_batch(proof.trace_root(), &t_positions, &proof.trace_proof(), hash_fn) {
        return Err(String::from("verification of trace Merkle proof failed"));
    }

    if !MerkleTree::verify_batch(proof.constraint_root(), &c_positions, &proof.constraint_proof(), hash_fn) {
        return Err(String::from("verification of constraint Merkle proof failed"));
    }

    // 4 ----- Compute constraint evaluations at DEEP point z -------------------------------------
    // derive DEEP point z from the root of the constraint tree
    let z = T::prng(*proof.constraint_root());

    // evaluate constraints at z
    let constraint_evaluation_at_z = evaluate_constraints(
        ConstraintEvaluator::from_proof(proof, &execution_path_hash, inputs, outputs),
        proof.get_state_at_z1(),
        proof.get_state_at_z2(),
        z
    );

    // 5 ----- Compute composition polynomial evaluations -----------------------------------------
    // derive coefficient for linear combination from the root of constraint tree
    let coefficients = CompositionCoefficients::<T>::new(*proof.constraint_root());

    // compute composition values separately for trace and constraints, and then add them together
    let t_composition = compose_registers(&proof, &t_positions, z, &coefficients);
    let c_composition = compose_constraints(&proof, &t_positions, &c_positions, z, constraint_evaluation_at_z, &coefficients);
    let evaluations = t_composition.iter().zip(c_composition).map(|(&t, c)| T::add(t, c)).collect::<Vec<T>>();
    
    // 6 ----- Verify low-degree proof -------------------------------------------------------------
    let max_degree = utils::get_composition_degree(proof.trace_length());
    return match fri::verify(&degree_proof, &evaluations, &t_positions, max_degree, options) {
        Ok(result) => Ok(result),
        Err(msg) => Err(format!("verification of low-degree proof failed: {}", msg))
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn evaluate_constraints<T>(evaluator: ConstraintEvaluator<T>, state1: TraceState<T>, state2: TraceState<T>, x: T) -> T
    where T: FiniteField + Accumulator + Hasher
{
    let (i_value, f_value) = evaluator.evaluate_boundaries(&state1, x);
    let t_value = evaluator.evaluate_transition_at(&state1, &state2, x);

    // Z(x) = x - 1
    let z = T::sub(x, T::ONE);
    let mut result = T::div(i_value, z);

    // Z(x) = x - x_at_last_step
    let z = T::sub(x, evaluator.get_x_at_last_step());
    result = T::add(result, T::div(f_value, z));

    // Z(x) = (x^steps - 1) / (x - x_at_last_step)
    let z = T::div(T::sub(T::exp(x, T::from_usize(evaluator.trace_length())), T::ONE), z);
    result = T::add(result, T::div(t_value, z));

    return result;
}

fn compose_registers<T>(proof: &StarkProof<T>, positions: &[usize], z: T, cc: &CompositionCoefficients<T>) -> Vec<T>
    where T: FiniteField + Accumulator
{    
    let lde_root = T::get_root_of_unity(proof.domain_size());
    let trace_root = T::get_root_of_unity(proof.trace_length());
    let next_z = T::mul(z, trace_root);

    let trace_at_z1 = proof.get_state_at_z1().registers().to_vec();
    let trace_at_z2 = proof.get_state_at_z2().registers().to_vec();
    let evaluations = proof.trace_evaluations();

    let incremental_degree = T::from_usize(utils::get_incremental_trace_degree(proof.trace_length()));

    let mut result = Vec::with_capacity(evaluations.len());
    for (registers, &position) in evaluations.into_iter().zip(positions) {
        let x = T::exp(lde_root, T::from_usize(position));
        
        let mut composition = T::ZERO;
        for (i, &value) in registers.iter().enumerate() {
            // compute T1(x) = (T(x) - T(z)) / (x - z)
            let t1 = T::div(T::sub(value, trace_at_z1[i]), T::sub(x, z));
            // multiply it by a pseudo-random coefficient, and combine with result
            composition = T::add(composition, T::mul(t1, cc.trace1[i]));

            // compute T2(x) = (T(x) - T(z * g)) / (x - z * g)
            let t2 = T::div(T::sub(value, trace_at_z2[i]), T::sub(x, next_z));
            // multiply it by a pseudo-random coefficient, and combine with result
            composition = T::add(composition, T::mul(t2, cc.trace2[i]));
        }

        // raise the degree to match composition degree
        let xp = T::exp(x, incremental_degree);
        let adj_composition = T::mul(T::mul(composition, xp), cc.t2_degree);
        composition = T::add(T::mul(composition, cc.t1_degree), adj_composition);

        result.push(composition);
    }

    return result;
}

fn compose_constraints<T>(proof: &StarkProof<T>, t_positions: &[usize], c_positions: &[usize], z: T, evaluation_at_z: T, cc: &CompositionCoefficients<T>) -> Vec<T>
    where T: FiniteField + Accumulator
{
    // build constraint evaluation values from the leaves of constraint Merkle proof
    let mut evaluations: Vec<T> = Vec::with_capacity(t_positions.len());
    let element_size = mem::size_of::<T>();
    let elements_per_leaf = 32 / element_size;
    let leaves = proof.constraint_proof().values;
    for &position in t_positions.iter() {
        let leaf_idx = c_positions.iter().position(|&v| v == position / elements_per_leaf).unwrap();
        let element_start = (position % elements_per_leaf) * element_size;
        let element_bytes = &leaves[leaf_idx][element_start..(element_start + element_size)];
        evaluations.push(T::from_bytes(element_bytes));
    }

    let lde_root = T::get_root_of_unity(proof.domain_size());

    // divide out deep point from the evaluations
    let mut result = Vec::with_capacity(evaluations.len());
    for (evaluation, &position) in evaluations.into_iter().zip(t_positions) {
        let x = T::exp(lde_root, T::from_usize(position));

        // compute C(x) = (P(x) - P(z)) / (x - z)
        let composition = T::div(T::sub(evaluation, evaluation_at_z), T::sub(x, z));
        // multiply by pseudo-random coefficient for linear combination
        result.push(T::mul(composition, cc.constraints));
    }

    return result;
}