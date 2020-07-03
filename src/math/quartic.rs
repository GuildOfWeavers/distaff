use crate::math::{ field };
use crate::utils::uninit_vector;

/// Evaluates degree 3 polynomial `p` at coordinate `x`. This function is about 30% faster than
/// the `polys::eval` function.
pub fn eval(p: &[u128], x: u128) -> u128 {
    debug_assert!(p.len() == 4, "Polynomial must have 4 terms");
    let mut y = field::add(p[0], field::mul(p[1], x));

    let x2 = field::mul(x, x);
    y = field::add(y, field::mul(p[2], x2));

    let x3 = field::mul(x2, x);
    y = field::add(y, field::mul(p[3], x3));

    return y;
}

/// Evaluates a batch of degree 3 polynomials at the provided X coordinate.
pub fn evaluate_batch(polys: &[[u128; 4]], x: u128) -> Vec<u128> {
    let n = polys.len();
    
    let mut result: Vec<u128> = Vec::with_capacity(n);
    unsafe { result.set_len(n); }

    for i in 0..n {
        result[i] = eval(&polys[i], x);
    }

    return result;
}

/// Interpolates a set of X, Y coordinates into a batch of degree 3 polynomials.
/// 
/// This function is many times faster than using `polys::interpolate` function in a loop. This is
/// primarily due to amortizing inversions over the entire batch.
pub fn interpolate_batch(xs: &[[u128; 4]], ys: &[[u128; 4]]) -> Vec<[u128; 4]> {
    debug_assert!(xs.len() == ys.len(), "number of X coordinates must be equal to number of Y coordinates");

    let n = xs.len();
    let mut equations: Vec<[u128; 4]> = Vec::with_capacity(n * 4);
    let mut inverses: Vec<u128> = Vec::with_capacity(n * 4);
    unsafe { 
        equations.set_len(n * 4);
        inverses.set_len(n * 4);
    }

    for (i, j) in (0..n).zip((0..equations.len()).step_by(4)) {
        
        let xs = xs[i];

        let x01 = field::mul(xs[0], xs[1]);
        let x02 = field::mul(xs[0], xs[2]);
        let x03 = field::mul(xs[0], xs[3]);
        let x12 = field::mul(xs[1], xs[2]);
        let x13 = field::mul(xs[1], xs[3]);
        let x23 = field::mul(xs[2], xs[3]);

        // eq0
        equations[j] = [
            field::mul(field::neg(x12), xs[3]),
            field::add(field::add(x12, x13), x23),
            field::sub(field::sub(field::neg(xs[1]), xs[2]), xs[3]),
            field::ONE
        ];
        inverses[j] = eval(&equations[j], xs[0]);

        // eq1
        equations[j + 1] = [
            field::mul(field::neg(x02), xs[3]),
            field::add(field::add(x02, x03), x23),
            field::sub(field::sub(field::neg(xs[0]), xs[2]), xs[3]),
            field::ONE
        ];
        inverses[j + 1] = eval(&equations[j + 1], xs[1]);

        // eq2
        equations[j + 2] = [
            field::mul(field::neg(x01), xs[3]),
            field::add(field::add(x01, x03), x13),
            field::sub(field::sub(field::neg(xs[0]), xs[1]), xs[3]),
            field::ONE
        ];
        inverses[j + 2] = eval(&equations[j + 2], xs[2]);

        // eq3
        equations[j + 3] = [
            field::mul(field::neg(x01), xs[2]),
            field::add(field::add(x01, x02), x12),
            field::sub(field::sub(field::neg(xs[0]), xs[1]), xs[2]),
            field::ONE
        ];
        inverses[j + 3] = eval(&equations[j + 3], xs[3]);
    }

    let inverses = field::inv_many(&inverses);

    let mut result: Vec<[u128; 4]> = Vec::with_capacity(n);
    unsafe { result.set_len(n); }

    for (i, j) in (0..n).zip((0..equations.len()).step_by(4)) {
        
        let ys = ys[i];

        // iteration 0
        let mut inv_y = field::mul(ys[0], inverses[j]);
        result[i][0] = field::mul(inv_y, equations[j][0]);
        result[i][1] = field::mul(inv_y, equations[j][1]);
        result[i][2] = field::mul(inv_y, equations[j][2]);
        result[i][3] = field::mul(inv_y, equations[j][3]);

        // iteration 1
        inv_y = field::mul(ys[1], inverses[j + 1]);
        result[i][0] = field::add(result[i][0], field::mul(inv_y, equations[j + 1][0]));
        result[i][1] = field::add(result[i][1], field::mul(inv_y, equations[j + 1][1]));
        result[i][2] = field::add(result[i][2], field::mul(inv_y, equations[j + 1][2]));
        result[i][3] = field::add(result[i][3], field::mul(inv_y, equations[j + 1][3]));

        // iteration 2
        inv_y = field::mul(ys[2], inverses[j + 2]);
        result[i][0] = field::add(result[i][0], field::mul(inv_y, equations[j + 2][0]));
        result[i][1] = field::add(result[i][1], field::mul(inv_y, equations[j + 2][1]));
        result[i][2] = field::add(result[i][2], field::mul(inv_y, equations[j + 2][2]));
        result[i][3] = field::add(result[i][3], field::mul(inv_y, equations[j + 2][3]));

        // iteration 3
        inv_y = field::mul(ys[3], inverses[j + 3]);
        result[i][0] = field::add(result[i][0], field::mul(inv_y, equations[j + 3][0]));
        result[i][1] = field::add(result[i][1], field::mul(inv_y, equations[j + 3][1]));
        result[i][2] = field::add(result[i][2], field::mul(inv_y, equations[j + 3][2]));
        result[i][3] = field::add(result[i][3], field::mul(inv_y, equations[j + 3][3]));
    }

    return result;
}

pub fn transpose(vector: &[u128], stride: usize) -> Vec<[u128; 4]> {
    assert!(vector.len() % (4 * stride) == 0, "vector length must be divisible by {}", 4 * stride);
    let row_count = vector.len() / (4 * stride);

    let mut result = to_quartic_vec(uninit_vector(row_count * 4));
    for i in 0..row_count {
        result[i] = [
            vector[i * stride],
            vector[(i + row_count) * stride],
            vector[(i + 2 * row_count) * stride],
            vector[(i + 3 * row_count) * stride]
        ];
    }

    return result;
}

/// Re-interprets a vector of integers as a vector of quartic elements.
pub fn to_quartic_vec(vector: Vec<u128>) -> Vec<[u128; 4]> {
    assert!(vector.len() % 4 == 0, "vector length must be divisible by 4");
    let mut v = std::mem::ManuallyDrop::new(vector);
    let p = v.as_mut_ptr();
    let len = v.len() / 4;
    let cap = v.capacity() / 4;
    return unsafe { Vec::from_raw_parts(p as *mut [u128; 4], len, cap) };
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    use crate::math::{ field, polynom };

    #[test]
    fn eval() {
        let x: u128 = 11269864713250585702;
        let poly: [u128; 4] = [384863712573444386, 7682273369345308472, 13294661765012277990, 16234810094004944758];
        assert_eq!(polynom::eval(&poly, x), super::eval(&poly, x));
    }

    #[test]
    fn interpolate_batch() {
        let r = field::get_root_of_unity(16);
        let xs = super::to_quartic_vec(field::get_power_series(r, 16));
        let ys = super::to_quartic_vec(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);

        let mut expected: Vec<[u128; 4]> = vec![];
        for i in 0..xs.len() {
            let mut row = [0u128; 4];
            row.copy_from_slice(&polynom::interpolate(&xs[i], &ys[i]));
            expected.push(row);
        }

        assert_eq!(expected, super::interpolate_batch(&xs, &ys));
    }

    #[test]
    fn evaluate_batch() {
        let x = field::rand();
        let polys: [[u128; 4]; 4] = [
            [7956382178997078105,  6172178935026293282,  5971474637801684060, 16793452009046991148],
            [7956382178997078109, 15205743380705406848, 12475269242634339237,   194846859619262948],
            [7956382178997078113, 12274564945409730015,  5971474637801684060,  1653291871389032149],
            [7956382178997078117,  3241000499730616449, 12475269242634339237, 18251897020816760349]
        ];

        let expected = vec![
            polynom::eval(&polys[0], x),
            polynom::eval(&polys[1], x),
            polynom::eval(&polys[2], x),
            polynom::eval(&polys[3], x)
        ];
        assert_eq!(expected, super::evaluate_batch(&polys, x));
    }

    #[test]
    fn to_quartic_vec() {
        let vector = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let expected: Vec<[u128; 4]> = vec![[1, 2, 3, 4], [5, 6, 7, 8], [9, 10, 11, 12], [13, 14, 15, 16]];
        assert_eq!(expected, super::to_quartic_vec(vector));
    }

    #[test]
    fn transpose() {
        let vector = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let expected: Vec<[u128; 4]> = vec![[1, 5, 9, 13], [2, 6, 10, 14], [3, 7, 11, 15], [4, 8, 12, 16]];
        assert_eq!(expected, super::transpose(&vector, 1));

        let expected: Vec<[u128; 4]> = vec![[1, 5, 9, 13], [3, 7, 11, 15]];
        assert_eq!(expected, super::transpose(&vector, 2));
    }
}