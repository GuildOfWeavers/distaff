mod evaluator;
mod decoder;
mod stack;
mod constraint_table;
mod constraint_poly;

pub use evaluator::{ Evaluator as ConstraintEvaluator};
pub use constraint_table::{ ConstraintTable };
pub use constraint_poly::{ ConstraintPoly };

pub const MAX_TRANSITION_CONSTRAINTS: usize = 128;
pub const MAX_CONSTRAINT_DEGREE     : usize = 8;