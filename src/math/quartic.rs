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

pub fn interpolate_batch(x_sets: &[u64], y_sets: &[u64]) -> Vec<u64> {

    let mut equations: Vec<u64> = Vec::with_capacity(x_sets.len() * 4);
    let mut inverses: Vec<u64> = Vec::with_capacity(x_sets.len());
    let mut eq_idx: usize = 0;

    for i in (0..x_sets.len()).step_by(4) {
        let xs = &x_sets[i..(i + 4)];
        
        let x01 = field::mul(xs[0], xs[1]);
        let x02 = field::mul(xs[0], xs[2]);
        let x03 = field::mul(xs[0], xs[3]);
        let x12 = field::mul(xs[1], xs[2]);
        let x13 = field::mul(xs[1], xs[3]);
        let x23 = field::mul(xs[2], xs[3]);
    
        // eq0
        equations.push(field::mul(field::neg(x12), xs[3]));
        equations.push(field::add(field::add(x12, x13), x23));
        equations.push(field::sub(field::sub(field::neg(xs[1]), xs[2]), xs[3]));
        equations.push(1);

        inverses.push(eval(&equations[eq_idx..(eq_idx + 4)], xs[0]));
        eq_idx += 4;

        // eq1
        equations.push(field::mul(field::neg(x02), xs[3]));
        equations.push(field::add(field::add(x02, x03), x23));
        equations.push(field::sub(field::sub(field::neg(xs[0]), xs[2]), xs[3]));
        equations.push(1);

        inverses.push(eval(&equations[eq_idx..(eq_idx + 4)], xs[1]));
        eq_idx += 4;

        // eq2
        equations.push(field::mul(field::neg(x01), xs[3]));
        equations.push(field::add(field::add(x01, x03), x13));
        equations.push(field::sub(field::sub(field::neg(xs[0]), xs[1]), xs[3]));
        equations.push(1);

        inverses.push(eval(&equations[eq_idx..(eq_idx + 4)], xs[2]));
        eq_idx += 4;

        // eq3
        equations.push(field::mul(field::neg(x01), xs[2]));
        equations.push(field::add(field::add(x01, x02), x12));
        equations.push(field::sub(field::sub(field::neg(xs[0]), xs[1]), xs[2]));
        equations.push(1);

        inverses.push(eval(&equations[eq_idx..(eq_idx + 4)], xs[3]));
        eq_idx += 4;
    }

    let inverses = field::inv_many(&inverses);

    let mut result: Vec<u64> = Vec::with_capacity(x_sets.len());
    for i in (0..x_sets.len()).step_by(4) {
        let ys = &y_sets[i..(i + 4)];

        let mut v = [0u64; 4];
        for j in 0..4 {
            let inv_y = field::mul(ys[j], inverses[i + j]);

            v[0] = field::add(v[0], field::mul(inv_y, equations[(i + j) * 4]));
            v[1] = field::add(v[1], field::mul(inv_y, equations[(i + j) * 4 + 1]));
            v[2] = field::add(v[2], field::mul(inv_y, equations[(i + j) * 4 + 2]));
            v[3] = field::add(v[3], field::mul(inv_y, equations[(i + j) * 4 + 3]));
        }

        result.push(v[0]);
        result.push(v[1]);
        result.push(v[2]);
        result.push(v[3]);
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
}