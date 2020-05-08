use crate::stark::{ ProofOptions };

pub fn find_pow_nonce(seed: [u64; 4], options: &ProofOptions) -> ([u64; 4], u64) {

    let hash = options.hash_function();
    let grinding_factor = options.grinding_factor();

    let mut inputs = [seed[0], seed[1], seed[2], seed[3], 0, 0, 0, 0];
    let mut output = [0u64; 4];

    loop {
        inputs[7] += 1;
        hash(&inputs, &mut output);
        if output[0].trailing_zeros() >= grinding_factor { break; }
    }

    return (output, inputs[7]);
}

pub fn verify_pow_nonce(seed: [u64; 4], nonce: u64, options: &ProofOptions) -> Result<[u64; 4], String> {

    let hash = options.hash_function();

    let inputs = [seed[0], seed[1], seed[2], seed[3], 0, 0, 0, nonce];
    let mut output = [0u64; 4];

    hash(&inputs, &mut output);
    if output[0].trailing_zeros() < options.grinding_factor() {
        return Err(String::from("seed proof-of-work verification failed"));
    }

    return Ok(output);
}