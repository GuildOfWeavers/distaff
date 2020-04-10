use std::time::{ Instant };
use crate::math::{ quartic::to_quartic_vec };
use crate::crypto::{ MerkleTree, hash::blake3 };
use crate::stark::{ TraceTable, TraceState, ConstraintTable };
use crate::utils::uninit_vector;


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

    // 3 ----- extend execution trace over evaluation domain
    let trace_length = trace.len();
    let now = Instant::now();
    trace.extend();
    let t = now.elapsed().as_millis();
    println!("Extended trace of {} registers in {} ms", trace.register_count(), t);

    // 4 ----- evaluate transition constraints
    let now = Instant::now();
    
    let mut hashed_states = to_quartic_vec(uninit_vector(trace.len() * 4));
    let mut constraints = ConstraintTable::new(trace_length, trace.max_stack_depth());

    let mut current = TraceState::new(trace.max_stack_depth());
    let mut next = TraceState::new(trace.max_stack_depth());
    for i in 0..trace.len() {

        trace.fill_state(&mut current, i);

        blake3(&current.state, &mut hashed_states[i]);

        if i % 4 == 0 {
            trace.fill_state(&mut next, (i + 32) % trace.len()); // TODO
    
            constraints.evaluate(&current, &next, i / 4);
        }
    }
    let t = now.elapsed().as_millis();
    println!("Hashed trace states and evaluated {} constraints in {} ms", constraints.constraint_count(), t);

    // 5 ----- build merkle tree of extended execution trace
    let now = Instant::now();
    let trace_tree = MerkleTree::new(hashed_states, blake3);
    let t = now.elapsed().as_millis();
    println!("Built trace merkle tree in {} ms", t);

    println!("{:?}", trace_tree.root());
    /*
    for i in (0..evaluations.stack[0].len()).step_by(8) {
        for j in 0..evaluations.decoder.len() {
            print!("{}\t", evaluations.decoder[j][i]);
        }
        print!("| ");
        for j in 0..evaluations.stack.len() {
            print!("{}\t", evaluations.stack[j][i]);
        }
        print!("\n");
    }
    */

    // 6 ----- compute composition polynomial
    // TODO

    // 7 ----- generate low-degree proof for composition polynomial
    // TODO

    // 8 ----- query extended execution trace at pseudo-random positions
    // TODO
}