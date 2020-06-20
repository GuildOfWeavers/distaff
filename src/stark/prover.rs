use std::{ mem, time::Instant };
use log::debug;
use crate::math::{ FiniteField, polynom, fft };
use crate::crypto::{ MerkleTree };
use crate::utils::{ Hasher, Accumulator };
use super::trace::{ TraceTable, TraceState };
use super::constraints::{ ConstraintTable, ConstraintPoly, MAX_CONSTRAINT_DEGREE };
use super::{ ProofOptions, StarkProof, CompositionCoefficients, DeepValues, fri, utils };

// PROVER FUNCTION
// ================================================================================================

pub fn prove<T>(trace: &mut TraceTable<T>, inputs: &[T], outputs: &[T], options: &ProofOptions) -> StarkProof<T>
    where T: FiniteField + Accumulator + Hasher
{
    // 1 ----- extend execution trace -------------------------------------------------------------
    let now = Instant::now();

    // build LDE domain and LDE twiddles (for FFT evaluation over LDE domain)
    let lde_root = T::get_root_of_unity(trace.domain_size());
    let lde_domain = T::get_power_series(lde_root, trace.domain_size());
    let lde_twiddles = twiddles_from_domain(&lde_domain);

    // extend the execution trace registers to LDE domain
    trace.extend(&lde_twiddles);
    debug!("Extended execution trace from {} to {} steps in {} ms",
        trace.unextended_length(),
        trace.domain_size(), 
        now.elapsed().as_millis());

    // 2 ----- build Merkle tree from the extended execution trace ------------------------------------
    let now = Instant::now();
    let trace_tree = trace.build_merkle_tree(options.hash_fn());
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

    debug!("Evaluated {} constraints over domain of {} elements in {} ms",
        constraints.constraint_count(),
        constraints.evaluation_domain_size(),
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
    let constraint_evaluations = evaluations_to_leaves(constraint_evaluations);
    let constraint_tree = MerkleTree::new(constraint_evaluations, options.hash_fn());
    debug!("Evaluated constraint polynomial and built constraint Merkle tree in {} ms",
        now.elapsed().as_millis());

    // 6 ----- build and evaluate deep composition polynomial -------------------------------------
    let now = Instant::now();

    // combine trace and constraint polynomials into the final deep composition polynomial
    let seed = constraint_tree.root();
    let (composition_poly, deep_values) = build_composition_poly(&trace, constraint_poly, seed);

    // evaluate the composition polynomial over LDE domain
    let mut composed_evaluations = composition_poly;
    debug_assert!(composed_evaluations.capacity() == lde_domain.len(), "invalid composition polynomial capacity");
    unsafe { composed_evaluations.set_len(composed_evaluations.capacity()); }
    polynom::eval_fft_twiddles(&mut composed_evaluations, &lde_twiddles, true);

    debug!("Built composition polynomial and evaluated it over domain of {} elements in {} ms",
        composed_evaluations.len(),
        now.elapsed().as_millis());

    // 7 ----- compute FRI layers for the composition polynomial ----------------------------------
    let now = Instant::now();
    let composition_degree = utils::get_composition_degree(trace.unextended_length());
    debug_assert!(composition_degree == polynom::infer_degree(&composed_evaluations));
    let (fri_trees, fri_values) = fri::reduce(&composed_evaluations, &lde_domain, options);
    debug!("Computed {} FRI layers from composition polynomial evaluations in {} ms",
    fri_trees.len(),
        now.elapsed().as_millis());

    // 8 ----- determine query positions -----------------------------------------------------------
    let now = Instant::now();

    // combine all FRI layer roots into a single vector
    let mut fri_roots: Vec<u8> = Vec::new();
    for tree in fri_trees.iter() {
        tree.root().iter().for_each(|&v| fri_roots.push(v));
    }

    // derive a seed from the combined roots
    let mut seed = [0u8; 32];
    options.hash_fn()(&fri_roots, &mut seed);

    // apply proof-of-work to get a new seed
    let (seed, pow_nonce) = utils::find_pow_nonce(seed, &options);

    // generate pseudo-random query positions
    let positions = utils::compute_query_positions(&seed, lde_domain.len(), options);
    debug!("Determined {} query positions from seed {} in {} ms",
        positions.len(),
        hex::encode(seed),
        now.elapsed().as_millis());

    // 9 ----- build proof object -----------------------------------------------------------------
    let now = Instant::now();

    // generate FRI proof
    let fri_proof = fri::build_proof(fri_trees, fri_values, &positions);

    // built a list of trace evaluations at queried positions
    let trace_evaluations = trace.get_register_values_at(&positions);

    // build a list of constraint positions
    let constraint_positions = utils::map_trace_to_constraint_positions::<T>(&positions);

    // build the proof object
    let proof = StarkProof::new(
        trace_tree.root(),
        trace_tree.prove_batch(&positions),
        trace_evaluations,
        constraint_tree.root(),
        constraint_tree.prove_batch(&constraint_positions),
        deep_values,
        fri_proof,
        pow_nonce,
        &options);

    debug!("Built proof object in {} ms", now.elapsed().as_millis());
    return proof;
}

// HELPER FUNCTIONS
// ================================================================================================
fn twiddles_from_domain<T: FiniteField>(domain: &[T]) -> Vec<T> {
    let mut twiddles = domain[..(domain.len() / 2)].to_vec();
    fft::permute(&mut twiddles);
    return twiddles;
}

fn evaluations_to_leaves<T: FiniteField>(evaluations: Vec<T>) -> Vec<[u8; 32]> {
    let element_size = mem::size_of::<T>();
    let elements_per_leaf = 32 / element_size;

    assert!(evaluations.len() % elements_per_leaf == 0,
        "number of values must be divisible by {}", elements_per_leaf);
    let mut v = std::mem::ManuallyDrop::new(evaluations);
    let p = v.as_mut_ptr();
    let len = v.len() / elements_per_leaf;
    let cap = v.capacity() / elements_per_leaf;
    return unsafe { Vec::from_raw_parts(p as *mut [u8; 32], len, cap) };
}

fn build_composition_poly<T>(trace: &TraceTable<T>, constraint_poly: ConstraintPoly<T>, seed: &[u8; 32]) -> (Vec<T>, DeepValues<T>)
    where T: FiniteField + Accumulator + Hasher
{
    // pseudo-randomly selection deep point z and coefficients for the composition
    let z = T::prng(*seed);
    let coefficients = CompositionCoefficients::new(*seed);

    // divide out deep point from trace polynomials and merge them into a single polynomial
    let (mut result, s1, s2) = trace.get_composition_poly(z, &coefficients);

    // divide out deep point from constraint polynomial and merge it into the result
    constraint_poly.merge_into(&mut result, z, &coefficients);

    return (result, DeepValues { trace_at_z1: s1, trace_at_z2: s2 });
}