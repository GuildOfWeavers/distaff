use serde::{ Serialize, Deserialize };
use crate::math::quartic::to_quartic_vec;
use crate::crypto::{ BatchMerkleProof, HashFunction };
use crate::stark::{ fri::FriProof, TraceState, ProofOptions };
use crate::utils::uninit_vector;

// TYPES AND INTERFACES
// ================================================================================================
#[derive(Clone, Serialize, Deserialize)]
pub struct StarkProof {
    trace_root  : [u64; 4],
    trace_depth : u8,
    trace_nodes : Vec<Vec<[u64; 4]>>,
    trace_states: Vec<Vec<u64>>,
    ld_proof    : FriProof,
    options     : ProofOptions
}

// STARK PROOF IMPLEMENTATION
// ================================================================================================
impl StarkProof {

    pub fn new(
        trace_root  : &[u64; 4],
        trace_proof : BatchMerkleProof, 
        trace_states: Vec<TraceState>,
        ld_proof    : FriProof,
        options     : ProofOptions ) -> StarkProof
    {
        return StarkProof {
            trace_root  : *trace_root,
            trace_depth : trace_proof.depth,
            trace_nodes : trace_proof.nodes,
            trace_states: sort_states(&trace_states, &trace_proof.values, options.hash_function()),
            ld_proof    : ld_proof,
            options     : options
        };
    }

    pub fn trace_length(&self) -> usize {
        return usize::pow(2, self.trace_depth as u32);
    }

    pub fn trace_proof(&self) -> BatchMerkleProof {

        let mut hashed_states = to_quartic_vec(uninit_vector(self.trace_states.len() * 4));
        for i in 0..self.trace_states.len() {
            self.options.hash_function()(&self.trace_states[i], &mut hashed_states[i]);
        }

        return BatchMerkleProof {
            nodes   : self.trace_nodes.clone(),
            values  : hashed_states,
            depth   : self.trace_depth,
         };
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn sort_states(trace_states: &[TraceState], proof_states: &[[u64; 4]], hash: HashFunction) -> Vec<Vec<u64>> {
    let mut hashed_states = to_quartic_vec(uninit_vector(trace_states.len() * 4));
    for i in 0..trace_states.len() {
        hash(&trace_states[i].state, &mut hashed_states[i]);
    }

    let mut sorted_states = Vec::new();
    for &state in proof_states.into_iter() {
        let idx = (&hashed_states).into_iter().position(|&v| v == state).unwrap();
        sorted_states.push(trace_states[idx].state.clone());
    }

    return sorted_states;
}