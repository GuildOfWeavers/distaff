mod span;
pub use span::{ Span, ExecutionHint };

mod hashing;
use hashing::{ hash_op, hash_acc, hash_seq };

mod flow;
pub use flow::{ ProgramBlock, Group, Switch, Loop };

#[cfg(test)]
mod tests;

// TYPES AND INTERFACES
// ================================================================================================
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