use crate::crypto::{ HashFunction, build_merkle_nodes };
use crate::{ MIN_TRACE_LENGTH, ACC_STATE_WIDTH, ACC_STATE_RATE };
use crate::utils::{ accumulator, as_bytes };
use super::{ opcodes::f128 as opcodes};

mod graph;
pub use graph::{ ExecutionGraph, ExecutionHint };

mod inputs;
pub use inputs::{ ProgramInputs };

pub mod assembly;

#[cfg(test)]
pub mod program2;

// TYPES AND INTERFACES
// ================================================================================================
type HashState = [u128; ACC_STATE_WIDTH];

pub struct Program {
    op_graph    : ExecutionGraph,
    tree_nodes  : Vec<[u8; 32]>,
    path_hashes : Vec<[u8; 32]>,
}

// PROGRAM IMPLEMENTATION
// ================================================================================================
impl Program {

    /// Constructs a new program from the provided execution graph.
    pub fn new(graph: ExecutionGraph, hash_fn: HashFunction) -> Program {

        let first_op = graph.operations()[0];
        assert!(first_op == opcodes::BEGIN, "a program must start with BEGIN operation");

        // hash all possible execution paths into individual hashes hashes
        let mut path_hashes = Vec::new();
        digest_graph(&graph, &mut path_hashes, [0; ACC_STATE_WIDTH], 0);
        
        let mut program = Program {
            op_graph    : graph,
            tree_nodes  : Vec::new(),
            path_hashes : path_hashes,
        };

        // if there is more than 1 path, build a Merkle tree out of path hashes
        if program.path_hashes.len() > 1 {
            // make sure number of hashes is a power of 2
            if !program.path_hashes.len().is_power_of_two() {
                program.path_hashes.resize(program.path_hashes.len().next_power_of_two(), [0; 32]);
            }

            program.tree_nodes = build_merkle_nodes(&program.path_hashes, hash_fn);
        }

        return program;
    }

    /// Constructs a new program from a linear execution path.
    pub fn from_path(execution_path: Vec<u128>) -> Program {

        let first_op = execution_path[0];
        assert!(first_op == opcodes::BEGIN, "a program must start with BEGIN operation");

        let graph = ExecutionGraph::new(execution_path);
        let mut path_hashes = Vec::new();
        digest_graph(&graph, &mut path_hashes, [0; ACC_STATE_WIDTH], 0);
        
        return Program {
            op_graph    : graph,
            tree_nodes  : Vec::new(),
            path_hashes : path_hashes,
        };
    }

    /// Returns hash of the program.
    pub fn hash(&self) -> &[u8; 32] {
        if self.path_hashes.len() == 1 {
            return &self.path_hashes[0];
        }
        return &self.tree_nodes[1];
    }

    /// Returns execution graph underlying the program.
    pub fn execution_graph(&self) -> &ExecutionGraph {
        return &self.op_graph;
    }

    /// Computes a Merkle authentication path from the execution path specified by `path_hash`,
    /// and returns this authentication path together with the index of the execution path
    /// in the program's execution tree.
    pub fn get_auth_path(&self, path_hash: &[u128; ACC_STATE_RATE]) -> (usize, Vec<[u8; 32]>) {

        // convert path hash into byte form
        let mut ph = [0u8; 32];
        ph.copy_from_slice(&as_bytes(path_hash));

        // find path hash in the program
        // TODO: switch to binary search
        let index = match self.path_hashes.iter().position(|&x| x == ph) {
            Some(i) => i,
            None => panic!("execution path with hash {:?} could not be found in the program", ph)
        };
        
        // make a copy of the index to save it for return value
        let ph_index = index;

        // if the program consists of a single execution path, return this hash
        if self.path_hashes.len() == 1 { return (ph_index, vec![ph]); }

        // otherwise, build a Merkle authentication path
        let mut result = Vec::new();

        result.push(self.path_hashes[index]);
        result.push(self.path_hashes[index ^ 1]);

        let mut index = (index + self.tree_nodes.len()) >> 1;
        while index > 1 {
            result.push(self.tree_nodes[index ^ 1]);
            index = index >> 1;
        }

        return (ph_index, result);
    }

    /// Verifies Merkle authentication path against the specifies `program_hash`
    pub fn verify_auth_path(program_hash: &[u8; 32], index: usize, auth_path: &[[u8; 32]], hash: HashFunction) -> bool {

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

// PUBLIC FUNCTIONS
// ================================================================================================

/// Computes the length of execution trace such that:
/// 1. The length of the trace is at least 16;
/// 2. The length of the trace is a power of 2;
/// 3. Last operation in the trace is a NOOP.
pub fn get_padded_length(length: usize, last_op: u128) -> usize {
    let new_length = if length.is_power_of_two() {
        if last_op == opcodes::NOOP {
            length
        }
        else {
            length.next_power_of_two() * 2
        }
    }
    else {
        length.next_power_of_two()
    };
    return std::cmp::max(new_length, MIN_TRACE_LENGTH);
}

// HELPER FUNCTIONS
// ================================================================================================
fn digest_graph(graph: &ExecutionGraph, hashes: &mut Vec<[u8; 32]>, mut state: HashState, mut step: usize) {

    let segment_ops = graph.operations();
    if graph.has_next() {
        // this is not the last segment of the program - so, update the state with all opcodes
        for i in 0..segment_ops.len() {
            accumulator::apply_round(&mut state, segment_ops[i], step);
            step += 1;
        }

        // the follow true and false branches of the consequent segments
        digest_graph(graph.true_branch(), hashes, state, step);
        digest_graph(graph.false_branch(), hashes, state, step);
    }
    else {
        // this is the last segment of the program - so, determine how much padding this path
        // needs, and append the appropriate number of NOOP operations at the end
        let segment_length = segment_ops.len();
        let path_length = get_padded_length(step + segment_length, segment_ops[segment_length - 1]);
        let mut segment_ops = segment_ops.to_vec();
        segment_ops.resize(path_length - step, opcodes::NOOP);

        // update the state with all opcodes but the last one
        for i in 0..(segment_ops.len() - 1) {
            accumulator::apply_round(&mut state, segment_ops[i], step);
            step += 1;
        }

        // record the final hash
        let mut path_hash = [0u8; 32];
        path_hash.copy_from_slice(&as_bytes(&state)[..32]);
        hashes.push(path_hash);
    }
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {

    use crate::{ utils::accumulator, crypto::hash::blake3 };
    use super::{ opcodes, ExecutionGraph, Program };

    #[test]
    fn new_program_single_path() {
        let mut path = vec![opcodes::BEGIN, opcodes::ADD, opcodes::MUL];
        let graph = ExecutionGraph::new(path.clone());
        let program1 = Program::new(graph, blake3);
        let program2 = Program::from_path(path.clone());

        pad_program(&mut path);
        let path_hash = accumulator::digest(&path[..(path.len() - 1)]);
        assert_eq!(path_hash, program1.path_hashes[0]);
        assert_eq!(path_hash, *program1.hash());
        assert_eq!(path_hash, program2.path_hashes[0]);
        assert_eq!(path_hash, *program2.hash());
    }

    #[test]
    fn new_program_two_paths() {
        let mut graph = ExecutionGraph::new(vec![opcodes::BEGIN, opcodes::ADD, opcodes::MUL]);
        graph.set_next(
            ExecutionGraph::new(vec![opcodes::ASSERT, opcodes::DROP]),
            ExecutionGraph::new(vec![opcodes::NOT, opcodes::ASSERT, opcodes::DUP]));

        let program = Program::new(graph, blake3);

        let mut path1 = vec![opcodes::BEGIN, opcodes::ADD, opcodes::MUL, opcodes::ASSERT, opcodes::DROP];
        pad_program(&mut path1);
        let path1_hash = accumulator::digest(&path1[..(path1.len() - 1)]);
        let mut path2 = vec![opcodes::BEGIN, opcodes::ADD, opcodes::MUL, opcodes::NOT, opcodes::ASSERT, opcodes::DUP];
        pad_program(&mut path2);
        let path2_hash = accumulator::digest(&path2[..(path2.len() - 1)]);

        let buf = [path1_hash, path2_hash].concat();
        let mut program_hash = [0u8; 32];
        blake3(&buf, &mut program_hash);

        assert_eq!(path1_hash, program.path_hashes[0]);
        assert_eq!(path2_hash, program.path_hashes[1]);
        assert_eq!(program_hash, *program.hash());
    }

    fn pad_program(program: &mut Vec<u128>) {
        let padded_length = super::get_padded_length(program.len(), *program.last().unwrap());
        program.resize(padded_length, opcodes::NOOP);
    }
}