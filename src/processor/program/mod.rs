use crate::crypto::{ HashFunction, build_merkle_nodes };
use crate::stark::{ Accumulator, MIN_TRACE_LENGTH };
use super::{ opcodes::f128 as opcodes};

mod execution_graph;
use execution_graph::ExecutionGraph;

// TYPES AND INTERFACES
// ================================================================================================
pub struct Program {
    op_graph    : ExecutionGraph,
    tree_nodes  : Vec<[u8; 32]>,
    path_hashes : Vec<[u8; 32]>,
    path_lengths: Vec<u32>,
    path_noops  : Vec<u32>,
}

struct ProgramInfo {
    path_hashes : Vec<[u8; 32]>,
    path_lengths: Vec<u32>,
    path_noops  : Vec<u32>,
}

// PROGRAM IMPLEMENTATION
// ================================================================================================
impl Program {

    pub fn new(graph: ExecutionGraph, hash_fn: HashFunction) -> Program {

        let mut info = ProgramInfo {
            path_hashes : Vec::new(),
            path_lengths: Vec::new(),
            path_noops  : Vec::new()
        };
        digest_graph(&graph, &mut info, [0; 4], 0, 0);
        
        let mut program = Program {
            op_graph    : graph,
            tree_nodes  : Vec::new(),
            path_hashes : info.path_hashes,
            path_lengths: info.path_lengths,
            path_noops  : info.path_noops
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
        let graph = ExecutionGraph::new(execution_path);
        let mut info = ProgramInfo {
            path_hashes : Vec::new(),
            path_lengths: Vec::new(),
            path_noops  : Vec::new()
        };
        digest_graph(&graph, &mut info, [0; 4], 0, 0);
        
        return Program {
            op_graph    : graph,
            tree_nodes  : Vec::new(),
            path_hashes : info.path_hashes,
            path_lengths: info.path_lengths,
            path_noops  : info.path_noops
        };
    }

    pub fn hash(&self) -> &[u8; 32] {
        if self.path_hashes.len() == 1 {
            return &self.path_hashes[0];
        }
        return &self.tree_nodes[1];
    }

    pub fn density(&self) -> f64 {
        let mut length = 0;
        let mut noops = 0;
        for i in 0..self.path_lengths.len() {
            length += self.path_lengths[i];
            noops += self.path_noops[i];
        }
        return (noops as f64) / (length as f64);
    }

    pub fn execution_graph(&self) -> &ExecutionGraph {
        return &self.op_graph;
    }

    pub fn get_path_hash(&self, index: usize) -> &[u8; 32] {
        return &self.path_hashes[index];
    }

    pub fn get_path_length(&self, index: usize) -> u32 {
        return self.path_lengths[index];
    }

    pub fn get_path_density(&self, index: usize) -> f64 {
        return (self.path_noops[index] as f64) / (self.path_lengths[index] as f64);
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn digest_graph(graph: &ExecutionGraph, info: &mut ProgramInfo, mut state: [u128; 4], mut step: usize, mut noops: u32) {

    let operations = graph.operations();
    if graph.has_next() {
        // this is not the last segment of the program - so, update the state with all opcodes
        for i in 0..operations.len() {
            u128::apply_round(&mut state, operations[i], step);
            if operations[i] == opcodes::NOOP {
                noops += 1;
            }
            step += 1;
        }

        // follow the true branch, but first pre-pend ASSERT to the execution path
        let mut t_state = state;
        u128::apply_round(&mut t_state, opcodes::ASSERT, step);
        digest_graph(graph.true_path(), info, t_state, step + 1, noops);

        // follow the false branch, but first pre-pend NOT ASSERT ot the execution path
        u128::apply_round(&mut state, opcodes::NOT, step);
        step += 1;
        u128::apply_round(&mut state, opcodes::ASSERT, step);
        digest_graph(graph.false_path(), info, state, step + 1, noops);
    }
    else {
        // this is the last segment of the program - so, determine how much padding this path
        // needs, and append the appropriate number of NOOP operations at the end
        let sprint_length = operations.len();
        let path_length = get_padded_length(step + sprint_length, operations[sprint_length - 1]);
        let mut operations = operations.to_vec();
        operations.resize(path_length - step + 1, opcodes::NOOP);

        // update the state with all opcodes but the last one
        for i in 0..(operations.len() - 1) {
            u128::apply_round(&mut state, operations[i], step);
            if operations[i] == opcodes::NOOP {
                noops += 1;
            }
            step += 1;
        }

        // record the final hash and path stats into the info object
        let state_bytes: &[u8; 512] = unsafe { &*(&state as *const _ as *const [u8; 512]) };
        let mut path_hash = [0u8; 32];
        path_hash.copy_from_slice(&state_bytes[..32]);
        info.path_hashes.push(path_hash);
        info.path_lengths.push(step as u32);
        info.path_noops.push(noops);
    }
}

fn get_padded_length(length: usize, last_op: u128) -> usize {
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

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {

    use crate::{ Accumulator, crypto::hash::blake3 };
    use super::{ opcodes, ExecutionGraph, Program, super::pad_program };

    #[test]
    fn new_program_single_path() {
        let mut path = vec![opcodes::BEGIN, opcodes::ADD, opcodes::MUL];
        let graph = ExecutionGraph::new(path.clone());
        let program1 = Program::new(graph, blake3);
        let program2 = Program::from_path(path.clone());

        pad_program(&mut path);
        let path_hash = u128::digest(&path);
        assert_eq!(path_hash, *program1.get_path_hash(0));
        assert_eq!(path_hash, *program1.hash());
        assert_eq!(path_hash, *program2.get_path_hash(0));
        assert_eq!(path_hash, *program2.hash());
    }

    #[test]
    fn new_program_two_paths() {
        let mut graph = ExecutionGraph::new(vec![opcodes::BEGIN, opcodes::ADD, opcodes::MUL]);
        graph.set_next(
            ExecutionGraph::new(vec![opcodes::DROP]),
            ExecutionGraph::new(vec![opcodes::DUP]));

        let program = Program::new(graph, blake3);

        let mut path1 = vec![opcodes::BEGIN, opcodes::ADD, opcodes::MUL, opcodes::ASSERT, opcodes::DROP];
        pad_program(&mut path1);
        let path1_hash = u128::digest(&path1);
        let mut path2 = vec![opcodes::BEGIN, opcodes::ADD, opcodes::MUL, opcodes::NOT, opcodes::ASSERT, opcodes::DUP];
        pad_program(&mut path2);
        let path2_hash = u128::digest(&path2);

        let buf = [path1_hash, path2_hash].concat();
        let mut program_hash = [0u8; 32];
        blake3(&buf, &mut program_hash);

        assert_eq!(path1_hash, *program.get_path_hash(0));
        assert_eq!(path2_hash, *program.get_path_hash(1));
        assert_eq!(program_hash, *program.hash());
    }
}