mod blocks;
pub use blocks::{ ProgramBlock, Span, Group, Switch, Loop, ExecutionHint };

mod hashing;
use hashing::{ hash_op, hash_acc, hash_seq };

mod assembly;

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
        return &self.root.body();
    }

    pub fn hash(&self) -> [u128; 4] {
        let (v0, v1) = self.root.get_hash();
        return hash_acc(0, v0, v1);
    }
}

impl std::fmt::Debug for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut body_code = format!("{:?}", self.root);
        body_code.replace_range(..5, "begin");
        write!(f, "{}", body_code)
    }
}