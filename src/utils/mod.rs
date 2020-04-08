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