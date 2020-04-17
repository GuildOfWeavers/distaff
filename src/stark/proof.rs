use serde::{ Serialize, Deserialize };
use crate::math::quartic::to_quartic_vec;
use crate::crypto::{ BatchMerkleProof };
use crate::stark::{ fri::FriProof, TraceState, ProofOptions };
use crate::utils::uninit_vector;

// TYPES AND INTERFACES
// ================================================================================================

// TODO: custom serialization should reduce size by 5% - 10%
#[derive(Clone, Serialize, Deserialize)]
pub struct StarkProof {
    trace_root  : [u64; 4],
    domain_depth: u8,
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
            domain_depth: trace_proof.depth,
            trace_nodes : trace_proof.nodes,
            trace_states: trace_states.into_iter().map(|s| s.state).collect(),
            ld_proof    : ld_proof,
            options     : options
        };
    }

    pub fn trace_root(&self) -> &[u64; 4] {
        return &self.trace_root;
    }

    pub fn options(&self) -> &ProofOptions {
        return &self.options;
    }

    pub fn domain_size(&self) -> usize {
        return usize::pow(2, self.domain_depth as u32);
    }

    pub fn trace_proof(&self) -> BatchMerkleProof {

        let mut hashed_states = to_quartic_vec(uninit_vector(self.trace_states.len() * 4));
        for i in 0..self.trace_states.len() {
            self.options.hash_function()(&self.trace_states[i], &mut hashed_states[i]);
        }

        return BatchMerkleProof {
            nodes   : self.trace_nodes.clone(),
            values  : hashed_states,
            depth   : self.domain_depth,
         };
    }

    pub fn ld_proof(&self) -> &FriProof {
        return &self.ld_proof;
    }

    pub fn trace_states(&self) -> Vec<TraceState> {
        let mut result = Vec::new();
        for raw_state in self.trace_states.iter() {
            result.push(TraceState::from_raw_state(raw_state.clone()));
        }
        return result;
    }
}