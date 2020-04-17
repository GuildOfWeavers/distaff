use std::time::{ Instant };
use crate::math::{ field, quartic::to_quartic_vec };
use crate::crypto::{ MerkleTree };
use crate::utils::uninit_vector;

use super::trace::{ TraceTable, TraceState };
use super::constraints::{ ConstraintTable, MAX_CONSTRAINT_DEGREE };
use super::{ ProofOptions, StarkProof, fri, utils::QueryIndexGenerator };

// PROVER FUNCTION
// ================================================================================================

pub fn prove(trace: &mut TraceTable, inputs: &[u64], outputs: &[u64], options: &ProofOptions) -> StarkProof {

    // 1 ----- extend execution trace -------------------------------------------------------------
    let now = Instant::now();
    trace.extend();
    let t = now.elapsed().as_millis();
    println!("Extended execution trace of {} registers to {} steps in {} ms", trace.register_count(), trace.len(), t);

    // 2 ----- evaluate transition constraints and hash extended trace ----------------------------
    let now = Instant::now();
    
    // allocate space to hold constraint evaluations and trace hashes
    let mut constraints = ConstraintTable::new(&trace);
    let mut hashed_states = to_quartic_vec(uninit_vector(trace.len() * 4));

    // allocate space to hold current and next states for constraint evaluations
    let mut current = TraceState::new(trace.max_stack_depth());
    let mut next = TraceState::new(trace.max_stack_depth());

    // we don't need to evaluate constraints over the entire extended execution trace; we need
    // to evaluate them over the domain extended to match max constraint degree - thus, we can
    // skip most trace states for the purposes of transition constraint evaluation.
    let skip = trace.extension_factor() / MAX_CONSTRAINT_DEGREE;
    for i in 0..trace.len() {
        // TODO: this loop should be parallelized and also potentially optimized to avoid copying
        // next state from the trace table twice

        // copy current state from the trace table and hash it
        trace.fill_state(&mut current, i);
        options.hash_function()(&current.state, &mut hashed_states[i]);

        if i % skip == 0 {
            // copy next trace state (wrapping around the execution trace) and evaluate constraints
            trace.fill_state(&mut next, (i + trace.extension_factor()) % trace.len());
            constraints.evaluate_transition(&current, &next, i / skip);
        }
    }

    let t = now.elapsed().as_millis();
    println!("Hashed trace states and evaluated {} transition constraints in {} ms", constraints.constraint_count(), t);

    // 3 ----- build merkle tree of extended execution trace --------------------------------------
    let now = Instant::now();
    let trace_tree = MerkleTree::new(hashed_states, options.hash_function());
    let t = now.elapsed().as_millis();
    println!("Built trace merkle tree in {} ms", t);

    // 4 ----- build evaluation domain for the extended execution trace ---------------------------
    // this domain is used later during constraint composition and low-degree proof generation
    let now = Instant::now();
    let root = field::get_root_of_unity(trace.len() as u64);
    let domain = field::get_power_series(root, trace.len());
    let t = now.elapsed().as_millis();
    println!("Built evaluation domain of {} elements in {} ms", domain.len() , t);

    // 5 ----- combine transition constraints into a single polynomial ----------------------------
    let now = Instant::now();

    // first set input/output constraints
    constraints.set_io_constraints(inputs, outputs);

    // then, compute composition polynomial
    let composition_poly = constraints.combine(trace_tree.root(), &domain);
    let t = now.elapsed().as_millis();
    println!("Computed composition polynomial in {} ms", t);

    // 6 ----- generate low-degree proof for composition polynomial -------------------------------
    let now = Instant::now();
    let composition_degree_plus_1 = constraints.composition_degree() + 1;
    let fri_proof = fri::prove(&composition_poly, &domain, composition_degree_plus_1, options);
    let t = now.elapsed().as_millis();
    println!("Generated low-degree proof for composition polynomial in {} ms", t);

    // 7 ----- query extended execution trace at pseudo-random positions --------------------------
    let now = Instant::now();

    // generate pseudo-random indexes based on the root of the composition Merkle tree
    let idx_generator = QueryIndexGenerator::new(options);
    let positions = idx_generator.get_trace_indexes(&fri_proof.ev_root, trace.len());

    // for each queried state, include the next state of the execution trace; this way
    // the verifier will be able to get two consecutive states for each query.
    let mut trace_states = Vec::new();
    let mut augmented_positions = positions.clone();
    for &position in positions.iter() {
        // save trace states at these positions to include them into the proof later
        trace_states.push(trace.get_state(position));

        let next_position = (position + options.extension_factor()) % trace.len();
        if !augmented_positions.contains(&next_position) {
            augmented_positions.push(next_position);
            trace_states.push(trace.get_state(next_position));
        }
    }

    // generate Merkle proof for the augmented positions
    let trace_proof = trace_tree.prove_batch(&augmented_positions);

    // build the proof object
    let proof = StarkProof::new(trace_tree.root(), trace_proof, trace_states, fri_proof, options.clone());

    let t = now.elapsed().as_millis();
    println!("Computed {} trace queries and built proof object in {} ms", positions.len(), t);

    return proof;
}