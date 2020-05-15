use serde::{ Serialize, Deserialize };
use crate::math::{ quartic::to_quartic_vec};
use crate::crypto::{ BatchMerkleProof };
use crate::stark::{ fri::FriProof, TraceState, ProofOptions };
use crate::utils::{ uninit_vector };

// TYPES AND INTERFACES
// ================================================================================================

// TODO: custom serialization should reduce size by 5% - 10%
#[derive(Clone, Serialize, Deserialize)]
pub struct StarkProof {
    trace_root          : [u64; 4],
    domain_depth        : u8,
    trace_nodes         : Vec<Vec<[u64; 4]>>,
    trace_evaluations   : Vec<Vec<u64>>,
    constraint_root     : [u64; 4],
    constraint_proof    : BatchMerkleProof,
    deep_values         : DeepValues,
    degree_proof        : FriProof,
    pow_nonce           : u64,
    options             : ProofOptions
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeepValues {
    pub trace_at_z1     : Vec<u64>,
    pub trace_at_z2     : Vec<u64>,
}

// STARK PROOF IMPLEMENTATION
// ================================================================================================
impl StarkProof {

    pub fn new(
        trace_root          : &[u64; 4],
        trace_proof         : BatchMerkleProof, 
        trace_evaluations   : Vec<Vec<u64>>,
        constraint_root     : &[u64; 4],
        constraint_proof    : BatchMerkleProof,
        deep_values         : DeepValues,
        degree_proof        : FriProof,
        pow_nonce           : u64,
        options             : &ProofOptions ) -> StarkProof
    {
        return StarkProof {
            trace_root          : *trace_root,
            domain_depth        : trace_proof.depth,
            trace_nodes         : trace_proof.nodes,
            trace_evaluations   : trace_evaluations,
            constraint_root     : *constraint_root,
            constraint_proof    : constraint_proof,
            deep_values         : deep_values,
            degree_proof        : degree_proof,
            pow_nonce           : pow_nonce,
            options             : options.clone()
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

        let mut hashed_states = to_quartic_vec(uninit_vector(self.trace_evaluations.len() * 4));
        for i in 0..self.trace_evaluations.len() {
            self.options.hash_function()(&self.trace_evaluations[i], &mut hashed_states[i]);
        }

        return BatchMerkleProof {
            nodes   : self.trace_nodes.clone(),
            values  : hashed_states,
            depth   : self.domain_depth,
         };
    }

    pub fn constraint_root(&self) -> &[u64; 4] {
        return &self.constraint_root;
    }

    pub fn constraint_proof(&self) -> BatchMerkleProof {
        return self.constraint_proof.clone();
    }

    pub fn degree_proof(&self) -> &FriProof {
        return &self.degree_proof;
    }

    pub fn trace_evaluations(&self) -> &[Vec<u64>] {
        return &self.trace_evaluations;
    }

    pub fn trace_length(&self) -> usize {
        return self.domain_size() / self.options.extension_factor();
    }

    pub fn stack_depth(&self) -> usize {
        return TraceState::compute_stack_depth(self.trace_evaluations[0].len());
    }

    pub fn pow_nonce(&self) -> u64 {
        return self.pow_nonce;
    }

    // DEEP VALUES
    // -------------------------------------------------------------------------------------------
    pub fn get_state_at_z1(&self) -> TraceState {
        return TraceState::from_raw_state(self.deep_values.trace_at_z1.clone());
    }

    pub fn get_state_at_z2(&self) -> TraceState {
        return TraceState::from_raw_state(self.deep_values.trace_at_z2.clone());
    }
}