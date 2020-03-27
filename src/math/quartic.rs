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

pub fn interpolate_batch(xSets: &[u64], ySets: &[u64]) -> Vec<u64> {

    let mut equations: Vec<u64> = Vec::with_capacity(xSets.len() * 4);
    let mut inverses: Vec<u64> = Vec::with_capacity(xSets.len());
    let mut eq_idx: usize = 0;

    for i in (0..xSets.len()).step_by(4) {
        let xs = &xSets[i..(i + 4)];
        
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

    eq_idx = 0;
    let result: Vec<u64> = Vec::with_capacity(xSets.len());
    for i in (0..xSets.len()).step_by(4) {
        let ys = &ySets[i..(i + 4)];

        let mut inv_y = field::mul(ys[0], inverses[i]);

    }


    return result;
}

#[cfg(test)]
mod tests {
    use super::{  eval as eval_poly };

    #[test]
    fn eval4() {
        let x = 11269864713250585702u64;
        let poly = [384863712573444386u64, 7682273369345308472, 13294661765012277990, 16234810094004944758];
        assert_eq!(15417995579153477369, eval_poly(&poly, x));
    }
}