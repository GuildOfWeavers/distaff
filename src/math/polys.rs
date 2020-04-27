use crate::math::{ field, fft };
use crate::utils::uninit_vector;

// POLYNOMIAL EVALUATION
// ================================================================================================

/// Evaluates polynomial `p` at coordinate `x`
pub fn eval(p: &[u64], x: u64) -> u64 {
    let mut y = 0u64;
    let mut power_of_x = 1u64;
    for i in 0..p.len() {
        y = field::add(y, field::mul(p[i], power_of_x));
        power_of_x = field::mul(power_of_x, x);
    }
    return y;
}

/// Evaluates polynomial `p` using FFT algorithm; the evaluation is done in-place, meaning
/// `p` is updated with results of the evaluation.
/// 
/// If `unpermute` parameter is set to false, the evaluations will be left in permuted state.
pub fn eval_fft(p: &mut [u64], unpermute: bool) {
    let g = field::get_root_of_unity(p.len() as u64);
    let twiddles = fft::get_twiddles(g, p.len());
    eval_fft_twiddles(p, &twiddles, unpermute);
}

/// Evaluates polynomial `p` using FFT algorithm; the evaluation is done in-place, meaning
/// `p` is updated with results of the evaluation. Unlike the previous function, this function
/// does not generate twiddles internally. Thus, the twiddles must be supplied as a parameter.
/// 
/// If `unpermute` parameter is set to false, the evaluations will be left in permuted state.
pub fn eval_fft_twiddles(p: &mut [u64], twiddles: &[u64], unpermute: bool) {
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
pub fn interpolate(xs: &[u64], ys: &[u64]) -> Vec<u64> {
    debug_assert!(xs.len() == ys.len(), "Number of X and Y coordinates must be the same");

    let roots = get_zero_roots(xs);
    let mut divisor = [0u64, 1];
    let mut numerators: Vec<Vec<u64>> = Vec::with_capacity(xs.len());
    for i in 0..xs.len() {
        divisor[0] = field::neg(xs[i]);
        numerators.push(div(&roots, &divisor));
    }

    let mut denominators: Vec<u64> = Vec::with_capacity(xs.len());
    for i in 0..xs.len() {
        denominators.push(eval(&numerators[i], xs[i]));
    }
    let denominators = field::inv_many(&denominators);

    let mut result = vec![0u64; xs.len()];
    for i in 0..xs.len() {
        let y_slice = field::mul(ys[i], denominators[i]);
        for j in 0..xs.len() {
            if numerators[i][j] != 0 && ys[i] != 0 {
                result[j] = field::add(result[j], field::mul(numerators[i][j], y_slice));
            }
        }
    }

    return result;
}

/// Uses FFT algorithm to interpolate a polynomial from provided values `v`; the interpolation
/// is done in-place, meaning `v` is updated with polynomial coefficients.
/// 
/// If `unpermute` parameter is set to false, the coefficients will be left in permuted state.
pub fn interpolate_fft(v: &mut [u64], unpermute: bool) {
    let g = field::get_root_of_unity(v.len() as u64);
    let twiddles = fft::get_inv_twiddles(g, v.len());
    interpolate_fft_twiddles(v, &twiddles, unpermute);
}

/// Uses FFT algorithm to interpolate a polynomial from provided values `v`; the interpolation
/// is done in-place, meaning `v` is updated with polynomial coefficients. Unlike the previous
/// function, this function does not generate inverse twiddles internally. Thus, the twiddles
/// must be supplied as a parameter.
/// 
/// If `unpermute` parameter is set to false, the evaluations will be left in permuted state.
pub fn interpolate_fft_twiddles(v: &mut [u64], inv_twiddles: &[u64], unpermute: bool) {
    // TODO: don't hard-code num_threads
    fft::fft_in_place(v, &inv_twiddles, 1, 1, 0, 1);
    let inv_length = field::inv(v.len() as u64);
    for e in v.iter_mut() {
        *e = field::mul(*e, inv_length);
    }
    if unpermute {
        fft::permute(v);
    }
}

// POLYNOMIAL MATH OPERATIONS
// ================================================================================================

/// Adds polynomial `a` to polynomial `b`
pub fn add(a: &[u64], b: &[u64]) -> Vec<u64> {
    let result_len = std::cmp::max(a.len(), b.len());
    let mut result = Vec::with_capacity(result_len);
    for i in 0..result_len {
        let c1 = if i < a.len() { a[i] } else { 0 };
        let c2 = if i < b.len() { b[i] } else { 0 };
        result.push(field::add(c1, c2));
    }
    return result;
}

/// Subtracts polynomial `b` from polynomial `a`
pub fn sub(a: &[u64], b: &[u64]) -> Vec<u64> {
    let result_len = std::cmp::max(a.len(), b.len());
    let mut result = Vec::with_capacity(result_len);
    for i in 0..result_len {
        let c1 = if i < a.len() { a[i] } else { 0 };
        let c2 = if i < b.len() { b[i] } else { 0 };
        result.push(field::sub(c1, c2));
    }
    return result;
}

/// Multiplies polynomial `a` by polynomial `b`
pub fn mul(a: &[u64], b: &[u64]) -> Vec<u64> {
    let result_len = a.len() + b.len() - 1;
    let mut result = vec![0u64; result_len];
    for i in 0..a.len() {
        for j in 0..b.len() {
            let s = field::mul(a[i], b[j]);
            result[i + j] = field::add(result[i + j], s);
        }
    }
    return result;
}

/// Multiplies every coefficient of polynomial `p` by constant `k`
pub fn mul_by_const(p: &[u64], k: u64) -> Vec<u64> {
    let mut result = Vec::with_capacity(p.len());
    for i in 0..p.len() {
        result.push(field::mul(p[i], k));
    }
    return result;
}

/// Divides polynomial `a` by polynomial `b`; if the polynomials don't divide evenly,
/// the remainder is ignored.
pub fn div(a: &[u64], b: &[u64]) -> Vec<u64> {
    
    let mut apos = get_last_non_zero_index(a);
    let mut a = a.to_vec();

    let bpos = get_last_non_zero_index(b);
    assert!(apos >= bpos, "cannot divide by polynomial of higher order");

    let mut result = vec![0u64; apos - bpos + 1];
    for i in (0..result.len()).rev() {
        let quot = field::div(a[apos], b[bpos]);
        result[i] = quot;
        for j in (0..bpos).rev() {
            a[i + j] = field::sub(a[i + j], field::mul(b[j], quot));
        }
        apos = apos.wrapping_sub(1);
    }

    return result;
}

/// Divides polynomial `a` by binomial (x + `b`) using Synthetic division method;
/// if the polynomials don't divide evenly, the remainder is ignored.
pub fn syn_div(a: &[u64], b: u64) -> Vec<u64> {
    let mut result = a.to_vec();
    syn_div_in_place(&mut result, b);
    return result;
}

/// Divides polynomial `a` by binomial (x + `b`) using Synthetic division method and stores the
/// result in `a`; if the polynomials don't divide evenly, the remainder is ignored.
pub fn syn_div_in_place(a: &mut [u64], b: u64) {
    let b = field::neg(b);
    let mut c = 0;
    for i in (0..a.len()).rev() {
        let temp = field::add(a[i], field::mul(b, c));
        a[i] = c;
        c = temp;
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn get_last_non_zero_index(vec: &[u64]) -> usize {
    for i in (0..vec.len()).rev() {
        if vec[i] != 0 { return i; }
    }
    return vec.len();
}

fn get_zero_roots(xs: &[u64]) -> Vec<u64> {
    let mut n = xs.len() + 1;
    let mut result = uninit_vector(n);
    
    n -= 1;
    result[n] = 1;

    for i in 0..xs.len() {
        n -= 1;
        result[n] = 0;
        for j in n..xs.len() {
            result[j] = field::sub(result[j], field::mul(result[j + 1], xs[i]));
        }
    }

    return result;
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    use crate::math::field;

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
        let poly = field::rand_vector(n);

        // evaluate polynomial using FFT
        let mut y1 = poly.clone();
        super::eval_fft(&mut y1, true);

        // evaluate polynomial using simple evaluation
        let roots = field::get_power_series(field::get_root_of_unity(n as u64), n);
        let y2 = roots.iter().map(|&x| super::eval(&poly, x)).collect::<Vec<u64>>();
        
        assert_eq!(y1, y2);
    }

    #[test]
    fn add() {
        let poly1 = [384863712573444386, 7682273369345308472, 13294661765012277990];
        let poly2 = [9918505539874556741, 16401861429499852246, 12181445947541805654];

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
        let poly1 = [384863712573444386, 7682273369345308472, 13294661765012277990];
        let poly2 = [9918505539874556741, 16401861429499852246, 12181445947541805654];

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
        let poly1 = [384863712573444386, 7682273369345308472, 13294661765012277990];
        let poly2 = [9918505539874556741, 16401861429499852246, 12181445947541805654];

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
        let poly1 = [3955396989677724641, 11645020397934612208, 5279606801653296505, 4127428352286805209, 5628361441431074344];
        let poly2 = [384863712573444386, 7682273369345308472, 13294661765012277990];
        let pr = vec![9918505539874556741, 16401861429499852246, 12181445947541805654];
        assert_eq!(pr, super::div(&poly1, &poly2));

        // divide degree 3 by degree 2
        let poly1 = [3955396989677724641, 11645020397934612208, 3726230352653943207, 12439170984765704776];
        let poly2 = [9918505539874556741, 16401861429499852246, 12181445947541805654];
        let pr = vec![384863712573444386, 7682273369345308472];
        assert_eq!(pr, super::div(&poly1, &poly2));

        // divide degree 3 by degree 3
        let poly1 = [14327042696637944021, 16658076832266294442, 5137918534171880203];
        let poly2 = [384863712573444386, 7682273369345308472, 13294661765012277990];
        let pr = vec![11269864713250585702];
        assert_eq!(pr, super::div(&poly1, &poly2));
    }

    #[test]
    fn syn_div() {
        let poly = super::mul(&[2, 1], &[3, 1]);

        let result = super::syn_div(&poly, 3);
        let mut expected = super::div(&poly, &[3, 1]);
        // syn_div() does not get rid of leading zeros
        expected.resize(result.len(), 0);

        assert_eq!(expected, result);
    }
}