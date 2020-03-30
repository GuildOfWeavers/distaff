use crate::math::field;
use std::sync::{ atomic::{AtomicU64, Ordering}, Arc };
use std::thread;

// ADDITION
// ================================================================================================

/// Computes a[i] + b[i] for all i and returns the results. The addition is split into batches
/// which are distributed across multiple threads.
pub fn add(a: &[u64], b: &[u64], num_threads: usize) -> Vec<u64> {
    let n = a.len();
    assert!(n == b.len(), "number of values must be the same for both operands");
    assert!(n % num_threads == 0, "number of values must be divisible by number of threads");
    let batch_size = n / num_threads;

    // create atomic references to both operands
    let a = Arc::new(unsafe { &*(a as *const _ as *const [AtomicU64]) });
    let b = Arc::new(unsafe { &*(b as *const _ as *const [AtomicU64]) });

    // create a vector to hold the result
    let mut result: Vec<AtomicU64> = Vec::with_capacity(n);
    unsafe { result.set_len(n); };
    let result = Arc::new(result);

    // add batches of values in separate threads
    let mut handles = vec![];
    for i in (0..n).step_by(batch_size) {
        let a = Arc::clone(&a);
        let b = Arc::clone(&b);
        let result = Arc::clone(&result);
        let handle = thread::spawn(move || {
            for j in i..(i + batch_size) {
                let ai = a[j].load(Ordering::Relaxed);
                let bi = b[j].load(Ordering::Relaxed);
                result[j].store(field::add(ai, bi), Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }

    // wait until all threads are done
    for handle in handles {
        handle.join().unwrap();
    }

    // return the result
    let result = Arc::try_unwrap(result).unwrap();
    return from_atomic(result);
}

/// Computes a[i] + b[i] for all i and stores the results in b[i]. The addition is split into
/// batches which are distributed across multiple threads.
pub fn add_in_place(a: &[u64], b: &mut [u64], num_threads: usize) {
    let n = a.len();
    assert!(n == b.len(), "number of values to multiply must be the same for both operands");
    assert!(n % num_threads == 0, "number of values must be divisible by number of threads");
    let batch_size = n / num_threads;

    // create atomic references to both operands
    let a = Arc::new(unsafe { &*(a as *const _ as *const [AtomicU64]) });
    let b = Arc::new(unsafe { &*(b as *const _ as *const [AtomicU64]) });

    // multiply batches of values in separate threads
    let mut handles = vec![];
    for i in (0..n).step_by(batch_size) {
        let a = Arc::clone(&a);
        let b = Arc::clone(&b);
        let handle = thread::spawn(move || {
            for j in i..(i + batch_size) {
                let ai = a[j].load(Ordering::Relaxed);
                let bi = b[j].load(Ordering::Relaxed);
                b[j].store(field::add(ai, bi), Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }

    // wait until all threads are done
    for handle in handles {
        handle.join().unwrap();
    }
}

// MULTIPLICATION
// ================================================================================================

/// Computes a[i] * b[i] for all i and returns the results. The multiplication is split into
/// batches which are distributed across multiple threads.
pub fn mul(a: &[u64], b: &[u64], num_threads: usize) -> Vec<u64> {
    let n = a.len();
    assert!(n == b.len(), "number of values must be the same for both operands");
    assert!(n % num_threads == 0, "number of values must be divisible by number of threads");
    let batch_size = n / num_threads;

    // create atomic references to both operands
    let a = Arc::new(unsafe { &*(a as *const _ as *const [AtomicU64]) });
    let b = Arc::new(unsafe { &*(b as *const _ as *const [AtomicU64]) });

    // create a vector to hold the result
    let mut result: Vec<AtomicU64> = Vec::with_capacity(n);
    unsafe { result.set_len(n); };
    let result = Arc::new(result);

    // multiply batches of values in separate threads
    let mut handles = vec![];
    for i in (0..n).step_by(batch_size) {
        let a = Arc::clone(&a);
        let b = Arc::clone(&b);
        let result = Arc::clone(&result);
        let handle = thread::spawn(move || {
            for j in i..(i + batch_size) {
                let ai = a[j].load(Ordering::Relaxed);
                let bi = b[j].load(Ordering::Relaxed);
                result[j].store(field::mul(ai, bi), Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }

    // wait until all threads are done
    for handle in handles {
        handle.join().unwrap();
    }

    // return the result
    let result = Arc::try_unwrap(result).unwrap();
    return from_atomic(result);
}

/// Computes a[i] * b[i] for all i and stores the results in b[i]. The multiplication is 
/// split into batches which are distributed across multiple threads.
pub fn mul_in_place(a: &[u64], b: &mut [u64], num_threads: usize) {
    let n = a.len();
    assert!(n == b.len(), "number of values to multiply must be the same for both operands");
    assert!(n % num_threads == 0, "number of values must be divisible by number of threads");
    let batch_size = n / num_threads;

    // create atomic references to both operands
    let a = Arc::new(unsafe { &*(a as *const _ as *const [AtomicU64]) });
    let b = Arc::new(unsafe { &*(b as *const _ as *const [AtomicU64]) });

    // multiply batches of values in separate threads
    let mut handles = vec![];
    for i in (0..n).step_by(batch_size) {
        let a = Arc::clone(&a);
        let b = Arc::clone(&b);
        let handle = thread::spawn(move || {
            for j in i..(i + batch_size) {
                let ai = a[j].load(Ordering::Relaxed);
                let bi = b[j].load(Ordering::Relaxed);
                b[j].store(field::mul(ai, bi), Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }

    // wait until all threads are done
    for handle in handles {
        handle.join().unwrap();
    }
}

// INVERSION
// ================================================================================================

/// Computes multiplicative inverse of provided values. The inversion is split into batches which
/// are distributed across multiple threads.
pub fn inv(values: &[u64], num_threads: usize) -> Vec<u64> {
    let n = values.len();
    assert!(n % num_threads == 0, "number of values must be divisible by number of threads");
    let batch_size = n / num_threads;

    // create atomic references to the values
    let values = Arc::new(unsafe { &*(values as *const _ as *const [AtomicU64]) });

    // create a vector to hold the result
    let mut result: Vec<AtomicU64> = Vec::with_capacity(n);
    unsafe { result.set_len(n); };
    let result = Arc::new(result);

    // break up the values into batches and invert each batch in a separate thread
    let mut handles = vec![];
    for i in (0..n).step_by(batch_size) {
        let values = Arc::clone(&values);
        let result = Arc::clone(&result);
        let handle = thread::spawn(move || {
            let values_slice = &values[i..(i + batch_size)];
            let values_slice = unsafe { &*(values_slice as *const _ as *const [u64]) };
            let result_slice = &result[i..(i + batch_size)];
            let result_slice = unsafe { &mut *(result_slice as *const _ as *mut [u64]) };
            field::inv_many_fill(values_slice, result_slice);
        });
        handles.push(handle);
    }

    // wait until all threads are done
    for handle in handles {
        handle.join().unwrap();
    }

    // return the result
    let result = Arc::try_unwrap(result).unwrap();
    return from_atomic(result);
}

// HELPER FUNCTIONS
// ================================================================================================
fn from_atomic(v: Vec<AtomicU64>) -> Vec<u64> {
    let mut v = std::mem::ManuallyDrop::new(v);
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();
    return unsafe { Vec::from_raw_parts(p as *mut u64, len, cap) };
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    use crate::math::{ field };

    #[test]
    fn add() {

        let n: usize = 1024;
        let num_threads: usize = 4;
        let x = field::rand_vector(n);
        let y = field::rand_vector(n);

        // compute expected results
        let mut expected = vec![0u64; n];
        for i in 0..n {
            expected[i] = field::add(x[i], y[i]);
        }

        assert_eq!(expected, super::add(&x, &y, num_threads));
    }

    #[test]
    fn add_in_place() {

        let n: usize = 1024;
        let num_threads: usize = 4;
        let x = field::rand_vector(n);
        let y = field::rand_vector(n);

        // compute expected results
        let mut expected = vec![0u64; n];
        for i in 0..n {
            expected[i] = field::add(x[i], y[i]);
        }

        let mut z = y.clone();
        assert_eq!(expected, super::add(&x, &mut z, num_threads));
    }

    #[test]
    fn mul() {

        let n: usize = 1024;
        let num_threads: usize = 4;
        let x = field::rand_vector(n);
        let y = field::rand_vector(n);

        // compute expected results
        let mut expected = vec![0u64; n];
        for i in 0..n {
            expected[i] = field::mul(x[i], y[i]);
        }

        assert_eq!(expected, super::mul(&x, &y, num_threads));
    }

    #[test]
    fn mul_in_place() {

        let n: usize = 1024;
        let num_threads: usize = 4;
        let x = field::rand_vector(n);
        let y = field::rand_vector(n);

        // compute expected results
        let mut expected = vec![0u64; n];
        for i in 0..n {
            expected[i] = field::mul(x[i], y[i]);
        }

        let mut z = y.clone();
        assert_eq!(expected, super::mul(&x, &mut z, num_threads));
    }

    #[test]
    fn inv() {

        let n: usize = 1024;
        let num_threads: usize = 4;
        let v = field::rand_vector(n);

        // compute expected results
        let expected = field::inv_many(&v);

        assert_eq!(expected, super::inv(&v, num_threads));
    }
}