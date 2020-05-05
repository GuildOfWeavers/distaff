use serde::{ Serialize, Deserialize };
use crate::math::{ field, quartic::to_quartic_vec};
use crate::crypto::{ BatchMerkleProof };
use crate::stark::{ fri::FriProof, TraceState, DeepValues, ProofOptions };
use crate::utils::{ uninit_vector, CopyInto };

// TYPES AND INTERFACES
// ================================================================================================

// TODO: custom serialization should reduce size by 5% - 10%
#[derive(Clone, Serialize, Deserialize)]
pub struct StarkProof {
    trace_root      : [u64; 4],
    domain_depth    : u8,
    trace_nodes     : Vec<Vec<[u64; 4]>>,
    trace_states    : Vec<Vec<u64>>,
    constraint_root : [u64; 4],
    constraint_proof: BatchMerkleProof,
    deep_values     : DeepValues,
    ld_proof        : FriProof,
    options         : ProofOptions
}

// STARK PROOF IMPLEMENTATION
// ================================================================================================
impl StarkProof {

    pub fn new(
        trace_root      : &[u64; 4],
        trace_proof     : BatchMerkleProof, 
        trace_states    : Vec<TraceState>,
        constraint_root : &[u64; 4],
        constraint_proof: BatchMerkleProof,
        deep_values     : DeepValues,
        ld_proof        : FriProof,
        options         : &ProofOptions ) -> StarkProof
    {
        return StarkProof {
            trace_root      : *trace_root,
            domain_depth    : trace_proof.depth,
            trace_nodes     : trace_proof.nodes,
            trace_states    : trace_states.into_iter().map(|s| s.registers().to_vec()).collect(),
            constraint_root : *constraint_root,
            constraint_proof: constraint_proof,
            deep_values     : deep_values,
            ld_proof        : ld_proof,
            options         : options.clone()
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

    pub fn constraint_root(&self) -> &[u64; 4] {
        return &self.constraint_root;
    }

    pub fn constraint_proof(&self) -> BatchMerkleProof {
        return self.constraint_proof.clone();
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

    pub fn trace_length(&self) -> usize {
        return self.domain_size() / self.options.extension_factor();
    }

    pub fn stack_depth(&self) -> usize {
        return TraceState::compute_stack_depth(self.trace_states[0].len());
    }

    // DEEP VALUES
    // -------------------------------------------------------------------------------------------
    pub fn get_deep_point_z(&self) -> u64 {
        return field::prng(self.constraint_root.copy_into());
    }

    pub fn get_constraint_evaluation_at_z(&self) -> u64 {
        return self.deep_values.constraints_at_z;
    }

    pub fn get_state_at_z1(&self) -> TraceState {
        return TraceState::from_raw_state(self.deep_values.trace_at_z.clone());
    }

    pub fn get_state_at_z2(&self) -> TraceState {
        return TraceState::from_raw_state(self.deep_values.trace_at_next_z.clone());
    }
}