use std::time::Instant;
use log::debug;
use std::collections::BTreeMap;
use crate::math::{ field };

use super::trace::{ TraceTable, TraceState };
use super::constraints::{ ConstraintEvaluator, ConstraintTable, MAX_CONSTRAINT_DEGREE };
use super::{ ProofOptions, StarkProof, fri, utils::QueryIndexGenerator };

// PROVER FUNCTION
// ================================================================================================

pub fn prove(trace: &mut TraceTable, program_hash: &[u64; 4], inputs: &[u64], outputs: &[u64], options: &ProofOptions) -> StarkProof {

    // 1 ----- extend execution trace -------------------------------------------------------------
    let now = Instant::now();
    trace.extend();
    debug!("Extended execution trace of {} registers to {} steps in {} ms",
        trace.register_count(),
        trace.domain_size(), 
        now.elapsed().as_millis());

    // 2 ----- build Merkle tree from extended execution trace ------------------------------------
    let now = Instant::now();
    let trace_tree = trace.to_merkle_tree(options.hash_function());
    debug!("Built trace Merkle tree in {} ms", 
        now.elapsed().as_millis());

    // 3 ----- build evaluation domain for the extended execution trace ---------------------------
    let now = Instant::now();
    let root = field::get_root_of_unity(trace.domain_size() as u64);
    let domain = field::get_power_series(root, trace.domain_size());
    debug!("Built evaluation domain of {} elements in {} ms",
        domain.len(),
        now.elapsed().as_millis());

    // 4 ----- evaluate constraints ---------------------------------------------------------------
    let now = Instant::now();
    
    // initialize constraint evaluator
    let constraint_evaluator = ConstraintEvaluator::new(
        trace_tree.root(), 
        trace.unextended_length(),
        trace.max_stack_depth(),
        MAX_CONSTRAINT_DEGREE,
        program_hash,
        inputs,
        outputs);

    // allocate space to hold constraint evaluations
    let mut constraints = ConstraintTable::new(constraint_evaluator, domain);
    
    // allocate space to hold current and next states for constraint evaluations
    let mut current = TraceState::new(trace.max_stack_depth());
    let mut next = TraceState::new(trace.max_stack_depth());

    // we don't need to evaluate constraints over the entire extended execution trace; we need
    // to evaluate them over the domain extended to match max constraint degree - thus, we can
    // skip most trace states for the purposes of constraint evaluation.
    for i in (0..trace.domain_size()).step_by(constraints.domain_stride()) {
        // TODO: this loop should be parallelized and also potentially optimized to avoid copying
        // next state from the trace table twice

        // copy current and next states from the trace table; next state may wrap around the
        // execution trace (close to the end of the trace)
        trace.fill_state(&mut current, i);
        trace.fill_state(&mut next, (i + trace.extension_factor()) % trace.domain_size());

        // evaluate the constraints
        constraints.evaluate(&current, &next, i);
    }

    debug!("Evaluated {} constraints in {} ms",
        constraints.constraint_count(),
        now.elapsed().as_millis());

    // 5 ----- combine transition constraints into a single polynomial ----------------------------
    let now = Instant::now();
    let composed_evaluations = constraints.compose();
    debug!("Computed composition polynomial in {} ms",
        now.elapsed().as_millis());

    // 6 ----- generate low-degree proof for composition polynomial -------------------------------
    let now = Instant::now();
    let composition_degree_plus_1 = constraints.composition_degree() + 1;
    let fri_proof = fri::prove(
        &composed_evaluations,
        constraints.domain(),
        composition_degree_plus_1,
        options);
    debug!("Generated low-degree proof for composition polynomial in {} ms",
        now.elapsed().as_millis());

    // 7 ----- query extended execution trace at pseudo-random positions --------------------------
    let now = Instant::now();

    // generate pseudo-random indexes based on the root of the composition Merkle tree
    let idx_generator = QueryIndexGenerator::new(options);
    let positions = idx_generator.get_trace_indexes(&fri_proof.ev_root, trace.domain_size());

    // for each queried step, collect the current and the next states of the execution trace;
    // this way, the verifier will be able to get two consecutive states for each query.
    let mut trace_states = BTreeMap::new();
    for &position in positions.iter() {
        let next_position = (position + options.extension_factor()) % trace.domain_size();

        trace_states.insert(position, trace.get_state(position));
        trace_states.insert(next_position, trace.get_state(next_position));
    }

    // sort the positions and corresponding states so that their orders align
    let augmented_positions = trace_states.keys().cloned().collect::<Vec<usize>>();
    let trace_states = trace_states.into_iter().map(|(_, v)| v).collect();

    // generate Merkle proof for the augmented positions
    let trace_proof = trace_tree.prove_batch(&augmented_positions);

    // build the proof object
    let proof = StarkProof::new(trace_tree.root(), trace_proof, trace_states, fri_proof, &options);

    debug!("Computed {} trace queries and built proof object in {} ms",
        positions.len(),
        now.elapsed().as_millis());

    return proof;
}