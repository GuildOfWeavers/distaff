use crossbeam_utils::thread;
use crate::math::{ field };
use crate::utils::{ uninit_vector };

// ADDITION
// ================================================================================================

/// Computes a[i] + b[i] for all i and returns the results. The addition is split into batches
/// which are distributed across multiple threads.
pub fn add(a: &[u128], b: &[u128], num_threads: usize) -> Vec<u128> {
    let n = a.len();
    assert!(n == b.len(), "number of values must be the same for both operands");
    assert!(n % num_threads == 0, "number of values must be divisible by number of threads");
    let batch_size = n / num_threads;

    // allocate space for the results
    let mut result = uninit_vector(n);

    // add batches of values in separate threads
    thread::scope(|s| {
        for i in (0..n).step_by(batch_size) {
            let result = unsafe { &mut *(&mut result[..] as *mut [u128]) };
            s.spawn(move |_| {
                for j in i..(i + batch_size) {
                    result[j] = field::add(a[j], b[j]);
                }
            });
        }
    }).unwrap();

    // return the result
    return result;
}

/// Computes a[i] + b[i] for all i and stores the results in b[i]. The addition is split into
/// batches which are distributed across multiple threads.
pub fn add_in_place(a: &mut [u128], b: &[u128], num_threads: usize) {
    let n = a.len();
    assert!(n == b.len(), "number of values must be the same for both operands");
    assert!(n % num_threads == 0, "number of values must be divisible by number of threads");
    let batch_size = n / num_threads;

    // add batches of values in separate threads
    thread::scope(|s| {
        for i in (0..n).step_by(batch_size) {
            let a = unsafe { &mut *(a as *mut [u128]) };
            s.spawn(move |_| {
                for j in i..(i + batch_size) {
                    a[j] = field::add(a[j], b[j]);
                }
            });
        }
    }).unwrap();
}

// SUBTRACTION
// ================================================================================================

/// Computes a[i] - b for all i and stores the results in a[i]. The subtraction is split into
/// batches which are distributed across multiple threads.
pub fn sub_const_in_place(a: &mut [u128], b: u128, num_threads: usize) {
    let n = a.len();
    assert!(n % num_threads == 0, "number of values must be divisible by number of threads");
    let batch_size = n / num_threads;

    // subtract batches of values in separate threads
    thread::scope(|s| {
        for i in (0..n).step_by(batch_size) {
            let a = unsafe { &mut *(a as *mut [u128]) };
            s.spawn(move |_| {
                for j in i..(i + batch_size) {
                    a[j] = field::sub(a[j], b);
                }
            });
        }
    }).unwrap();
}

// MULTIPLICATION
// ================================================================================================

/// Computes a[i] * b[i] for all i and returns the results. The multiplication is split into
/// batches which are distributed across multiple threads.
pub fn mul(a: &[u128], b: &[u128], num_threads: usize) -> Vec<u128> {
    let n = a.len();
    assert!(n == b.len(), "number of values must be the same for both operands");
    assert!(n % num_threads == 0, "number of values must be divisible by number of threads");
    let batch_size = n / num_threads;

    // allocate space for the results
    let mut result = uninit_vector(n);

    // multiply batches of values in separate threads
    thread::scope(|s| {
        for i in (0..n).step_by(batch_size) {
            let result = unsafe { &mut *(&mut result[..] as *mut [u128]) };
            s.spawn(move |_| {
                for j in i..(i + batch_size) {
                    result[j] = field::mul(a[j], b[j]);
                }
            });
        }
    }).unwrap();

    // return the result
    return result;
}

/// Computes a[i] * b[i] for all i and stores the results in b[i]. The multiplication is 
/// split into batches which are distributed across multiple threads.
pub fn mul_in_place(a: &mut [u128], b: &[u128], num_threads: usize) {
    let n = a.len();
    assert!(n == b.len(), "number of values must be the same for both operands");
    assert!(n % num_threads == 0, "number of values must be divisible by number of threads");
    let batch_size = n / num_threads;

    // multiply batches of values in separate threads
    thread::scope(|s| {
        for i in (0..n).step_by(batch_size) {
            let a = unsafe { &mut *(a as *mut [u128]) };
            s.spawn(move |_| {
                for j in i..(i + batch_size) {
                    a[j] = field::mul(a[j], b[j]);
                }
            });
        }
    }).unwrap();
}

/// Computes a[i] + b[i] * c for all i and saves result into a. The operation is 
/// split into batches which are distributed across multiple threads.
pub fn mul_acc(a: &mut[u128], b: &[u128], c: u128, num_threads: usize) {
    let n = a.len();
    assert!(n == b.len(), "number of values must be the same for both arrays");
    assert!(n % num_threads == 0, "number of values must be divisible by number of threads");
    let batch_size = n / num_threads;
    
    // accumulate batches of values in separate threads
    thread::scope(|s| {
        for i in (0..n).step_by(batch_size) {
            let a = unsafe { &mut *(a as *mut [u128]) };
            s.spawn(move |_| {
                for j in i..(i + batch_size) {
                    a[j] = field::add(a[j], field::mul(b[j], c));
                }
            });
        }
    }).unwrap();
}

// INVERSION
// ================================================================================================

/// Computes multiplicative inverse of provided values. The inversion is split into batches which
/// are distributed across multiple threads.
pub fn inv(values: &[u128], num_threads: usize) -> Vec<u128> {
    let n = values.len();
    assert!(n % num_threads == 0, "number of values must be divisible by number of threads");
    let batch_size = n / num_threads;

    // allocate space for the results
    let result = uninit_vector(n);

    // break up the values into batches and invert each batch in a separate thread
    thread::scope(|s| {
        for i in (0..n).step_by(batch_size) {
            let values_slice = &values[i..(i + batch_size)];
            let values_slice = unsafe { &*(values_slice as *const _ as *const [u128]) };
            let result_slice = &result[i..(i + batch_size)];
            let result_slice = unsafe { &mut *(result_slice as *const _ as *mut [u128]) };
            s.spawn(move |_| {
                field::inv_many_fill(values_slice, result_slice);
            });
        }
    }).unwrap();

    // return the result
    return result;
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
        let mut expected = vec![field::ZERO; n];
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
        let mut expected = vec![field::ZERO; n];
        for i in 0..n {
            expected[i] = field::add(x[i], y[i]);
        }

        let mut z = y.clone();
        super::add_in_place(&mut z, &x, num_threads);
        assert_eq!(expected, z);
    }

    #[test]
    fn sub_const_in_place() {

        let n: usize = 1024;
        let num_threads: usize = 4;
        let mut x = field::rand_vector(n);
        let y = field::rand();

        // compute expected results
        let mut expected = vec![field::ZERO; n];
        for i in 0..n {
            expected[i] = field::sub(x[i], y);
        }

        super::sub_const_in_place(&mut x, y, num_threads);
        assert_eq!(expected, x);
    }

    #[test]
    fn mul() {

        let n: usize = 1024;
        let num_threads: usize = 4;
        let x = field::rand_vector(n);
        let y = field::rand_vector(n);

        // compute expected results
        let mut expected = vec![field::ZERO; n];
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
        let mut expected = vec![field::ZERO; n];
        for i in 0..n {
            expected[i] = field::mul(x[i], y[i]);
        }

        let mut z = y.clone();
        super::mul_in_place(&mut z, &x, num_threads);
        assert_eq!(expected, z);
    }

    #[test]
    fn mul_acc() {
        let n: usize = 1024;
        let num_threads: usize = 4;
        let mut x = field::rand_vector(n);
        let y = field::rand_vector(n);
        let z = field::rand();

        // compute expected result
        let mut expected = x.clone();
        field::mul_acc(&mut expected, &y, z);

        super::mul_acc(&mut x, &y, z, num_threads);
        assert_eq!(expected, x);
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