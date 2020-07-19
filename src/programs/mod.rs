use crate::crypto::{ HashFunction, build_merkle_nodes };
use crate::processor::{ OpCode, OpHint };
use crate::utils::{ as_bytes };

pub mod assembly;

pub mod blocks;
use blocks::{ ProgramBlock, Span, Group, Switch, Loop };

mod inputs;
pub use inputs::{ ProgramInputs };

mod hashing;
use hashing::{ hash_op, hash_acc, hash_seq };

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================
use crate::{
    SPONGE_WIDTH,
    PROGRAM_DIGEST_SIZE,
    SPONGE_CYCLE_LENGTH as CYCLE_LENGTH };

// TYPES AND INTERFACES
// ================================================================================================
#[derive(Clone)]
pub struct Program {
    procedures  : Vec<Group>,
    proc_hashes : Vec<[u8; 32]>,
    tree_nodes  : Vec<[u8; 32]>,
}

// PROGRAM IMPLEMENTATION
// ================================================================================================
impl Program {

    /// Constructs a new program from a list of procedures.
    pub fn new(procedures: Vec<Group>, hash_fn: HashFunction) -> Program {
        
        assert!(procedures.len() > 0, "a program must contain at least one procedure");

        let proc_hashes = hash_procedures(&procedures);
        let mut program = Program { procedures, proc_hashes, tree_nodes: Vec::new() };

        // if there is more than 1 path, build a Merkle tree out of path hashes
        if program.proc_hashes.len() > 1 {
            // make sure number of hashes is a power of 2
            if !program.proc_hashes.len().is_power_of_two() {
                program.proc_hashes.resize(program.proc_hashes.len().next_power_of_two(), [0; 32]);
            }
            program.tree_nodes = build_merkle_nodes(&program.proc_hashes, hash_fn);
        }

        return program;
    }

    /// Constructs a new program from a single procedure.
    pub fn from_proc(procedure: Vec<ProgramBlock>) -> Program {
        let procedures = vec![Group::new(procedure)];
        let proc_hashes = hash_procedures(&procedures);
        return Program { procedures, proc_hashes, tree_nodes: Vec::new() };
    }

    /// Returns a program block for the procedure at the specified `index`.
    pub fn get_proc(&self, index: usize) -> &Group {
        return &self.procedures[index];
    }

    /// Returns total number of procedures in the program.
    pub fn proc_count(&self) -> usize {
        return self.procedures.len();
    }

    /// Returns hash of the program.
    pub fn hash(&self) -> &[u8; 32] {
        if self.proc_hashes.len() == 1 {
            return &self.proc_hashes[0];
        }
        return &self.tree_nodes[1];
    }

    /// Returns a Merkle authentication path for the procedure specified by `proc_index`.
    pub fn get_proc_path(&self, proc_index: usize) -> Vec<[u8; 32]> {
        
        // if the program consists of a single procedure, return its hash
        if self.proc_hashes.len() == 1 { return vec![self.proc_hashes[proc_index]]; }

        // otherwise, build a Merkle authentication path
        let mut result = Vec::new();

        result.push(self.proc_hashes[proc_index]);
        result.push(self.proc_hashes[proc_index ^ 1]);

        let mut index = (proc_index + self.tree_nodes.len()) >> 1;
        while index > 1 {
            result.push(self.tree_nodes[index ^ 1]);
            index = index >> 1;
        }

        return result;
    }

    /// Verifies Merkle authentication path against the specifies `program_hash`
    pub fn verify_proc_path(program_hash: &[u8; 32], index: usize, auth_path: &[[u8; 32]], hash: HashFunction) -> bool {

        // if authentication path contains only one node, assume this node is program hash
        if auth_path.len() == 1 {
            return auth_path[0] == *program_hash;
        }

        // otherwise, run standard Merkle path authentication program
        let mut buf = [0u8; 64];
        let mut v = [0u8; 32];

        let r = index & 1;
        &buf[0..32].copy_from_slice(&auth_path[r]);
        &buf[32..64].copy_from_slice(&auth_path[1 - r]);
        hash(&buf, &mut v);

        let mut index = (index + usize::pow(2, (auth_path.len() - 1) as u32)) >> 1;
        for i in 2..auth_path.len() {
            if index & 1 == 0 {
                &buf[0..32].copy_from_slice(&v);
                &buf[32..64].copy_from_slice(&auth_path[i]);
            }
            else {
                &buf[0..32].copy_from_slice(&auth_path[i]);
                &buf[32..64].copy_from_slice(&v);
            }
            hash(&buf, &mut v);
            index = index >> 1;
        }

        return v == *program_hash;
    }
}

impl std::fmt::Debug for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        for i in 0..self.procedures.len() {
            let mut body_code = format!("{:?}", self.procedures[i]);
            body_code.replace_range(..5, "begin");
            if i == self.procedures.len() - 1 {
                write!(f, "{}", body_code)?;
            }
            else {
                write!(f, "{}\n", body_code)?;
            }
        }

        return Ok(());
    }
}

// HELPER FUNCTIONS
// ================================================================================================

fn hash_procedures(procedures: &Vec<Group>) -> Vec<[u8; 32]> {

    let mut hashes = Vec::with_capacity(procedures.len());

    for procedure in procedures.iter() {
        let (v0, v1) = procedure.get_hash();
        let hash = hash_acc(0, v0, v1);
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(as_bytes(&hash[..PROGRAM_DIGEST_SIZE]));
        hashes.push(hash_bytes);
    }

    return hashes;
}