use std::time::Instant;
use log::debug;
use std::collections::BTreeMap;
use crate::math::{ field, polynom, fft, quartic::to_quartic_vec };
use crate::crypto::{ MerkleTree };
use crate::utils::{ CopyInto };

use super::trace::{ TraceTable, TraceState };
use super::constraints::{ ConstraintTable, ConstraintPoly, MAX_CONSTRAINT_DEGREE };
use super::{ ProofOptions, StarkProof, fri, utils::QueryIndexGenerator, CompositionCoefficients, DeepValues };

// PROVER FUNCTION
// ================================================================================================

pub fn prove(trace: &mut TraceTable, inputs: &[u64], outputs: &[u64], options: &ProofOptions) -> StarkProof {

    // 1 ----- extend execution trace -------------------------------------------------------------
    let now = Instant::now();

    // build LDE domain and LDE twiddles (for FFT evaluation over LDE domain)
    let lde_root = field::get_root_of_unity(trace.domain_size() as u64);
    let lde_domain = field::get_power_series(lde_root, trace.domain_size());
    let mut lde_twiddles = lde_domain[..(lde_domain.len() / 2)].to_vec();
    fft::permute(&mut lde_twiddles);

    // extend the execution trace registers to LDE domain
    trace.extend(&lde_twiddles);
    debug!("Extended execution trace of {} registers to {} steps in {} ms",
        trace.register_count(),
        trace.domain_size(), 
        now.elapsed().as_millis());

    // 2 ----- build Merkle tree from extended execution trace ------------------------------------
    let now = Instant::now();
    let trace_tree = trace.build_merkle_tree(options.hash_function());
    debug!("Built trace Merkle tree in {} ms", 
        now.elapsed().as_millis());

    // 3 ----- evaluate constraints ---------------------------------------------------------------
    let now = Instant::now();
    
    // initialize constraint evaluation table
    let mut constraints = ConstraintTable::new(&trace, trace_tree.root(), inputs, outputs);
    
    // allocate space to hold current and next states for constraint evaluations
    let mut current = TraceState::new(trace.max_stack_depth());
    let mut next = TraceState::new(trace.max_stack_depth());

    // we don't need to evaluate constraints over the entire extended execution trace; we need
    // to evaluate them over the domain extended to match max constraint degree - thus, we can
    // skip most trace states for the purposes of constraint evaluation.
    let stride = trace.extension_factor() / MAX_CONSTRAINT_DEGREE;
    for i in (0..trace.domain_size()).step_by(stride) {
        // TODO: this loop should be parallelized and also potentially optimized to avoid copying
        // next state from the trace table twice

        // copy current and next states from the trace table; next state may wrap around the
        // execution trace (close to the end of the trace)
        trace.fill_state(&mut current, i);
        trace.fill_state(&mut next, (i + trace.extension_factor()) % trace.domain_size());

        // evaluate the constraints
        constraints.evaluate(&current, &next, lde_domain[i], i / stride);
    }

    debug!("Evaluated {} constraints in {} ms",
        constraints.constraint_count(),
        now.elapsed().as_millis());

    // 4 ----- convert constraint evaluations into a polynomial -----------------------------------
    let now = Instant::now();
    let constraint_poly = constraints.combine_polys();
    debug!("Converted constraint evaluations into a single polynomial of degree {} in {} ms",
        constraint_poly.degree(),
        now.elapsed().as_millis());

    // 5 ----- build Merkle tree from constraint polynomial evaluations ---------------------------
    let now = Instant::now();
    
    // evaluate constraint polynomial over the evaluation domain
    let constraint_evaluations = constraint_poly.eval(&lde_twiddles);

    // put evaluations into a Merkle tree; 4 evaluations per leaf
    let constraint_evaluations = to_quartic_vec(constraint_evaluations);
    let constraint_tree = MerkleTree::new(constraint_evaluations, options.hash_function());
    debug!("Evaluated constraint polynomial and built constraint Merkle tree in {} ms",
        now.elapsed().as_millis());

    // 6 ----- build and evaluate deep composition polynomial -------------------------------------
    let now = Instant::now();

    // combine trace and constraint polynomials into the final deep composition polynomial
    let (composition_poly, deep_values) = build_composition_poly(&trace, constraint_poly, &constraint_tree);

    // evaluate the composition polynomial over LDE domain
    let mut composed_evaluations = composition_poly;
    debug_assert!(composed_evaluations.capacity() == lde_domain.len(), "invalid composition polynomial capacity");
    unsafe { composed_evaluations.set_len(composed_evaluations.capacity()); }
    polynom::eval_fft_twiddles(&mut composed_evaluations, &lde_twiddles, true);

    debug!("Built composition polynomial and evaluated it over domain of {} elements in {} ms",
        composed_evaluations.len(),
        now.elapsed().as_millis());

    // 7 ----- generate low-degree proof for composition polynomial -------------------------------
    let now = Instant::now();
    let composition_degree = get_composition_degree(trace.unextended_length());
    debug_assert!(composition_degree == polynom::infer_degree(&composed_evaluations));
    let fri_proof = fri::prove(
        &composed_evaluations,
        &lde_domain,
        composition_degree + 1,
        options);
    debug!("Generated low-degree proof for composition polynomial in {} ms",
        now.elapsed().as_millis());

    // 8 ----- solve proof of work to determine query positions -----------------------------------

    // TODO: solve proof of work

    // generate pseudo-random query positions
    let idx_generator = QueryIndexGenerator::new(options); // TODO
    let positions = idx_generator.get_trace_indexes(&fri_proof.ev_root, trace.domain_size());    

    // 9 ----- query extended execution trace and constraint evaluations at these positions -------
    let now = Instant::now();

    // for each queried step, collect the current and the next states of the execution trace;
    // this way, the verifier will be able to get two consecutive states for each query.
    let mut trace_states = BTreeMap::new();
    for &position in positions.iter() {
        trace_states.insert(position, trace.get_state(position));
    }

    // sort the positions and corresponding states so that their orders align
    let positions = trace_states.keys().cloned().collect::<Vec<usize>>();
    let trace_states = trace_states.into_iter().map(|(_, v)| v).collect();

    // build a list of constraint positions
    let mut constraint_positions = Vec::with_capacity(positions.len());
    for &position in positions.iter() {
        let cp = position / 4;
        if !constraint_positions.contains(&cp) {
            constraint_positions.push(cp);
        }
    }

    // build the proof object
    let proof = StarkProof::new(
        trace_tree.root(),
        trace_tree.prove_batch(&positions),
        trace_states,
        constraint_tree.root(),
        constraint_tree.prove_batch(&constraint_positions),
        deep_values,
        fri_proof,
        &options);

    debug!("Computed {} trace queries and built proof object in {} ms",
        positions.len(),
        now.elapsed().as_millis());

    return proof;
}

// HELPER FUNCTIONS
// ================================================================================================

fn build_composition_poly(trace: &TraceTable, constraint_poly: ConstraintPoly, constraint_tree: &MerkleTree) -> (Vec<u64>, DeepValues) {

    // pseudo-randomly selection deep point z and coefficients for the composition
    let z = field::prng(constraint_tree.root().copy_into());
    let coefficients = CompositionCoefficients::new(constraint_tree.root());

    // divide out deep point from trace polynomials and merge them into a single polynomial
    let composition_degree = get_composition_degree(trace.unextended_length());
    let (mut result, s1, s2) = trace.get_composition_poly(z, composition_degree, &coefficients);

    // divide out deep point from constraint polynomial and merge it into the result
    let constraints_at_z = constraint_poly.merge_into(&mut result, z, &coefficients);

    return (result, DeepValues { trace_at_z: s1, trace_at_next_z: s2, constraints_at_z });
}

fn get_composition_degree(trace_length: usize) -> usize {
    return (MAX_CONSTRAINT_DEGREE - 1) * trace_length - 1;
}