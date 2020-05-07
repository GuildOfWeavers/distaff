use crate::{ math::field, crypto::{ MerkleTree, BatchMerkleProof }, utils::CopyInto };
use super::{
    TraceState, StarkProof, ConstraintEvaluator, fri, utils::compute_query_positions,
    CompositionCoefficients, MAX_CONSTRAINT_DEGREE };

// VERIFIER FUNCTION
// ================================================================================================

pub fn verify(program_hash: &[u8; 32], inputs: &[u64], outputs: &[u64], proof: &StarkProof) -> Result<bool, String> {

    let options = proof.options();
    let lde_domain_size = proof.domain_size();
    let lde_root = field::get_root_of_unity(lde_domain_size as u64);
    let degree_proof = proof.degree_proof();

    // 1 ----- Verify deep point evaluation -------------------------------------------------------
    let constraint_evaluation_at_z = evaluate_constraints(
        ConstraintEvaluator::from_proof(proof, program_hash, inputs, outputs),
        proof.get_state_at_z(),
        proof.get_state_at_next_z(),
        proof.get_deep_point_z()
    );

    if constraint_evaluation_at_z != proof.get_constraint_evaluation_at_z() {
        return Err(String::from("verification of deep point evaluation failed"));
    }

    // 2 ----- Verify proof of work and determine query positions ---------------------------------
    // TODO: verify proof of work

    let positions = compute_query_positions(&degree_proof.rem_root, lde_domain_size, options);
    //positions.sort();

    // 3 ----- Verify trace and constraint Merkle proofs ------------------------------------------
    
    // verify the trace proof
    if !MerkleTree::verify_batch(proof.trace_root(), &positions, &proof.trace_proof(), options.hash_function()) {
        return Err(String::from("verification of trace Merkle proof failed"));
    }
    // verify the constraint proof
    let mut constraint_positions = Vec::with_capacity(positions.len());
    for &position in positions.iter() {
        let cp = position / 4;
        if !constraint_positions.contains(&cp) {
            constraint_positions.push(cp);
        }
    }

    if !MerkleTree::verify_batch(proof.constraint_root(), &constraint_positions, &proof.constraint_proof(), options.hash_function()) {
        return Err(String::from("verification of constraint Merkle proof failed"));
    }

    // 4 ----- Compute composition values ---------------------------------------------------------
    
    let z = field::prng(proof.constraint_root().copy_into());
    let g = field::get_root_of_unity(proof.trace_length() as u64);
    let next_z = field::mul(z, g);
    let coefficients = CompositionCoefficients::new(proof.constraint_root());

    let t_incremental_degree = (get_composition_degree(proof.trace_length()) - (proof.trace_length() - 2)) as u64;
    let trace_at_z1 = proof.get_state_at_z().registers().to_vec();
    let trace_at_z2 = proof.get_state_at_next_z().registers().to_vec();
    let trace_states = proof.trace_states();

    let c_evaluations = get_constraint_evaluations(&proof.constraint_proof(), &positions, &constraint_positions);

    let mut evaluations = Vec::new();
    for (i, &position) in positions.iter().enumerate() {
        let x = field::exp(lde_root, position as u64);
        let registers = trace_states[i].registers();

        let mut t1_composition = 0;
        let mut t2_composition = 0;

        for (j, &value) in registers.iter().enumerate() {
            // compute T1(x) = (T(x) - T(z)) / (x - z), multiply it by a pseudo-random coefficient
            let t1 = field::div(field::sub(value, trace_at_z1[j]), field::sub(x, z));
            t1_composition = field::add(t1_composition, field::mul(t1, coefficients.trace1[j]));

            // compute T2(x) = (T(x) - T(z * g)) / (x - z * g), multiply it by a pseudo-random
            let t2 = field::div(field::sub(value, trace_at_z2[j]), field::sub(x, next_z));
            t2_composition = field::add(t2_composition, field::mul(t2, coefficients.trace2[j]));
        }

        let mut t_composition = field::add(t1_composition, t2_composition);

        let xp = field::exp(x, t_incremental_degree);
        let t2_composition_adj = field::mul(field::mul(t_composition, xp), coefficients.t2_degree);
        t_composition = field::add(field::mul(t_composition, coefficients.t1_degree), t2_composition_adj);

        // compute C(x) = (P(x) - P(z)) / (x - z)
        let mut c_composition = field::sub(c_evaluations[i], proof.get_constraint_evaluation_at_z());
        c_composition = field::div(c_composition, field::sub(x, z));
        c_composition = field::mul(c_composition, coefficients.constraints);

        evaluations.push(field::add(t_composition, c_composition));
    }
    
    // 5 ----- Verify log-degree proof -------------------------------------------------------------
    let composition_degree_plus_1 = get_composition_degree(proof.trace_length()) + 1;
    return match fri::verify(&degree_proof, &evaluations, &positions, lde_root, composition_degree_plus_1, options) {
        Ok(result) => Ok(result),
        Err(msg) => Err(format!("verification of low-degree proof failed: {}", msg))
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn evaluate_constraints(evaluator: ConstraintEvaluator, state1: TraceState, state2: TraceState, x: u64) -> u64 {

    let (i_value, f_value) = evaluator.evaluate_boundaries(&state1, x);
    let t_value = evaluator.evaluate_transition_at(&state1, &state2, x);

    // Z(x) = x - 1
    let z = field::sub(x, field::ONE);
    let mut result = field::div(i_value, z);

    // Z(x) = x - x_at_last_step
    let z = field::sub(x, evaluator.get_x_at_last_step());
    result = field::add(result, field::div(f_value, z));

    // Z(x) = (x^steps - 1) / (x - x_at_last_step)
    let z = field::div(field::sub(field::exp(x, evaluator.trace_length() as u64), field::ONE), z);
    result = field::add(result, field::div(t_value, z));

    return result;
}

// TODO: move to utils
fn get_composition_degree(trace_length: usize) -> usize {
    return (MAX_CONSTRAINT_DEGREE - 1) * trace_length - 1;
}

fn get_constraint_evaluations(proof: &BatchMerkleProof, positions: &[usize], constraint_positions: &[usize]) -> Vec<u64> {
    let values = &proof.values;
    let mut result = Vec::new();
    for &position in positions.iter() {
        let idx = constraint_positions.iter().position(|&v| v == position / 4).unwrap();
        let value = values[idx][position % 4];
        result.push(value);
    }

    return result;
}