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