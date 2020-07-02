use std::collections::HashMap;
use super::{ opcodes };

// TYPES AND INTERFACES
// ================================================================================================
pub struct ExecutionGraph {
    operations  : Vec<u128>,
    op_hints    : HashMap<usize, ExecutionHint>,
    t_branch    : Option<Box<ExecutionGraph>>,
    f_branch    : Option<Box<ExecutionGraph>>,
}

#[derive(Copy, Clone)]
pub enum ExecutionHint {
    EqStart,
    RcStart(u32),
    CmpStart(u32),
    None
}

// EXECUTION GRAPH IMPLEMENTATION
// ================================================================================================
impl ExecutionGraph {

    /// Constructs an edge of an execution graph from the provided sequence of operations.
    pub fn new(operations: Vec<u128>) -> ExecutionGraph {
        let last_op = operations[operations.len() - 1];
        assert!(last_op != opcodes::PUSH, "execution path cannot end with a PUSH operation");

        return ExecutionGraph {
            operations  : operations,
            op_hints    : HashMap::new(),
            t_branch    : None,
            f_branch    : None,
        };
    }

    pub fn with_hints(operations: Vec<u128>, hints: HashMap<usize, ExecutionHint>) -> ExecutionGraph {
        let last_op = operations[operations.len() - 1];
        assert!(last_op != opcodes::PUSH, "execution path cannot end with a PUSH operation");

        return ExecutionGraph {
            operations  : operations,
            op_hints    : hints,
            t_branch    : None,
            f_branch    : None,
        };
    }

    /// Attaches true and false execution branches to this edge.
    pub fn set_next(&mut self, true_branch: ExecutionGraph, false_branch: ExecutionGraph) {
        // make sure true branch starts with ASSERT operation
        assert!(true_branch.operations[0] == opcodes::ASSERT,
            "true branch of the execution graph must start with ASSERT operation");        

        // make sure false branch starts with NOT ASSERT operation
        assert!(false_branch.operations[0] == opcodes::NOT 
            && false_branch.operations[1] == opcodes::ASSERT,
            "false branch of the execution graph must start with NOT ASSERT operations");

        self.t_branch = Some(Box::new(true_branch));
        self.f_branch = Some(Box::new(false_branch));
    }

    pub fn has_next(&self) -> bool {
        return self.t_branch.is_some();
    }

    pub fn operations(&self) -> &[u128] {
        return &self.operations;
    }

    pub fn true_branch(&self) -> &ExecutionGraph {
        return self.t_branch.as_ref().unwrap();
    }

    pub fn false_branch(&self) -> &ExecutionGraph {
        return self.f_branch.as_ref().unwrap();
    }

    pub fn get_hint(&self, op_index: usize) -> ExecutionHint {
        return match self.op_hints.get(&op_index) {
            Some(&hint) => hint,
            None => ExecutionHint::None
        };
    }
}