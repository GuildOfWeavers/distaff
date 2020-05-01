mod evaluator;
mod decoder;
mod stack;
mod constraint_table;
mod constraint_polys;
mod composition_poly;
mod prng_coefficients;

pub use evaluator::{ Evaluator as ConstraintEvaluator};
pub use constraint_table::{ ConstraintTable };
pub use constraint_polys::{ ConstraintPolys };
pub use composition_poly::{ CompositionPoly };
pub use prng_coefficients::{ CompositionCoefficients };

pub const MAX_TRANSITION_CONSTRAINTS: usize = 128;
pub const MAX_CONSTRAINT_DEGREE     : usize = 8;