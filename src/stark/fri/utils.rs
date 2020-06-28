use crate::crypto::{ HashFunction };
use crate::utils::{ uninit_vector, as_bytes };

pub fn get_augmented_positions(positions: &[usize], column_length: usize) -> Vec<usize> {
    let row_length = column_length / 4;
    let mut result = Vec::new();
    for i in 0..positions.len() {
        let ap = positions[i] % row_length;
        if !result.contains(&ap) {
            result.push(ap);
        }
    }    
    return result;
}

pub fn hash_values(values: &Vec<[u128; 4]>, hash: HashFunction) -> Vec<[u8; 32]> {
    let mut result: Vec<[u8; 32]> = uninit_vector(values.len());
    for i in 0..values.len() {
        hash(as_bytes(&values[i]), &mut result[i]);
    }
    return result;
}