use crate::math;

pub fn fast_ft(values: &[u64], roots: &[u64], depth: usize, offset: usize) -> Vec<u64> {
    let step = 1 << depth;
    let result_length = roots.len() / step;

    if result_length <= 4 { 
        return base_ft(values, roots, depth, offset);
    }

    let even = fast_ft(values, roots, depth + 1, offset);
    let odd = fast_ft(values, roots, depth + 1, offset + step);

    let half_length = result_length / 2;
    let mut result = vec![0u64; result_length];
    for i in 0..half_length {
        let x = even[i];
        let y = odd[i];
        let y_times_root = math::mul(y, roots[i * step]);
        result[i] = math::add(x, y_times_root);
        result[i + half_length] = math::sub(x, y_times_root);
    }
    return result;
}

fn base_ft(values: &[u64], roots: &[u64], depth: usize, offset: usize) -> Vec<u64> {
    let step = 1 << depth;
    let mut result = vec![0u64; 4];
    for i in 0..4 {
        let mut last = math::mul(values[offset], roots[0]);
        last = math::add(last, math::mul(values[offset + step], roots[i * step]));
        last = math::add(last, math::mul(values[offset + 2 * step], roots[(i * 2) % 4 * step]));
        last = math::add(last, math::mul(values[offset + 3 * step], roots[(i * 3) % 4 * step]));
        result[i] = last;
    }
    return result;
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    use super::{ fast_ft as fft, math };

    #[test]
    fn fast_ft() {
        let p = [384863712573444386u64, 7682273369345308472, 13294661765012277990, 11269864713250585702];
        let r = math::get_root_of_unity(4);
        let mut roots = vec![0u64; 4];
        math::fill_power_series(r, &mut roots);

        let expected = vec![14184919679745593253, 12895889562832176027, 13174131275425851499, 16624745973598226656];
        assert_eq!(expected, fft(&p, &roots, 0, 0));
    }
}