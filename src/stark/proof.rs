use serde::{ Serialize, Deserialize };
use crate::crypto::{ BatchMerkleProof };
use crate::stark::{ fri::FriProof, TraceState, ProofOptions };
use crate::utils::{ uninit_vector, as_bytes };

// TYPES AND INTERFACES
// ================================================================================================

// TODO: custom serialization should reduce size by 5% - 10%
#[derive(Clone, Serialize, Deserialize)]
pub struct StarkProof {
    trace_root          : [u8; 32],
    trace_info          : TraceInfo,
    trace_nodes         : Vec<Vec<[u8; 32]>>,
    trace_evaluations   : Vec<Vec<u128>>,
    constraint_root     : [u8; 32],
    constraint_proof    : BatchMerkleProof,
    deep_values         : DeepValues,
    degree_proof        : FriProof,
    pow_nonce           : u64,
    options             : ProofOptions
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeepValues {
    pub trace_at_z1     : Vec<u128>,
    pub trace_at_z2     : Vec<u128>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraceInfo {
    pub domain_depth    : u8,
    pub ctx_depth       : u8,
    pub loop_depth      : u8,
    pub stack_depth     : u8,
    pub op_count        : u32,
}

// STARK PROOF IMPLEMENTATION
// ================================================================================================
impl StarkProof {
    pub fn new(
        trace_root          : &[u8; 32],
        trace_proof         : BatchMerkleProof,
        trace_evaluations   : Vec<Vec<u128>>,
        constraint_root     : &[u8; 32],
        constraint_proof    : BatchMerkleProof,
        deep_values         : DeepValues,
        degree_proof        : FriProof,
        pow_nonce           : u64,
        op_count            : u128,
        ctx_depth           : usize,
        loop_depth          : usize,
        stack_depth         : usize,
        options             : &ProofOptions ) -> StarkProof
    {
        let trace_info = TraceInfo {
            domain_depth        : trace_proof.depth,
            ctx_depth           : ctx_depth as u8,
            loop_depth          : loop_depth as u8,
            stack_depth         : stack_depth as u8,
            op_count            : op_count as u32,
        };

        return StarkProof {
            trace_root          : *trace_root,
            trace_info          : trace_info,
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

    pub fn trace_root(&self) -> &[u8; 32] {
        return &self.trace_root;
    }

    pub fn options(&self) -> &ProofOptions {
        return &self.options;
    }

    pub fn domain_size(&self) -> usize {
        return usize::pow(2, self.trace_info.domain_depth as u32);
    }

    pub fn trace_proof(&self) -> BatchMerkleProof {

        let hash = self.options.hash_fn();
        let mut hashed_states = uninit_vector::<[u8; 32]>(self.trace_evaluations.len());
        for i in 0..self.trace_evaluations.len() {
            hash(as_bytes(&self.trace_evaluations[i]), &mut hashed_states[i]);
        }

        return BatchMerkleProof {
            nodes   : self.trace_nodes.clone(),
            values  : hashed_states,
            depth   : self.trace_info.domain_depth,
         };
    }

    pub fn constraint_root(&self) -> &[u8; 32] {
        return &self.constraint_root;
    }

    pub fn constraint_proof(&self) -> BatchMerkleProof {
        return self.constraint_proof.clone();
    }

    pub fn degree_proof(&self) -> &FriProof {
        return &self.degree_proof;
    }

    pub fn trace_evaluations(&self) -> &[Vec<u128>] {
        return &self.trace_evaluations;
    }

    pub fn pow_nonce(&self) -> u64 {
        return self.pow_nonce;
    }

    // TRACE INFO
    // -------------------------------------------------------------------------------------------
    pub fn trace_length(&self) -> usize {
        return self.domain_size() / self.options.extension_factor();
    }

    pub fn ctx_depth(&self) -> usize {
        return self.trace_info.ctx_depth as usize;
    }

    pub fn loop_depth(&self) -> usize {
        return self.trace_info.loop_depth as usize;
    }

    pub fn stack_depth(&self) -> usize {
        return self.trace_info.stack_depth as usize;
    }

    pub fn op_count(&self) -> u128 {
        return self.trace_info.op_count as u128;
    }

    // DEEP VALUES
    // -------------------------------------------------------------------------------------------
    pub fn get_state_at_z1(&self) -> TraceState {
        return TraceState::from_vec(
            self.ctx_depth(),
            self.loop_depth(),
            self.stack_depth(),
            &self.deep_values.trace_at_z1);
    }

    pub fn get_state_at_z2(&self) -> TraceState {
        return TraceState::from_vec(
            self.ctx_depth(),
            self.loop_depth(),
            self.stack_depth(),
            &self.deep_values.trace_at_z2);
    }
}