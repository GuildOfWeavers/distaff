use std::mem;
use crate::math::{ FieldElement, FiniteField, fft };
use crate::utils::{ uninit_vector, filled_vector };

// POLYNOMIAL EVALUATION
// ================================================================================================

/// Evaluates polynomial `p` at coordinate `x`
pub fn eval<T>(p: &[T], x: T) -> T
    where T: FieldElement + FiniteField<T>
{
    let mut y = T::ZERO;
    let mut power_of_x = T::ONE;
    for i in 0..p.len() {
        y = T::add(y, T::mul(p[i], power_of_x));
        power_of_x = T::mul(power_of_x, x);
    }
    return y;
}

/// Evaluates polynomial `p` using FFT algorithm; the evaluation is done in-place, meaning
/// `p` is updated with results of the evaluation.
/// 
/// If `unpermute` parameter is set to false, the evaluations will be left in permuted state.
pub fn eval_fft<T>(p: &mut [T], unpermute: bool)
    where T: FieldElement + FiniteField<T>
{
    let g = T::get_root_of_unity(p.len());
    let twiddles = fft::get_twiddles(g, p.len());
    eval_fft_twiddles(p, &twiddles, unpermute);
}

/// Evaluates polynomial `p` using FFT algorithm; the evaluation is done in-place, meaning
/// `p` is updated with results of the evaluation. Unlike the previous function, this function
/// does not generate twiddles internally. Thus, the twiddles must be supplied as a parameter.
/// 
/// If `unpermute` parameter is set to false, the evaluations will be left in permuted state.
pub fn eval_fft_twiddles<T>(p: &mut [T], twiddles: &[T], unpermute: bool)
    where T: FieldElement + FiniteField<T>
{
    debug_assert!(p.len() == twiddles.len() * 2, "Invalid number of twiddles");
    // TODO: don't hard-code num_threads
    fft::fft_in_place(p, &twiddles, 1, 1, 0, 1);
    if unpermute {
        fft::permute(p);
    }
}

// POLYNOMIAL INTERPOLATION
// ================================================================================================

/// Uses Lagrange interpolation to build a polynomial from X and Y coordinates.
pub fn interpolate<T>(xs: &[T], ys: &[T]) -> Vec<T>
    where T: FieldElement + FiniteField<T>
{
    debug_assert!(xs.len() == ys.len(), "Number of X and Y coordinates must be the same");

    let roots = get_zero_roots(xs);
    let mut divisor = [T::ZERO, T::ONE];
    let mut numerators: Vec<Vec<T>> = Vec::with_capacity(xs.len());
    for i in 0..xs.len() {
        divisor[0] = T::neg(xs[i]);
        numerators.push(div(&roots, &divisor));
    }

    let mut denominators: Vec<T> = Vec::with_capacity(xs.len());
    for i in 0..xs.len() {
        denominators.push(eval(&numerators[i], xs[i]));
    }
    let denominators = T::inv_many(&denominators);

    let mut result = vec![T::ZERO; xs.len()];
    for i in 0..xs.len() {
        let y_slice = T::mul(ys[i], denominators[i]);
        for j in 0..xs.len() {
            if numerators[i][j] != T::ZERO && ys[i] != T::ZERO {
                result[j] = T::add(result[j], T::mul(numerators[i][j], y_slice));
            }
        }
    }

    return result;
}

/// Uses FFT algorithm to interpolate a polynomial from provided values `v`; the interpolation
/// is done in-place, meaning `v` is updated with polynomial coefficients.
/// 
/// If `unpermute` parameter is set to false, the coefficients will be left in permuted state.
pub fn interpolate_fft<T>(v: &mut [T], unpermute: bool)
    where T: FieldElement + FiniteField<T>
{
    let g = T::get_root_of_unity(v.len());
    let twiddles = fft::get_inv_twiddles(g, v.len());
    interpolate_fft_twiddles(v, &twiddles, unpermute);
}

/// Uses FFT algorithm to interpolate a polynomial from provided values `v`; the interpolation
/// is done in-place, meaning `v` is updated with polynomial coefficients. Unlike the previous
/// function, this function does not generate inverse twiddles internally. Thus, the twiddles
/// must be supplied as a parameter.
/// 
/// If `unpermute` parameter is set to false, the evaluations will be left in permuted state.
pub fn interpolate_fft_twiddles<T>(v: &mut [T], inv_twiddles: &[T], unpermute: bool)
    where T: FieldElement + FiniteField<T>
{
    // TODO: don't hard-code num_threads
    fft::fft_in_place(v, &inv_twiddles, 1, 1, 0, 1);
    let inv_length = T::inv(T::from(v.len()));
    for e in v.iter_mut() {
        *e = T::mul(*e, inv_length);
    }
    if unpermute {
        fft::permute(v);
    }
}

// POLYNOMIAL MATH OPERATIONS
// ================================================================================================

/// Adds polynomial `a` to polynomial `b`
pub fn add<T>(a: &[T], b: &[T]) -> Vec<T>
    where T: FieldElement + FiniteField<T>
{
    let result_len = std::cmp::max(a.len(), b.len());
    let mut result = Vec::with_capacity(result_len);
    for i in 0..result_len {
        let c1 = if i < a.len() { a[i] } else { T::ZERO };
        let c2 = if i < b.len() { b[i] } else { T::ZERO };
        result.push(T::add(c1, c2));
    }
    return result;
}

/// Subtracts polynomial `b` from polynomial `a`
pub fn sub<T>(a: &[T], b: &[T]) -> Vec<T>
    where T: FieldElement + FiniteField<T>
{
    let result_len = std::cmp::max(a.len(), b.len());
    let mut result = Vec::with_capacity(result_len);
    for i in 0..result_len {
        let c1 = if i < a.len() { a[i] } else { T::ZERO };
        let c2 = if i < b.len() { b[i] } else { T::ZERO };
        result.push(T::sub(c1, c2));
    }
    return result;
}

/// Multiplies polynomial `a` by polynomial `b`
pub fn mul<T>(a: &[T], b: &[T]) -> Vec<T>
    where T: FieldElement + FiniteField<T>
{
    let result_len = a.len() + b.len() - 1;
    let mut result = vec![T::ZERO; result_len];
    for i in 0..a.len() {
        for j in 0..b.len() {
            let s = T::mul(a[i], b[j]);
            result[i + j] = T::add(result[i + j], s);
        }
    }
    return result;
}

/// Multiplies every coefficient of polynomial `p` by constant `k`
pub fn mul_by_const<T>(p: &[T], k: T) -> Vec<T>
    where T: FieldElement + FiniteField<T>
{
    let mut result = Vec::with_capacity(p.len());
    for i in 0..p.len() {
        result.push(T::mul(p[i], k));
    }
    return result;
}

/// Divides polynomial `a` by polynomial `b`; if the polynomials don't divide evenly,
/// the remainder is ignored.
pub fn div<T>(a: &[T], b: &[T]) -> Vec<T>
    where T: FieldElement + FiniteField<T>
{
    
    let mut apos = degree_of(a);
    let mut a = a.to_vec();

    let bpos = degree_of(b);
    assert!(apos >= bpos, "cannot divide by polynomial of higher degree");
    if bpos == 0 {
        assert!(b[0] != T::ZERO, "cannot divide polynomial by zero");
    }

    let mut result = vec![T::ZERO; apos - bpos + 1];
    for i in (0..result.len()).rev() {
        let quot = T::div(a[apos], b[bpos]);
        result[i] = quot;
        for j in (0..bpos).rev() {
            a[i + j] = T::sub(a[i + j], T::mul(b[j], quot));
        }
        apos = apos.wrapping_sub(1);
    }

    return result;
}

/// Divides polynomial `a` by binomial (x - `b`) using Synthetic division method;
/// if the polynomials don't divide evenly, the remainder is ignored.
pub fn syn_div<T>(a: &[T], b: T) -> Vec<T>
    where T: FieldElement + FiniteField<T>
{
    let mut result = a.to_vec();
    syn_div_in_place(&mut result, b);
    return result;
}

/// Divides polynomial `a` by binomial (x - `b`) using Synthetic division method and stores the
/// result in `a`; if the polynomials don't divide evenly, the remainder is ignored.
pub fn syn_div_in_place<T>(a: &mut [T], b: T)
    where T: FieldElement + FiniteField<T>
{
    let mut c = T::ZERO;
    for i in (0..a.len()).rev() {
        let temp = T::add(a[i], T::mul(b, c));
        a[i] = c;
        c = temp;
    }
}

/// Divides polynomial `a` by polynomial (x^degree - 1) / (x - exceptions[i]) for all i using
/// Synthetic division method and stores the result in `a`; if the polynomials don't divide evenly,
/// the remainder is ignored.
pub fn syn_div_expanded_in_place<T>(a: &mut [T], degree: usize, exceptions: &[T])
    where T: FieldElement + FiniteField<T>
{

    // allocate space for the result
    let mut result = filled_vector(a.len(), a.len() + exceptions.len(), T::ZERO);

    // compute a / (x^degree - 1)
    result.copy_from_slice(&a);
    let degree_offset = a.len() - degree;
    for i in (0..degree_offset).rev() {
        result[i] = T::add(result[i], result[i + degree]);
    }

    // multiply result by (x - exceptions[i]) in place
    for &exception in exceptions {

        // exception term is negative
        let exception = T::neg(exception);

        // extend length of result since we are raising degree
        unsafe { result.set_len(result.len() + 1); }

        let mut next_term = result[0];
        result[0] = T::ZERO;
        for i in 0..(result.len() - 1) {
            result[i] = T::add(result[i], T::mul(next_term, exception));
            mem::swap(&mut next_term, &mut result[i + 1]);
        }
    }

    // copy result back into `a` skipping remainder terms
    a[..(degree_offset + exceptions.len())].copy_from_slice(&result[degree..]);

    // fill the rest of the result with 0
    for i in (degree_offset + exceptions.len())..a.len() { a[i] = T::ZERO; }
}

// DEGREE INFERENCE
// ================================================================================================

/// Returns degree of the polynomial `poly`
pub fn degree_of<T>(poly: &[T]) -> usize
    where T: FieldElement + FiniteField<T>
{
    for i in (0..poly.len()).rev() {
        if poly[i] != T::ZERO { return i; }
    }
    return 0;
}

/// Returns degree of a polynomial with which evaluates to `evaluations` over the domain of
/// corresponding roots of unity.
pub fn infer_degree(evaluations: &[u64]) -> usize {
    assert!(evaluations.len().is_power_of_two(), "number of evaluations must be a power of 2");
    let mut poly = evaluations.to_vec();
    interpolate_fft(&mut poly, true);
    return degree_of(&poly);
}

// HELPER FUNCTIONS
// ================================================================================================
fn get_zero_roots<T>(xs: &[T]) -> Vec<T>
    where T: FieldElement + FiniteField<T>
{
    let mut n = xs.len() + 1;
    let mut result = uninit_vector(n);
    
    n -= 1;
    result[n] = T::ONE;

    for i in 0..xs.len() {
        n -= 1;
        result[n] = T::ZERO;
        for j in n..xs.len() {
            result[j] = T::sub(result[j], T::mul(result[j + 1], xs[i]));
        }
    }

    return result;
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {

    use crate::math::{ F64, FiniteField };
    use crate::utils::remove_leading_zeros;

    #[test]
    fn eval() {
        let x = 11269864713250585702u64;
        let poly = [384863712573444386u64, 7682273369345308472, 13294661765012277990, 16234810094004944758];

        assert_eq!(0, super::eval(&[], x));
        assert_eq!(384863712573444386, super::eval(&poly[..1], x));   // constant
        assert_eq!(17042940544839738828, super::eval(&poly[..2], x)); // degree 1
        assert_eq!(6485711713712766590, super::eval(&poly[..3], x));  // degree 2
        assert_eq!(15417995579153477369, super::eval(&poly, x));      // degree 3
    }

    #[test]
    fn eval_fft() {
        let n: usize = 1024;

        // create a random polynomial
        let poly = F64::rand_vector(n);

        // evaluate polynomial using FFT
        let mut y1 = poly.clone();
        super::eval_fft(&mut y1, true);

        // evaluate polynomial using simple evaluation
        let roots = F64::get_power_series(F64::get_root_of_unity(n), n);
        let y2 = roots.iter().map(|&x| super::eval(&poly, x)).collect::<Vec<u64>>();
        
        assert_eq!(y1, y2);
    }

    #[test]
    fn add() {
        let poly1: [F64; 3] = [384863712573444386, 7682273369345308472, 13294661765012277990];
        let poly2: [F64; 3] = [9918505539874556741, 16401861429499852246, 12181445947541805654];

        // same degree
        let pr = vec![10303369252448001127, 5637390918409137421, 7029363832118060347];
        assert_eq!(pr, super::add(&poly1, &poly2));

        // poly1 is lower degree
        let pr = vec![10303369252448001127, 5637390918409137421, 12181445947541805654];
        assert_eq!(pr, super::add(&poly1[..2], &poly2));

        // poly2 is lower degree
        let pr = vec![10303369252448001127, 5637390918409137421, 13294661765012277990];
        assert_eq!(pr, super::add(&poly1, &poly2[..2]));
    }

    #[test]
    fn sub() {
        let poly1: [F64; 3] = [384863712573444386, 7682273369345308472, 13294661765012277990];
        let poly2: [F64; 3] = [9918505539874556741, 16401861429499852246, 12181445947541805654];

        // same degree
        let pr = vec![8913102053134910942, 9727155820281479523, 1113215817470472336];
        assert_eq!(pr, super::sub(&poly1, &poly2));

        // poly1 is lower degree
        let pr = vec![8913102053134910942, 9727155820281479523, 6265297932894217643];
        assert_eq!(pr, super::sub(&poly1[..2], &poly2));

        // poly2 is lower degree
        let pr = vec![8913102053134910942, 9727155820281479523, 13294661765012277990];
        assert_eq!(pr, super::sub(&poly1, &poly2[..2]));
    }

    #[test]
    fn mul() {
        let poly1: [F64; 3] = [384863712573444386, 7682273369345308472, 13294661765012277990];
        let poly2: [F64; 3] = [9918505539874556741, 16401861429499852246, 12181445947541805654];

        // same degree
        let pr = vec![3955396989677724641, 11645020397934612208, 5279606801653296505, 4127428352286805209, 5628361441431074344];
        assert_eq!(pr, super::mul(&poly1, &poly2));

        // poly1 is lower degree
        let pr = vec![3955396989677724641, 11645020397934612208, 3726230352653943207, 12439170984765704776];
        assert_eq!(pr, super::mul(&poly1[..2], &poly2));

        // poly2 is lower degree
        let pr = vec![3955396989677724641, 11645020397934612208, 13101514511927787479, 10135001247957123730];
        assert_eq!(pr, super::mul(&poly1, &poly2[..2]));
    }

    #[test]
    fn mul_by_const() {
        let poly = [384863712573444386, 7682273369345308472, 13294661765012277990];
        let c = 11269864713250585702u64;
        let pr = vec![14327042696637944021, 16658076832266294442, 5137918534171880203];
        assert_eq!(pr, super::mul_by_const(&poly, c));
    }

    #[test]
    fn div() {
        // divide degree 4 by degree 2
        let poly1: [F64; 5] = [3955396989677724641, 11645020397934612208, 5279606801653296505, 4127428352286805209, 5628361441431074344];
        let poly2: [F64; 3] = [384863712573444386, 7682273369345308472, 13294661765012277990];
        let pr = vec![9918505539874556741, 16401861429499852246, 12181445947541805654];
        assert_eq!(pr, super::div(&poly1, &poly2));

        // divide degree 3 by degree 2
        let poly1: [F64; 4] = [3955396989677724641, 11645020397934612208, 3726230352653943207, 12439170984765704776];
        let poly2: [F64; 3] = [9918505539874556741, 16401861429499852246, 12181445947541805654];
        let pr = vec![384863712573444386, 7682273369345308472];
        assert_eq!(pr, super::div(&poly1, &poly2));

        // divide degree 3 by degree 3
        let poly1: [F64; 3] = [14327042696637944021, 16658076832266294442, 5137918534171880203];
        let poly2: [F64; 3] = [384863712573444386, 7682273369345308472, 13294661765012277990];
        let pr = vec![11269864713250585702];
        assert_eq!(pr, super::div(&poly1, &poly2));
    }

    #[test]
    fn syn_div() {
        let poly = super::mul(&[2, 1], &[3, 1]);

        let result = super::syn_div(&poly, F64::neg(3));
        let expected = super::div(&poly, &[3, 1]);

        assert_eq!(expected, remove_leading_zeros(&result));
    }

    #[test]
    fn syn_div_expanded_in_place() {

        // build the polynomial
        let ys = vec![0, 1, 2, 3, 0, 5, 6, 7, 0, 9, 10, 11, 12, 13, 14, 15];
        let mut poly = ys.clone();
        super::interpolate_fft(&mut poly, true);

        // build the divisor polynomial
        let root = F64::get_root_of_unity(poly.len());
        let domain = F64::get_power_series(root, poly.len());

        let z_poly = vec![F64::neg(F64::ONE), 0, 0, 0, 1];
        let z_degree = z_poly.len() - 1;
        let z_poly = super::div(&z_poly, &[F64::neg(domain[12]), 1]);
        
        // compute the result
        let mut result = poly.clone();
        super::syn_div_expanded_in_place(&mut result, z_degree, &[domain[12]]);

        let expected = super::div(&poly, &z_poly);

        assert_eq!(expected, remove_leading_zeros(&result));
        assert_eq!(poly, remove_leading_zeros(&super::mul(&expected, &z_poly)));
    }

    #[test]
    fn degree_of() {
        assert_eq!(0, super::degree_of::<F64>(&[]));
        assert_eq!(0, super::degree_of::<F64>(&[1]));
        assert_eq!(1, super::degree_of::<F64>(&[1, 2]));
        assert_eq!(1, super::degree_of::<F64>(&[1, 2, 0]));
        assert_eq!(2, super::degree_of::<F64>(&[1, 2, 3]));
        assert_eq!(2, super::degree_of::<F64>(&[1, 2, 3, 0]));
    }

    #[test]
    fn infer_degree() {
        let poly = vec![1, 2, 3, 4];

        let mut evaluations = poly.clone();
        evaluations.resize(16, 0);
        super::eval_fft(&mut evaluations, true);
        assert_eq!(super::degree_of(&poly), super::infer_degree(&evaluations));

        let mut evaluations = poly.clone();
        evaluations.resize(32, 0);
        super::eval_fft(&mut evaluations, true);
        assert_eq!(super::degree_of(&poly), super::infer_degree(&evaluations));
    }
}