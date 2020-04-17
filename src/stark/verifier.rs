use std::collections::BTreeMap;
use crate::crypto::{ MerkleTree };
use super::{ StarkProof, utils::QueryIndexGenerator, fri };

pub fn verify(program_hash: &[u8; 32], inputs: &[u64], outputs: &[u64], proof: &StarkProof) -> Result<bool, String> {

    let options = proof.options();
    let ld_proof = proof.ld_proof();

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
        return Err(String::from("Verification of trace Merkle proof failed"));
    }

    // 2 ----- compute composition values for all queries -----------------------------------------
    
    // build a map of positions to trace states
    let mut trace_states = BTreeMap::new();
    for (i, state) in proof.trace_states().into_iter().enumerate() {
        trace_states.insert(augmented_positions[i], state);
    }

    for step in positions {
        let current = trace_states.get(&step).unwrap();
        let next_step = (step + options.extension_factor()) % proof.domain_size();
        let next = trace_states.get(&next_step).unwrap();

        // TODO: evaluate constraints
        // TODO: evaluate composition polynomial
    }

    // 3 ----- Verify log-degree proof -------------------------------------------------------------
    // TODO

    return Ok(true);
}