pub fn uninit_vector(length: usize) -> Vec<u64> {
    let mut vector = Vec::with_capacity(length);
    unsafe { vector.set_len(length); }
    return vector;
}