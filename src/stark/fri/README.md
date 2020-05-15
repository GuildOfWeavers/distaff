# FRI protocol
[FRI protocol](https://eccc.weizmann.ac.il/report/2017/134/) allows us to prove and efficiently verify that a sequence of values is on the same degree < *d* polynomial.

Within Distaff VM we use a radix-4 implementation of FRI. This means that at every reduction step, polynomial degree and evaluation domain are reduced by a factor of 4. This implementation was originally adapted from Vitalik Buterin's [implementation of FRI](https://github.com/ethereum/research/tree/master/mimc_stark).

Sections below describe how FRI proofs are generated and verified.

## Proving low degree
The process of generating a low-degree proof takes as inputs evaluations of a polynomial *P(x)* over domain *D* and consists of the following steps:

1. First, we reduce the polynomial evaluations into FRI layers and commit to them.
2. Then, we query FRI layers to build FRI proof.

### Building FRI layers
To reduce polynomial evaluations to FRI layers we invoke `reduce()` function in the [prover](prover.rs) module. This function does the following:

1. *P(x)* evaluations are transposed into a matrix with 4 columns. The number of rows in these matrixes is *n/4*, where *n* is the size of the original domain. This basically re-interprets *P(x)* evaluations as evaluations of *Q(x, y)* such that *P(x) = Q(x, x<sup>4</sup>)*.
2. A Merkle tree is built from the rows of the evaluation matrix.
3. Each row in the evaluation matrix is interpreted as evaluations of degree 3 polynomial against the corresponding values in the domain. These polynomials are interpolated and we get *n/4* polynomials of degree 3.
4. A pseudo-random value is generated using the root of the Merkle tree we built in step 2 above as a seed.
5. All degree 3 polynomials are evaluated at this pseudo-random point and we get *n/4* new evaluations. These evaluations become inputs for generating the next FRI layer.

The above process is repeated until the evaluation domain reaches 256. The output of this process is a set of Merkle trees - one Merkle tree per layer. The leaves in these trees contain transposed polynomial evaluations from the preceding layer.

### Building FRI proof
To build FRI proof we invoke `build_proof()` function in the [prover](prover.rs) module. In addition to FRI layers built in the previous step, the function takes a list of query positions as inputs, and does the following:

For every FRI layer except for the last one:
1. Map query positions to the corresponding positions at this FRI layer.
2. Save layer Merkle tree root and authentication paths to the augmented query positions into the proof.

For the last layer, save all of the evaluations (up to 256) into the proof.

## Verifying low degree
To verify a low-degree proof we invoke `verify()` function in the [verifier](verifier.rs) module. The function takes FRI proof, a list of sampled polynomial evaluations and their corresponding positions in the evaluation domain, and a max degree of a polynomial implied by the evaluations.

The function rejects if the sampled evaluations are not on the same polynomial with degree <= the specified max degree.

TODO: provide detailed description.