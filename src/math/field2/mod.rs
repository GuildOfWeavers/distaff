pub enum Field { }

pub mod prime64;

pub trait FiniteField<T: Copy> {

    const MODULUS: T;

    const ZERO: T;
    const ONE: T;

    fn add(a: T, b: T) -> T;

    fn sub(a: T, b: T) -> T;

    fn mul(a: T, b: T) -> T;

    fn mul_acc(a: &mut [T], b: &[T], c: T) {
        for i in 0..a.len() {
            a[i] = Self::add(a[i], Self::mul(b[i], c));
        }
    }

    fn inv(x: T) -> T;

    fn inv_many(values: &[T]) -> Vec<T>;

    fn inv_many_fill(values: &[T], result: &mut [T]);

    fn div(a: T, b: T) -> T {
        let b = Self::inv(b);
        return Self::mul(a, b);
    }

    fn exp(b: T, p: T) -> T;

    fn neg(x: T) -> T {
        return Self::sub(Self::ZERO, x);
    }

    fn get_root_of_unity(order: usize) -> T;

    fn get_power_series(b: T, length: usize) -> Vec<T>;

    fn rand() -> T;

    fn rand_vector(length: usize) -> Vec<T>;

    fn prng(seed: [u8; 32]) -> T;

    fn prng_vector(seed: [u8; 32], length: usize) -> Vec<T>;
}