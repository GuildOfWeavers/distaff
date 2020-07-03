use crossbeam_utils::thread;
use crate::math::field;

// CONSTANTS
// ================================================================================================
const USIZE_BITS: usize = 0_usize.count_zeros() as usize;
const MAX_LOOP: usize = 256;

// PUBLIC FUNCTIONS
// ================================================================================================

/// In-place recursive FFT with permuted output. If `num_threads` is > 1, the computation is
/// performed in multiple threads. Number of threads must be a power of 2.
/// 
/// Adapted from: https://github.com/0xProject/OpenZKP/tree/master/algebra/primefield/src/fft
pub fn fft_in_place(values: &mut [u128], twiddles: &[u128], count: usize, stride: usize, offset: usize, num_threads: usize) {
    
    let size = values.len() / stride;
    debug_assert!(size.is_power_of_two());
    debug_assert!(offset < stride);
    debug_assert_eq!(values.len() % size, 0);
    debug_assert!(num_threads.is_power_of_two());
    
    // Keep recursing until size is 2
    if size > 2 {
        if stride == count && count < MAX_LOOP {
            fft_in_place(values, twiddles, 2 * count, 2 * stride, offset, num_threads);
        } else if num_threads > 1 {
            // run half of FFT in the current thread, and spin up a new thread for the other half
            thread::scope(|s| {
                // get another mutable reference to values to be used inside the new thread;
                // this is OK because halves of FFT don't step on each other
                let values2 = unsafe { &mut *(values as *mut [u128]) };
                s.spawn(move |_| {
                    fft_in_place(values2, twiddles, count, 2 * stride, offset, num_threads / 2);
                });
                fft_in_place(values, twiddles, count, 2 * stride, offset + stride, num_threads / 2);
            }).unwrap();
        }
        else {
            fft_in_place(values, twiddles, count, 2 * stride, offset, num_threads);
            fft_in_place(values, twiddles, count, 2 * stride, offset + stride, num_threads);
        }
    }

    for offset in offset..(offset + count) {
        butterfly(values, offset, stride);
    }

    let last_offset = offset + size * stride;
    for (i, offset) in (offset..last_offset).step_by(2 * stride).enumerate().skip(1) {
        for j in offset..(offset + count) {
            butterfly_twiddle(values, twiddles[i], j, stride);
        }
    }
}

pub fn get_twiddles(root: u128, size: usize) -> Vec<u128> {
    assert!(size.is_power_of_two());
    assert!(field::exp(root, size as u128) == field::ONE);
    let mut twiddles = field::get_power_series(root, size / 2);
    permute(&mut twiddles);
    return twiddles;
}

pub fn get_inv_twiddles(root: u128, size: usize) -> Vec<u128> {
    let inv_root = field::exp(root, (size - 1) as u128);
    return get_twiddles(inv_root, size);
}

pub fn permute(v: &mut [u128]) {
    let n = v.len();
    for i in 0..n {
        let j = permute_index(n, i);
        if j > i {
            v.swap(i, j);
        }
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn permute_index(size: usize, index: usize) -> usize {
    debug_assert!(index < size);
    if size == 1 { return 0 }
    debug_assert!(size.is_power_of_two());
    let bits = size.trailing_zeros() as usize;
    return index.reverse_bits() >> (USIZE_BITS - bits);
}

#[inline(always)]
fn butterfly(values: &mut [u128], offset: usize, stride: usize) {
    let i = offset;
    let j = offset + stride;
    let temp = values[i];
    values[i] = field::add(temp, values[j]);
    values[j] = field::sub(temp, values[j]);
}

#[inline(always)]
fn butterfly_twiddle(values: &mut [u128], twiddle: u128, offset: usize, stride: usize) {
    let i = offset;
    let j = offset + stride;
    let temp = values[i];
    values[j] = field::mul(values[j], twiddle);
    values[i] = field::add(temp, values[j]);
    values[j] = field::sub(temp, values[j]);
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    use crate::math::{ field, polynom };

    #[test]
    fn fft_in_place() {
        // degree 3
        let mut p: [u128; 4] = [1, 2, 3, 4];
        let g = field::get_root_of_unity(4);
        let xs = field::get_power_series(g, 4);
        let expected: Vec<u128> = xs.into_iter().map(|x| polynom::eval(&p, x)).collect();
        let twiddles = super::get_twiddles(g, 4);
        super::fft_in_place(&mut p, &twiddles, 1, 1, 0, 1);
        super::permute(&mut p);
        assert_eq!(expected, p);

        // degree 7
        let mut p: [u128; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let g = field::get_root_of_unity(8);
        let twiddles = super::get_twiddles(g, 8);
        let xs = field::get_power_series(g, 8);
        let expected: Vec<u128> = xs.into_iter().map(|x| polynom::eval(&p, x)).collect();
        super::fft_in_place(&mut p, &twiddles, 1, 1, 0, 1);
        super::permute(&mut p);
        assert_eq!(expected, p);

        // degree 15
        let mut p: [u128; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let g = field::get_root_of_unity(16);
        let twiddles = super::get_twiddles(g, 16);
        let xs = field::get_power_series(g, 16);
        let expected: Vec<u128> = xs.into_iter().map(|x| polynom::eval(&p, x)).collect();
        super::fft_in_place(&mut p, &twiddles, 1, 1, 0, 1);
        super::permute(&mut p);
        assert_eq!(expected, p);

        // degree 1023
        let mut p = field::rand_vector(1024);
        let g = field::get_root_of_unity(1024);
        let roots = field::get_power_series(g, 1024);
        let expected = roots.iter().map(|x| polynom::eval(&p, *x)).collect::<Vec<u128>>();
        let twiddles = super::get_twiddles(g, 1024);
        super::fft_in_place(&mut p, &twiddles, 1, 1, 0, 1);
        super::permute(&mut p);
        assert_eq!(expected, p);
    }
}