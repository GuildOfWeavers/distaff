use std::time::{ Instant };

use crate::stark::{ TraceTable, ConstraintTable };


pub fn prove(program: &[u64], extension_factor: usize) {

    // 1 ----- execute the program to generate its execution trace
    let now = Instant::now();
    let mut trace = TraceTable::new(&program, extension_factor);
    let t = now.elapsed().as_millis();
    println!("Generated trace table of {} steps in {} ms", trace.len(), t);

    // 2 ----- interpolate trace into polynomials
    let now = Instant::now();
    trace.interpolate();
    let t = now.elapsed().as_millis();
    println!("Interpolated trace of {} registers in {} ms", trace.register_count(), t);

    // 3 ----- evaluate transition constraints
    let now = Instant::now();
    let evaluations = ConstraintTable::new(&trace);
    let t = now.elapsed().as_millis();
    println!("Evaluated {} constraints in {} ms", evaluations.constraint_count(), t);

    // 4 ----- extend execution trace over evaluation domain
    let now = Instant::now();
    trace.extend();
    let t = now.elapsed().as_millis();
    println!("Extended trace of {} registers in {} ms", trace.register_count(), t);

    // 5 ----- build merkle tree of extended execution trace
    // TODO

    // 6 ----- compute composition polynomial
    // TODO

    // 7 ----- generate low-degree proof for composition polynomial
    // TODO

    // 8 ----- query extended execution trace at pseudo-random positions
    // TODO
}