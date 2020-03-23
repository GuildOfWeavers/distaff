use crate::math;

/// Evaluates a polynomial at a provided x coordinates
pub fn eval(poly: &[u64], x: u64) -> u64 {
    if poly.len() == 0 { return 0; }
    else if poly.len() == 1 { return poly[0]; }
    else if poly.len() == 2 { return math::add(poly[0], math::mul(poly[1], x)); }
    else if poly.len() == 3 {
        let y = math::add(poly[0], math::mul(poly[1], x));
        let x2 = math::mul(x, x);
        return math::add(y, math::mul(poly[2], x2));
    }
    else {
        let mut y = 0u64;
        let mut power_of_x = 1u64;
        for i in 0..poly.len() {
            y = math::add(y, math::mul(poly[i], power_of_x));
            power_of_x = math::mul(power_of_x, x);
        }
        return y;
    }
}

/// Computes a[i] + b[i] for all i
pub fn add(a: &[u64], b: &[u64]) -> Vec<u64> {
    let result_len = std::cmp::max(a.len(), b.len());
    let mut result = Vec::with_capacity(result_len);
    for i in 0..result_len {
        let c1 = if i < a.len() { a[i] } else { 0 };
        let c2 = if i < b.len() { b[i] } else { 0 };
        result.push(math::add(c1, c2));
    }
    return result;
}

/// Computes a[i] - b[i] for all i
pub fn sub(a: &[u64], b: &[u64]) -> Vec<u64> {
    let result_len = std::cmp::max(a.len(), b.len());
    let mut result = Vec::with_capacity(result_len);
    for i in 0..result_len {
        let c1 = if i < a.len() { a[i] } else { 0 };
        let c2 = if i < b.len() { b[i] } else { 0 };
        result.push(math::sub(c1, c2));
    }
    return result;
}

pub fn mul(poly1: &[u64], poly2: &[u64]) -> Vec<u64> {
    let result_len = poly1.len() + poly2.len() - 1;
    let mut result = vec![0u64; result_len];
    for i in 0..poly1.len() {
        for j in 0..poly2.len() {
            let s = math::mul(poly1[i], poly2[j]);
            result[i + j] = math::add(result[i + j], s);
        }
    }
    return result;
}

pub fn mul_by_const(poly: &[u64], c: u64) -> Vec<u64> {
    let mut result = Vec::with_capacity(poly.len());
    for i in 0..poly.len() {
        result.push(math::mul(poly[i], c));
    }
    return result;
}

pub fn div(poly1: &[u64], poly2: &[u64], result: &mut [u64]) {
    // TODO: implement
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    use super::{ 
        eval as eval_poly, add as add_polys, sub as sub_polys, mul as mul_polys, mul_by_const as mul_poly_by_const 
    };

    #[test]
    fn eval() {
        let x = 11269864713250585702u64;
        let poly = [384863712573444386u64, 7682273369345308472, 13294661765012277990, 16234810094004944758];

        assert_eq!(0, eval_poly(&[], x));
        assert_eq!(384863712573444386, eval_poly(&poly[..1], x));   // constant
        assert_eq!(17042940544839738828, eval_poly(&poly[..2], x)); // degree 1
        assert_eq!(6485711713712766590, eval_poly(&poly[..3], x));  // degree 2
        assert_eq!(15417995579153477369, eval_poly(&poly, x));      // degree 3
    }

    #[test]
    fn add() {
        let poly1 = [384863712573444386, 7682273369345308472, 13294661765012277990];
        let poly2 = [9918505539874556741, 16401861429499852246, 12181445947541805654];

        // same degree
        let pr = vec![10303369252448001127, 5637390918409137421, 7029363832118060347];
        assert_eq!(pr, add_polys(&poly1, &poly2));

        // poly1 is lower degree
        let pr = vec![10303369252448001127, 5637390918409137421, 12181445947541805654];
        assert_eq!(pr, add_polys(&poly1[..2], &poly2));

        // poly2 is lower degree
        let pr = vec![10303369252448001127, 5637390918409137421, 13294661765012277990];
        assert_eq!(pr, add_polys(&poly1, &poly2[..2]));
    }

    #[test]
    fn sub() {
        let poly1 = [384863712573444386, 7682273369345308472, 13294661765012277990];
        let poly2 = [9918505539874556741, 16401861429499852246, 12181445947541805654];

        // same degree
        let pr = vec![8913102053134910942, 9727155820281479523, 1113215817470472336];
        assert_eq!(pr, sub_polys(&poly1, &poly2));

        // poly1 is lower degree
        let pr = vec![8913102053134910942, 9727155820281479523, 6265297932894217643];
        assert_eq!(pr, sub_polys(&poly1[..2], &poly2));

        // poly2 is lower degree
        let pr = vec![8913102053134910942, 9727155820281479523, 13294661765012277990];
        assert_eq!(pr, sub_polys(&poly1, &poly2[..2]));
    }

    #[test]
    fn mul() {
        let poly1 = [384863712573444386, 7682273369345308472, 13294661765012277990];
        let poly2 = [9918505539874556741, 16401861429499852246, 12181445947541805654];

        // same degree
        let pr = vec![3955396989677724641, 11645020397934612208, 5279606801653296505, 4127428352286805209, 5628361441431074344];
        assert_eq!(pr, mul_polys(&poly1, &poly2));

        // poly1 is lower degree
        let pr = vec![3955396989677724641, 11645020397934612208, 3726230352653943207, 12439170984765704776];
        assert_eq!(pr, mul_polys(&poly1[..2], &poly2));

        // poly2 is lower degree
        let pr = vec![3955396989677724641, 11645020397934612208, 13101514511927787479, 10135001247957123730];
        assert_eq!(pr, mul_polys(&poly1, &poly2[..2]));
    }

    #[test]
    fn mul_by_constant() {
        let poly = [384863712573444386, 7682273369345308472, 13294661765012277990];
        let c = 11269864713250585702u64;
        let pr = vec![14327042696637944021, 16658076832266294442, 5137918534171880203];
        assert_eq!(pr, mul_poly_by_const(&poly, c));
    }
}