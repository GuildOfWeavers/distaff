use distaff::{ math::F128, ProgramInputs, ProofOptions };

mod utils;

pub mod fibonacci;

pub struct Example {
    pub program         : Vec<F128>,
    pub inputs          : ProgramInputs<F128>,
    pub num_outputs     : usize,
    pub options         : ProofOptions,
    pub expected_result : Vec<F128>
}