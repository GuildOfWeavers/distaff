# FRI protocol

## Proving low degree

Then, we apply radix-4 FRI to compute FRI layers for the composition polynomial evaluations. This means that at every layer we reduce the the domain size and the degree of the polynomial by a factor of 4 until the size of the domain reaches 256.

For the example we used above, FRI layers will look like so:
* Layer 0: domain size 1024, degree 111
* Layer 1: domain size 256, degree 27

The layers are constructed as follows:
1. The domain and *P(x)* evaluations are transposed into matrixes of 4 columns. The number of rows in these matrixes is *n/4*, where *n* is the size of the original domain.
2. A Merkle tree is built from the rows of the evaluation matrix.
3. Each row in the evaluation matrix is interpreted as evaluations of degree 3 polynomial against the corresponding row in the domain matrix. These polynomials are interpolated and we get *n/4* polynomials of degree 3.
4. A pseudo-random value is generated using the root of the Merkle tree we built in step 2 above as a seed.
5. Degree 3 polynomials are evaluated at this pseudo-random point and we get *n/4* new evaluations. These evaluations become inputs for generating the next FRI layer.

The output of this process is a set of Merkle trees - one Merkle tree per layer. The leaves in these trees contain transposed polynomial evaluations from the preceding layer.

## Verifying low degree

For each FRI layer:
1. We extract column values from the corresponding Merkle authentication paths, and make sure they match evaluation values from the preceding layer.
2. We adjust query positions to align with the positions at the current FRI layer.
3. Then, we verify Merkle authentication paths against adjusted positions.
4. A pseudo-random value is generated using the root of the Merkle tree we built in step 2 above as a seed.
5. Degree 3 polynomials are evaluated at this pseudo-random point and we get *n/4* new evaluations. These evaluations become inputs for generating the next FRI layer.