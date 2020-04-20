use std::time::{ Instant };
use std::collections::BTreeMap;
use crate::math::{ field, quartic::to_quartic_vec };
use crate::crypto::{ MerkleTree };
use crate::utils::uninit_vector;

use super::trace::{ TraceTable, TraceState };
use super::constraints::{ ConstraintEvaluator, ConstraintTable, MAX_CONSTRAINT_DEGREE };
use super::{ ProofOptions, StarkProof, fri, utils::QueryIndexGenerator };

// PROVER FUNCTION
// ================================================================================================

pub fn prove(trace: &mut TraceTable, inputs: &[u64], outputs: &[u64], options: &ProofOptions) -> StarkProof {

    // 1 ----- extend execution trace -------------------------------------------------------------
    let now = Instant::now();
    trace.extend();
    let t = now.elapsed().as_millis();
    println!("Extended execution trace of {} registers to {} steps in {} ms", trace.register_count(), trace.len(), t);

    // 2 ----- build Merkle tree from extended execution trace ------------------------------------
    let now = Instant::now();
    let mut trace_state = TraceState::new(trace.max_stack_depth());
    let mut hashed_states = to_quartic_vec(uninit_vector(trace.len() * 4));
    for i in 0..trace.len() {
        // TODO: this loop should be parallelized
        trace.fill_state(&mut trace_state, i);
        options.hash_function()(trace_state.registers(), &mut hashed_states[i]);
    }
    let trace_tree = MerkleTree::new(hashed_states, options.hash_function());
    let t = now.elapsed().as_millis();
    println!("Built trace Merkle tree in {} ms", t);

    // 3 ----- build evaluation domain for the extended execution trace ---------------------------
    let now = Instant::now();
    let root = field::get_root_of_unity(trace.len() as u64);
    let domain = field::get_power_series(root, trace.len());
    let t = now.elapsed().as_millis();
    println!("Built evaluation domain of {} elements in {} ms", domain.len() , t);

    // 4 ----- evaluate constraints ---------------------------------------------------------------
    let now = Instant::now();
    
    // initialize constraint evaluator and allocate space to hold constraint evaluations
    let constraint_evaluator = ConstraintEvaluator::new(
        trace_tree.root(), 
        trace.len() / trace.extension_factor(),
        trace.max_stack_depth(),
        inputs,
        outputs
    );
    let mut constraints = ConstraintTable::new(constraint_evaluator, trace.extension_factor());
    
    // allocate space to hold current and next states for constraint evaluations
    let mut current = TraceState::new(trace.max_stack_depth());
    let mut next = TraceState::new(trace.max_stack_depth());

    // we don't need to evaluate constraints over the entire extended execution trace; we need
    // to evaluate them over the domain extended to match max constraint degree - thus, we can
    // skip most trace states for the purposes of constraint evaluation.
    let stride = trace.extension_factor() / MAX_CONSTRAINT_DEGREE;
    for i in (0..trace.len()).step_by(stride) {
        // TODO: this loop should be parallelized and also potentially optimized to avoid copying
        // next state from the trace table twice

        // copy current and next states from the trace table; next state may wrap around the
        // execution trace (close to the end of the trace)
        trace.fill_state(&mut current, i);
        trace.fill_state(&mut next, (i + trace.extension_factor()) % trace.len());

        // evaluate the constraints
        constraints.evaluate(&current, &next, domain[i], i / stride);
    }

    let t = now.elapsed().as_millis();
    println!("Evaluated {} constraints in {} ms", constraints.constraint_count(), t);

    // 5 ----- combine transition constraints into a single polynomial ----------------------------
    let now = Instant::now();

    // compute composition polynomial evaluations
    let composed_evaluations = constraints.compose(&domain);
    let t = now.elapsed().as_millis();
    println!("Computed composition polynomial in {} ms", t);

    // 6 ----- generate low-degree proof for composition polynomial -------------------------------
    let now = Instant::now();
    let composition_degree_plus_1 = constraints.composition_degree() + 1;
    let fri_proof = fri::prove(&composed_evaluations, &domain, composition_degree_plus_1, options);
    let t = now.elapsed().as_millis();
    println!("Generated low-degree proof for composition polynomial in {} ms", t);

    // 7 ----- query extended execution trace at pseudo-random positions --------------------------
    let now = Instant::now();

    // generate pseudo-random indexes based on the root of the composition Merkle tree
    let idx_generator = QueryIndexGenerator::new(options);
    let positions = idx_generator.get_trace_indexes(&fri_proof.ev_root, trace.len());

    // for each queried step, collect the current and the next states of the execution trace;
    // this way, the verifier will be able to get two consecutive states for each query.
    let mut trace_states = BTreeMap::new();
    for &position in positions.iter() {
        let next_position = (position + options.extension_factor()) % trace.len();

        trace_states.insert(position, trace.get_state(position));
        trace_states.insert(next_position, trace.get_state(next_position));
    }

    // sort the positions and corresponding states so that their orders align
    let augmented_positions = trace_states.keys().cloned().collect::<Vec<usize>>();
    let trace_states = trace_states.into_iter().map(|(_, v)| v).collect();

    // generate Merkle proof for the augmented positions
    let trace_proof = trace_tree.prove_batch(&augmented_positions);

    // build the proof object
    let proof = StarkProof::new(trace_tree.root(), trace_proof, trace_states, fri_proof, options.clone());

    let t = now.elapsed().as_millis();
    println!("Computed {} trace queries and built proof object in {} ms", positions.len(), t);

    return proof;
}