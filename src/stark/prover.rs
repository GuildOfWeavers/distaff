use std::time::{ Instant };
use crate::math::{ quartic::to_quartic_vec };
use crate::crypto::{ MerkleTree, hash::blake3 };
use crate::stark::{ TraceTable, TraceState, ConstraintTable, MAX_CONSTRAINT_DEGREE };
use crate::utils::uninit_vector;

// PROVER FUNCTION
// ================================================================================================

pub fn prove(trace: &TraceTable, extension_factor: usize) {

    // 1 ----- evaluate transition constraints and hash extended trace ----------------------------
    let now = Instant::now();
    
    // allocate space to hold constraint evaluations and trace hashes
    let mut constraints = ConstraintTable::new(trace.len() / extension_factor, trace.max_stack_depth());
    let mut hashed_states = to_quartic_vec(uninit_vector(trace.len() * 4));

    // allocate space to hold current and next states for constraint evaluations
    let mut current = TraceState::new(trace.max_stack_depth());
    let mut next = TraceState::new(trace.max_stack_depth());

    // we don't need to evaluate constraints over the entire extended execution trace; we need
    // to evaluate them over the domain extended to match max constraint degree - thus, we can
    // skip most trace states for the purposes of constraint evaluation
    let skip = extension_factor / MAX_CONSTRAINT_DEGREE;
    for i in 0..trace.len() {
        // TODO: this loop should be parallelized and also potentially optimized to avoid copying
        // next state from the trace table twice

        // copy current state from the trace table and hash it
        trace.fill_state(&mut current, i);
        blake3(&current.state, &mut hashed_states[i]);

        if i % skip == 0 {
            // copy next trace state (wrapping around the execution trace) and evaluate constraints
            trace.fill_state(&mut next, (i + extension_factor) % trace.len());
            constraints.evaluate(&current, &next, i / skip);
        }
    }

    let t = now.elapsed().as_millis();
    println!("Hashed trace states and evaluated {} constraints in {} ms", constraints.constraint_count(), t);

    // 2 ----- build merkle tree of extended execution trace --------------------------------------
    let now = Instant::now();
    let trace_tree = MerkleTree::new(hashed_states, blake3);
    let t = now.elapsed().as_millis();
    println!("Built trace merkle tree in {} ms", t);

    // 3 ----- compute composition polynomial -----------------------------------------------------
    // TODO

    // 4 ----- generate low-degree proof for composition polynomial -------------------------------
    // TODO

    // 5 ----- query extended execution trace at pseudo-random positions --------------------------
    // TODO

    /*
    println!("{:?}", trace_tree.root());
    for i in (0..constraints.stack[0].len()).step_by(8) {
        for j in 0..constraints.decoder.len() {
            print!("{}\t", constraints.decoder[j][i]);
        }
        print!("| ");
        for j in 0..constraints.stack.len() {
            print!("{}\t", constraints.stack[j][i]);
        }
        print!("| ");
        for j in 0..constraints.op_acc.len() {
            print!("{}\t", constraints.op_acc[j][i]);
        }
        print!("\n");
    }
    */
}