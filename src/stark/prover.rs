use crate::stark::{ TraceTable, ConstraintTable };


pub fn prove(program: &[u64], extension_factor: usize) {

    // 1 ----- execute the program to generate its execution trace
    let mut trace = TraceTable::new(&program, extension_factor);

    // 2 ----- interpolate trace into polynomials
    trace.interpolate();

    // 3 ----- evaluate transition constraints
    let evaluations = ConstraintTable::new(&trace);

    // 4 ----- extend execution trace over evaluation domain
    trace.extend();

    // 5 ----- build merkle tree of extended execution trace
    // TODO

    // 6 ----- compute composition polynomial
    // TODO

    // 7 ----- generate low-degree proof for composition polynomial
    // TODO

    // 8 ----- query extended execution trace at pseudo-random positions
    // TODO
}