use crate::crypto::{ HashFunction, build_merkle_nodes };
use crate::{ MIN_TRACE_LENGTH };
use crate::utils::{ Accumulator };
use super::{ opcodes::f128 as opcodes};

mod execution_graph;
pub use execution_graph::{ ExecutionGraph };

mod inputs;
pub use inputs::{ ProgramInputs };

// CONSTANTS
// ================================================================================================
const STATE_ELEMENTS: usize = 4;
const STATE_WIDTH: usize = STATE_ELEMENTS * 128;

// TYPES AND INTERFACES
// ================================================================================================
type HashState = [u128; STATE_ELEMENTS];

pub struct Program {
    op_graph    : ExecutionGraph,
    tree_nodes  : Vec<[u8; 32]>,
    path_hashes : Vec<[u8; 32]>,
}

// PROGRAM IMPLEMENTATION
// ================================================================================================
impl Program {

    pub fn new(graph: ExecutionGraph, hash_fn: HashFunction) -> Program {

        let first_op = graph.operations()[0];
        assert!(first_op == opcodes::BEGIN, "a program must start with BEGIN operation");

        // hash all possible execution paths into individual hashes hashes
        let mut path_hashes = Vec::new();
        digest_graph(&graph, &mut path_hashes, [0; STATE_ELEMENTS], 0);
        
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

    pub fn from_path(execution_path: Vec<u128>) -> Program {

        let first_op = execution_path[0];
        assert!(first_op == opcodes::BEGIN, "a program must start with BEGIN operation");

        let graph = ExecutionGraph::new(execution_path);
        let mut path_hashes = Vec::new();
        digest_graph(&graph, &mut path_hashes, [0; STATE_ELEMENTS], 0);
        
        return Program {
            op_graph    : graph,
            tree_nodes  : Vec::new(),
            path_hashes : path_hashes,
        };
    }

    pub fn hash(&self) -> &[u8; 32] {
        if self.path_hashes.len() == 1 {
            return &self.path_hashes[0];
        }
        return &self.tree_nodes[1];
    }

    pub fn execution_graph(&self) -> &ExecutionGraph {
        return &self.op_graph;
    }

    pub fn get_path_hash(&self, index: usize) -> &[u8; 32] {
        return &self.path_hashes[index];
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
            u128::apply_round(&mut state, segment_ops[i], step);
            step += 1;
        }

        // the follow true and false branches of the consequent segments
        digest_graph(graph.true_path(), hashes, state, step);
        digest_graph(graph.false_path(), hashes, state, step);
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
            u128::apply_round(&mut state, segment_ops[i], step);
            step += 1;
        }

        // record the final hash
        let state_bytes: &[u8; STATE_WIDTH] = unsafe { &*(&state as *const _ as *const [u8; STATE_WIDTH]) };
        let mut path_hash = [0u8; 32];
        path_hash.copy_from_slice(&state_bytes[..32]);
        hashes.push(path_hash);
    }
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {

    use crate::{ utils::Accumulator, crypto::hash::blake3 };
    use super::{ opcodes, ExecutionGraph, Program };

    #[test]
    fn new_program_single_path() {
        let mut path = vec![opcodes::BEGIN, opcodes::ADD, opcodes::MUL];
        let graph = ExecutionGraph::new(path.clone());
        let program1 = Program::new(graph, blake3);
        let program2 = Program::from_path(path.clone());

        pad_program(&mut path);
        let path_hash = u128::digest(&path[..(path.len() - 1)]);
        assert_eq!(path_hash, *program1.get_path_hash(0));
        assert_eq!(path_hash, *program1.hash());
        assert_eq!(path_hash, *program2.get_path_hash(0));
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
        let path1_hash = u128::digest(&path1[..(path1.len() - 1)]);
        let mut path2 = vec![opcodes::BEGIN, opcodes::ADD, opcodes::MUL, opcodes::NOT, opcodes::ASSERT, opcodes::DUP];
        pad_program(&mut path2);
        let path2_hash = u128::digest(&path2[..(path2.len() - 1)]);

        let buf = [path1_hash, path2_hash].concat();
        let mut program_hash = [0u8; 32];
        blake3(&buf, &mut program_hash);

        assert_eq!(path1_hash, *program.get_path_hash(0));
        assert_eq!(path2_hash, *program.get_path_hash(1));
        assert_eq!(program_hash, *program.hash());
    }

    fn pad_program(program: &mut Vec<u128>) {
        let padded_length = super::get_padded_length(program.len(), *program.last().unwrap());
        program.resize(padded_length, opcodes::NOOP);
    }
}