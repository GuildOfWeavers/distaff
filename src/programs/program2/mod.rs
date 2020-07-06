mod span;
pub use span::{ Span, ExecutionHint };

mod hashing;
use hashing::{ hash_op, hash_acc, hash_seq };

mod flow;
pub use flow::{ ProgramBlock, Group, Switch, Loop };

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================
pub const BASE_CYCLE_LENGTH: usize = 16;    // TODO: move to global constants?

// TYPES AND INTERFACES
// ================================================================================================
#[derive(Clone)]
pub struct Program {
    root    : Group,
}

// PROGRAM IMPLEMENTATION
// ================================================================================================
impl Program {

    pub fn new(body: Vec<ProgramBlock>) -> Program {

        return Program {
            root    : Group::new(body)
        };
    }

    pub fn body(&self) -> &[ProgramBlock] {
        return &self.root.blocks();
    }

    pub fn hash(&self) -> [u128; 4] {
        return self.root.hash([0, 0, 0, 0]);
    }
}