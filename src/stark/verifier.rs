use super::{ StarkProof, fri, utils::QueryIndexGenerator };

pub fn verify(program_hash: &[u8; 32], inputs: &[u64], outputs: &[u64], proof: &StarkProof) -> Result<bool, String> {

    // 1 ----- Verify trace Merkle proof ----------------------------------------------------------
    
    // generate indexes at which the trace tree was queried
    //let idx_generator = QueryIndexGenerator::new(options);
    //let positions = idx_generator.get_trace_indexes(&proof.ld_proof.ev_root, trace.len());

    return Ok(true);
}