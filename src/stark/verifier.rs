use std::collections::BTreeMap;
use crate::{ math::field, crypto::MerkleTree, utils };
use super::{ StarkProof, ConstraintEvaluator, fri, utils::QueryIndexGenerator };

// VERIFIER FUNCTION
// ================================================================================================

pub fn verify(program_hash: &[u8; 32], inputs: &[u64], outputs: &[u64], proof: &StarkProof) -> Result<bool, String> {

    let options = proof.options();
    let ld_proof = proof.ld_proof();
    let domain_root = field::get_root_of_unity(proof.domain_size() as u64);

    // 1 ----- Verify trace Merkle proof ----------------------------------------------------------
    
    // generate indexes at which the trace tree was queried
    let idx_generator = QueryIndexGenerator::new(options);
    let positions = idx_generator.get_trace_indexes(ld_proof.ev_root(), proof.domain_size());
    let mut augmented_positions = positions.clone();
    for &position in positions.iter() {
        let next_position = (position + options.extension_factor()) % proof.domain_size();
        if !augmented_positions.contains(&next_position) {
            augmented_positions.push(next_position);
        }
    }
    augmented_positions.sort();

    // verify the proof
    if !MerkleTree::verify_batch(proof.trace_root(), &augmented_positions, &proof.trace_proof(), options.hash_function()) {
        return Err(String::from("verification of trace Merkle proof failed"));
    }

    // 2 ----- compute composition values for all queries -----------------------------------------
    
    // build a map of positions to trace states
    let mut trace_states = BTreeMap::new();
    for (i, state) in proof.trace_states().into_iter().enumerate() {
        trace_states.insert(augmented_positions[i], state);
    }

    // initialize constraint evaluator
    let constraint_evaluator = ConstraintEvaluator::new(
        proof.trace_root(), 
        proof.trace_length(),
        proof.stack_depth(),
        options.extension_factor(),
        &utils::quartic_from_bytes(*program_hash),
        inputs,
        outputs
    );

    let x_at_last_step = field::exp(domain_root, (proof.domain_size() - options.extension_factor()) as u64);

    let mut evaluations = Vec::new();
    for step in positions {
        let x = field::exp(domain_root, step as u64);

        let current = trace_states.get(&step).unwrap();
        let next_step = (step + options.extension_factor()) % proof.domain_size();
        let next = trace_states.get(&next_step).unwrap();

        let p_value = constraint_evaluator.combine_trace_registers(&current, x);
        let (i_value, f_value) = constraint_evaluator.evaluate_boundaries(&current, x);
        let t_value = constraint_evaluator.evaluate_transition(&current, &next, x, step);

        // Z(x) = x - 1
        let z = field::sub(x, field::ONE);
        let mut evaluation = field::add(p_value, field::div(i_value, z));

        // Z(x) = x - x_at_last_step
        let z = field::sub(x, x_at_last_step);
        evaluation = field::add(evaluation, field::div(f_value, z));

        // Z(x) = (x^steps - 1) / (x - x_at_last_step)
        let z = field::div(field::sub(field::exp(x, proof.trace_length() as u64), field::ONE), z);
        evaluation = field::add(evaluation, field::div(t_value, z));

        evaluations.push(evaluation);
    }

    // 3 ----- Verify log-degree proof -------------------------------------------------------------
    let composition_degree_plus_1 = constraint_evaluator.composition_degree() + 1;
    return match fri::verify(&ld_proof, &evaluations, domain_root, composition_degree_plus_1, options) {
        Ok(result) => Ok(result),
        Err(msg) => Err(format!("verification of low-degree proof failed: {}", msg))
    }
}