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

pub fn remove_leading_zeros(values: &[u64]) -> Vec<u64> {
    for i in (0..values.len()).rev() {
        if values[i] != 0 {
            return values[0..(i + 1)].to_vec();
        }
    }

    return [].to_vec();
}

// TYPE CONVERSIONS
// ================================================================================================
pub trait CopyInto<T> {
    fn copy_into(&self) -> T;
}

impl CopyInto<[u8; 32]> for [u64; 4] {
    fn copy_into(&self) -> [u8; 32] {
        return unsafe { *(self as *const [u64; 4] as *const [u8; 32]) };
    }
}

impl CopyInto<[u64; 4]> for [u8; 32] {
    fn copy_into(&self) -> [u64; 4] {
        return unsafe { *(self as *const [u8; 32] as *const [u64; 4]) };
    }
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    
    use super::CopyInto;

    #[test]
    fn u8x32_into_u64x8() {
        let mut source: [u8; 32] = [
            1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0,
            3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0,
        ];

        let mut target: [u64; 4] = source.copy_into();

        // data is copied correctly
        assert_eq!([1, 2, 3, 4], target);

        // changing target data does not change source
        target[0] = 6;
        let expected = [
            1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0,
            3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(expected, source);

        // changing source doesn't change target
        source[0] = 7;
        assert_eq!([6, 2, 3, 4], target);
    }

    #[test]
    fn u64x4_into_u8x32() {
        let mut source: [u64; 4] = [1, 2, 3, 4];
        let mut target: [u8; 32] = source.copy_into();

        // data is copied correctly
        let expected = [
            1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0,
            3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(expected, target);

        // changing target data does not change source
        target[0] = 6;
        assert_eq!([1, 2, 3, 4], source);

        // changing source doesn't change target
        source[0] = 7;
        let expected = [
            6, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0,
            3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(expected, target);
    }
}