use crate::math::field;
use std::sync::{ atomic::{AtomicU64, Ordering}, Arc };
use std::thread;

// MULTIPLICATION
// ================================================================================================

pub fn mul(a: &[u64], b: &[u64], num_threads: usize) -> Vec<u64> {
    let n = a.len();
    assert!(n == b.len(), "number of values must be the same for both operands");
    assert!(n % num_threads == 0, "number of values must be divisible by number of threads");
    let batch_size = n / num_threads;

    // create atomic references to both operands
    let a = Arc::new(unsafe { &*(a as *const _ as *const [AtomicU64]) });
    let b = Arc::new(unsafe { &*(b as *const _ as *const [AtomicU64]) });

    // create a vector to cold the result
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

// HELPER FUNCTIONS
// ================================================================================================
fn from_atomic(v: Vec<AtomicU64>) -> Vec<u64> {
    let mut v = std::mem::ManuallyDrop::new(v);
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();
    return unsafe { Vec::from_raw_parts(p as *mut u64, len, cap) };
}