use distaff::{ Program, ProgramInputs, ProofOptions };

mod utils;

pub mod comparison;
pub mod conditional;
pub mod fibonacci;
pub mod merkle;
pub mod range;

pub struct Example {
    pub program         : Program,
    pub inputs          : ProgramInputs,
    pub num_outputs     : usize,
    pub options         : ProofOptions,
    pub expected_result : Vec<u128>
}