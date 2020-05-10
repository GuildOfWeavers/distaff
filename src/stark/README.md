## Proof generation

To generate a STARK proof, we use `prove()` function from the [prover](/prover.rs) module. The function takes the following parameters:

* **trace**: `&TraceTable` - an execution trace resulting from executing a program. The trace table is built by the [processor](/../processor) module.
* **inputs**: `&[64]`
* **outputs**: `&[u64]`
* **options**: `&ProofOptions`

### 1. Extend execution trace

First, we interpolate all registers *r<sub>i</sub>* of the trace table into polynomials *T<sub>i</sub>(x)*. These polynomials are called *trace polynomials*.

Then, trace polynomials are evaluated over a larger domain to obtain the extended execution trace. This domain is the extension domain *D<sub>lde</sub>*. It is larger than the domain of the execution trace by the `extension_factor` specified by the options object.

Extended execution trace can be thought of as a 2-dimensional matrix such that each row is:

*T<sub>0</sub>(x<sub>i</sub>), T<sub>1</sub>(x<sub>i</sub>), T<sub>2</sub>(x<sub>i</sub>), ...*

where *x<sub>i</sub> = ω<sub>lde</sub>*.

### 2. Build Merkle tree from the extended execution trace


### 3. Evaluate constraints

### 4. Convert constraint evaluations into a single polynomial

### 5. Build Merkle tree from constraint polynomial evaluations

### 6. Build and evaluate deep composition polynomial

### 7. Compute FRI layers for the composition polynomial

### 8. Determine query positions

### 9. Build proof object


## Proof verification