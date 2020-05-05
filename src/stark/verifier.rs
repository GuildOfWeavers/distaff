use crate::{ math::field, crypto::MerkleTree };
use super::{ TraceState, StarkProof, ConstraintEvaluator, fri, utils::QueryIndexGenerator };

// VERIFIER FUNCTION
// ================================================================================================

pub fn verify(program_hash: &[u8; 32], inputs: &[u64], outputs: &[u64], proof: &StarkProof) -> Result<bool, String> {

    let options = proof.options();
    let degree_proof = proof.degree_proof();
    let domain_root = field::get_root_of_unity(proof.domain_size() as u64);

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

    let idx_generator = QueryIndexGenerator::new(options);
    let mut positions = idx_generator.get_trace_indexes(degree_proof.ev_root(), proof.domain_size());
    positions.sort();

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
    
    let mut evaluations = Vec::new(); // TODO
    

    // 5 ----- Verify log-degree proof -------------------------------------------------------------
    let composition_degree_plus_1 = 16; // TODO
    return match fri::verify(&degree_proof, &evaluations, domain_root, composition_degree_plus_1, options) {
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