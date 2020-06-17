use super::{ opcodes };

// TYPES AND INTERFACES
// ================================================================================================
pub struct ExecutionGraph {
    pub operations  : Vec<u128>,
    pub true_path   : Option<Box<ExecutionGraph>>,
    pub false_path  : Option<Box<ExecutionGraph>>,
}

// EXECUTION GRAPH IMPLEMENTATION
// ================================================================================================
impl ExecutionGraph {

    pub fn new(operations: Vec<u128>) -> ExecutionGraph {
        let last_op = operations[operations.len() - 1];
        assert!(last_op != opcodes::PUSH, "execution path cannot end with a PUSH operation");

        return ExecutionGraph {
            operations  : operations,
            true_path   : None,
            false_path  : None
        };
    }

    pub fn set_next(&mut self, true_path: ExecutionGraph, false_path: ExecutionGraph) {
        // make sure true path starts with ASSERT operation
        assert!(true_path.operations[0] == opcodes::ASSERT,
            "true branch of the execution graph must start with ASSERT operation");        

        // make sure false path starts with NOT ASSERT operation
        assert!(false_path.operations[0] == opcodes::NOT 
            && false_path.operations[1] == opcodes::ASSERT,
            "false branch of the execution graph must start with NOT ASSERT operations");

        self.true_path = Some(Box::new(true_path));
        self.false_path = Some(Box::new(false_path));
    }

    pub fn has_next(&self) -> bool {
        return self.true_path.is_some();
    }

    pub fn operations(&self) -> &[u128] {
        return &self.operations;
    }

    pub fn true_path(&self) -> &ExecutionGraph {
        return self.true_path.as_ref().unwrap();
    }

    pub fn false_path(&self) -> &ExecutionGraph {
        return self.false_path.as_ref().unwrap();
    }
}