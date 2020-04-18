use crate::math::{ polys };

pub fn uninit_vector(length: usize) -> Vec<u64> {
    let mut vector = Vec::with_capacity(length);
    unsafe { vector.set_len(length); }
    return vector;
}

pub fn zero_filled_vector(length: usize, capacity: usize) -> Vec<u64> {
    let mut vector = vec![0; capacity];
    vector.truncate(length);
    return vector;
}

/// Computes log_2(base^exponent)
pub fn pow_log2(base: u64, mut exponent: u32) -> u64 {
    let mut twos = 0;
    while exponent % 2 == 0 {
        twos += 1;
        exponent = exponent / 2;
    }
    return u64::pow(2, twos) * (64 - u64::pow(base, exponent).leading_zeros() as u64 - 1);
}

pub fn infer_evaluation_degree(evaluations: &[u64]) -> u64 {

    let mut evaluations = evaluations.to_vec();
    polys::interpolate_fft(&mut evaluations, true);
    
    for i in (0..evaluations.len()).rev() {
        if evaluations[i] != 0 {
            return i as u64;
        }
    }

    return 0;
}