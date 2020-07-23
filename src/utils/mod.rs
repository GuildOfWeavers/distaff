use std::{ mem, slice, ops::Range };

// RE-EXPORTS
// ================================================================================================
pub mod hasher;
pub mod sponge;

// VECTOR FUNCTIONS
// ================================================================================================
pub fn uninit_vector<T>(length: usize) -> Vec<T> {
    let mut vector = Vec::with_capacity(length);
    unsafe { vector.set_len(length); }
    return vector;
}

pub fn filled_vector<T: Copy>(length: usize, capacity: usize, value: T) -> Vec<T> {
    let mut vector = vec![value; capacity];
    vector.truncate(length);
    return vector;
}

#[cfg(test)]
pub fn remove_leading_zeros(values: &[u128]) -> Vec<u128> {
    for i in (0..values.len()).rev() {
        if values[i] != 0 {
            return values[0..(i + 1)].to_vec();
        }
    }

    return [].to_vec();
}

// TYPE CONVERSIONS
// ================================================================================================
pub fn as_bytes<T>(values: &[T]) -> &[u8] {
    let value_size = mem::size_of::<T>();
    let result = unsafe {
        slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * value_size)
    };
    return result;
}

// RANGE
// ================================================================================================
pub trait RangeSlider {
    fn slide(self, slide_by: usize) -> Self;
}

impl RangeSlider for Range<usize> {
    fn slide(self, width: usize) -> Range<usize> {
        return Range {
            start   : self.end,
            end     : self.end + width
        };
    }
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {

    #[test]
    fn as_bytes() {
        let source: [u64; 4] = [1, 2, 3, 4];
        
        // should convert correctly
        let bytes = super::as_bytes(&source);
        let expected = [
            1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0,
            3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(expected, bytes);
    }
}