use crate::math::field;

/// Evaluates degree 3 polynomial `p` at coordinate `x`. This function is about 30% faster than
/// the `polys::eval` function.
pub fn eval(p: &[u64], x: u64) -> u64 {
    debug_assert!(p.len() == 4, "Polynomial must have 4 terms");
    let mut y = field::add(p[0], field::mul(p[1], x));

    let x2 = field::mul(x, x);
    y = field::add(y, field::mul(p[2], x2));

    let x3 = field::mul(x2, x);
    y = field::add(y, field::mul(p[3], x3));

    return y;
}

/// Evaluates a batch of degree 3 polynomials at provided X coordinates. The polynomials are assumed
/// to be recorded as a sequence of 4 sequential coefficients.
pub fn evaluate_batch(polys: &[u64], xs: &[u64]) -> Vec<u64> {
    let n = polys.len() / 4;   // number of polynomials
    debug_assert!(polys.len() % 4 == 0, "each polynomial must contain 4 coefficients");
    debug_assert!(n == xs.len(), "number of polynomials must be equal to number of X coordinates");

    let mut result: Vec<u64> = Vec::with_capacity(n);
    unsafe { result.set_len(n); }

    for (i, j) in (0..n).zip((0..polys.len()).step_by(4)) {
        result[i] = eval(&polys[j..(j + 4)], xs[i]);
    }

    return result;
}

/// Interpolates a set of X, Y coordinates into a batch of degree 3 polynomials; The coordinates are
/// assumed to be in batches with 4 sequential coordinates per batch.
/// 
/// This function is many times faster than using `polys::interpolate` function in a loop. This is
/// primarily due to amortizing inversions over the entire batch.
pub fn interpolate_batch(xs: &[u64], ys: &[u64]) -> Vec<u64> {
    let n = xs.len() / 4;   // number of batches
    debug_assert!(xs.len() == ys.len(), "number of X coordinates must be equal to number of Y coordinates");
    debug_assert!(xs.len() % 4 == 0, "coordinate batches must consists of 4 coordinates per batch");

    let mut equations: Vec<u64> = Vec::with_capacity(n * 16);
    let mut inverses: Vec<u64> = Vec::with_capacity(n * 4);
    unsafe { 
        equations.set_len(n * 16);
        inverses.set_len(n * 4);
    }

    for (i, j) in (0..xs.len()).step_by(4).zip((0..equations.len()).step_by(16)) {
        
        let x01 = field::mul(xs[i + 0], xs[i + 1]);
        let x02 = field::mul(xs[i + 0], xs[i + 2]);
        let x03 = field::mul(xs[i + 0], xs[i + 3]);
        let x12 = field::mul(xs[i + 1], xs[i + 2]);
        let x13 = field::mul(xs[i + 1], xs[i + 3]);
        let x23 = field::mul(xs[i + 2], xs[i + 3]);

        // eq0
        equations[j + 0] = field::mul(field::neg(x12), xs[i + 3]);
        equations[j + 1] = field::add(field::add(x12, x13), x23);
        equations[j + 2] = field::sub(field::sub(field::neg(xs[i + 1]), xs[i + 2]), xs[i + 3]);
        equations[j + 3] = 1;
        inverses[i] = eval(&equations[j..(j + 4)], xs[i + 0]);

        // eq1
        equations[j + 4] = field::mul(field::neg(x02), xs[i + 3]);
        equations[j + 5] = field::add(field::add(x02, x03), x23);
        equations[j + 6] = field::sub(field::sub(field::neg(xs[i + 0]), xs[i + 2]), xs[i + 3]);
        equations[j + 7] = 1;
        inverses[i + 1] = eval(&equations[(j + 4)..(j + 8)], xs[i + 1]);

        // eq2
        equations[j +  8] = field::mul(field::neg(x01), xs[i + 3]);
        equations[j +  9] = field::add(field::add(x01, x03), x13);
        equations[j + 10] = field::sub(field::sub(field::neg(xs[i + 0]), xs[i + 1]), xs[i + 3]);
        equations[j + 11] = 1;
        inverses[i + 2] = eval(&equations[(j + 8)..(j + 12)], xs[i + 2]);

        // eq3
        equations[j + 12] = field::mul(field::neg(x01), xs[i + 2]);
        equations[j + 13] = field::add(field::add(x01, x02), x12);
        equations[j + 14] = field::sub(field::sub(field::neg(xs[i + 0]), xs[i + 1]), xs[i + 2]);
        equations[j + 15] = 1;
        inverses[i + 3] = eval(&equations[(j + 12)..(j + 16)], xs[i + 3]);
    }

    let inverses = field::inv_many(&inverses);

    let mut result: Vec<u64> = Vec::with_capacity(n * 4);
    unsafe { result.set_len(n * 4); }

    for (i, j) in (0..ys.len()).step_by(4).zip((0..equations.len()).step_by(16)) {
        
        // iteration 0
        let mut inv_y = field::mul(ys[i], inverses[i]);
        result[i + 0] = field::mul(inv_y, equations[j + 0]);
        result[i + 1] = field::mul(inv_y, equations[j + 1]);
        result[i + 2] = field::mul(inv_y, equations[j + 2]);
        result[i + 3] = field::mul(inv_y, equations[j + 3]);

        // iteration 1
        inv_y = field::mul(ys[i + 1], inverses[i + 1]);
        result[i + 0] = field::add(result[i + 0], field::mul(inv_y, equations[j + 4]));
        result[i + 1] = field::add(result[i + 1], field::mul(inv_y, equations[j + 5]));
        result[i + 2] = field::add(result[i + 2], field::mul(inv_y, equations[j + 6]));
        result[i + 3] = field::add(result[i + 3], field::mul(inv_y, equations[j + 7]));

        // iteration 2
        inv_y = field::mul(ys[i + 2], inverses[i + 2]);
        result[i + 0] = field::add(result[i + 0], field::mul(inv_y, equations[j +  8]));
        result[i + 1] = field::add(result[i + 1], field::mul(inv_y, equations[j +  9]));
        result[i + 2] = field::add(result[i + 2], field::mul(inv_y, equations[j + 10]));
        result[i + 3] = field::add(result[i + 3], field::mul(inv_y, equations[j + 11]));

        // iteration 3
        inv_y = field::mul(ys[i + 3], inverses[i + 3]);
        result[i + 0] = field::add(result[i + 0], field::mul(inv_y, equations[j + 12]));
        result[i + 1] = field::add(result[i + 1], field::mul(inv_y, equations[j + 13]));
        result[i + 2] = field::add(result[i + 2], field::mul(inv_y, equations[j + 14]));
        result[i + 3] = field::add(result[i + 3], field::mul(inv_y, equations[j + 15]));
    }

    return result;
}

#[cfg(test)]
mod tests {
    use crate::math::{ field };

    #[test]
    fn eval() {
        let x = 11269864713250585702u64;
        let poly = [384863712573444386u64, 7682273369345308472, 13294661765012277990, 16234810094004944758];
        assert_eq!(15417995579153477369, super::eval(&poly, x));
    }

    #[test]
    fn interpolate_batch() {
        let r = field::get_root_of_unity(16);
        let xs = field::get_power_series(r, 16);
        let ys = [1u64, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

        let expected = vec![
            7956382178997078105u64,  6172178935026293282,  5971474637801684060, 16793452009046991148,
               7956382178997078109, 15205743380705406848, 12475269242634339237,   194846859619262948,
               7956382178997078113, 12274564945409730015,  5971474637801684060,  1653291871389032149,
               7956382178997078117,  3241000499730616449, 12475269242634339237,  18251897020816760349
        ];
        assert_eq!(expected, super::interpolate_batch(&xs, &ys));
    }

    #[test]
    fn evaluate_batch() {
        let r = field::get_root_of_unity(16);
        let xs = field::get_power_series(r, 16).iter().step_by(4).map(|x| *x).collect::<Vec<u64>>();
        
        let polys = [
            7956382178997078105u64,  6172178935026293282,  5971474637801684060, 16793452009046991148,
               7956382178997078109, 15205743380705406848, 12475269242634339237,   194846859619262948,
               7956382178997078113, 12274564945409730015,  5971474637801684060,  1653291871389032149,
               7956382178997078117,  3241000499730616449, 12475269242634339237,  18251897020816760349
        ];
        let expected = vec![1u64, 5, 9, 13];
        assert_eq!(expected, super::evaluate_batch(&polys, &xs));
    }
}