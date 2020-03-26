use crate::{ math };

/// Evaluates degree 3 polynomial `p` at coordinate `x`. This function is about 30% faster than
/// the `polys::eval` function.
pub fn eval(p: &[u64], x: u64) -> u64 {
    debug_assert!(p.len() == 4, "Polynomial must have 4 terms");
    let mut y = math::add(p[0], math::mul(p[1], x));

    let x2 = math::mul(x, x);
    y = math::add(y, math::mul(p[2], x2));

    let x3 = math::mul(x2, x);
    y = math::add(y, math::mul(p[3], x3));

    return y;
}

pub fn interpolate_batch(xSets: &[u64], ySets: &[u64]) -> Vec<u64> {

    let mut equations: Vec<u64> = Vec::with_capacity(xSets.len() * 4);
    let mut inverses: Vec<u64> = Vec::with_capacity(xSets.len());
    let mut eq_idx: usize = 0;

    for i in (0..xSets.len()).step_by(4) {
        let xs = &xSets[i..(i + 4)];
        
        let x01 = math::mul(xs[0], xs[1]);
        let x02 = math::mul(xs[0], xs[2]);
        let x03 = math::mul(xs[0], xs[3]);
        let x12 = math::mul(xs[1], xs[2]);
        let x13 = math::mul(xs[1], xs[3]);
        let x23 = math::mul(xs[2], xs[3]);
    
        // eq0
        equations.push(math::mul(math::neg(x12), xs[3]));
        equations.push(math::add(math::add(x12, x13), x23));
        equations.push(math::sub(math::sub(math::neg(xs[1]), xs[2]), xs[3]));
        equations.push(1);

        inverses.push(eval(&equations[eq_idx..(eq_idx + 4)], xs[0]));
        eq_idx += 4;

        // eq1
        equations.push(math::mul(math::neg(x02), xs[3]));
        equations.push(math::add(math::add(x02, x03), x23));
        equations.push(math::sub(math::sub(math::neg(xs[0]), xs[2]), xs[3]));
        equations.push(1);

        inverses.push(eval(&equations[eq_idx..(eq_idx + 4)], xs[1]));
        eq_idx += 4;

        // eq2
        equations.push(math::mul(math::neg(x01), xs[3]));
        equations.push(math::add(math::add(x01, x03), x13));
        equations.push(math::sub(math::sub(math::neg(xs[0]), xs[1]), xs[3]));
        equations.push(1);

        inverses.push(eval(&equations[eq_idx..(eq_idx + 4)], xs[2]));
        eq_idx += 4;

        // eq3
        equations.push(math::mul(math::neg(x01), xs[2]));
        equations.push(math::add(math::add(x01, x02), x12));
        equations.push(math::sub(math::sub(math::neg(xs[0]), xs[1]), xs[2]));
        equations.push(1);

        inverses.push(eval(&equations[eq_idx..(eq_idx + 4)], xs[3]));
        eq_idx += 4;
    }

    let inverses = math::inv_many(&inverses);

    eq_idx = 0;
    let result: Vec<u64> = Vec::with_capacity(xSets.len());
    for i in (0..xSets.len()).step_by(4) {
        let ys = &ySets[i..(i + 4)];

        let mut inv_y = math::mul(ys[0], inverses[i]);

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