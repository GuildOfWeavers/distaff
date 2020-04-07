use crate::math::{ field, fft, polys };
use crate::trace::{ TraceState };

// TYPES AND INTERFACES
// ================================================================================================
pub struct TraceTable {
    op_code     : Vec<u64>,
    op_bits     : [Vec<u64>; 8],
    copy_flag   : Vec<u64>,
    stack       : Vec<Vec<u64>>,
}

// TRACE TABLE IMPLEMENTATION
// ================================================================================================
impl TraceTable {

    pub fn new(trace_polys: &Vec<Vec<u64>>, extension_factor: usize) -> TraceTable {
        let domain_length = trace_polys[0].len() * extension_factor;
        let root = field::get_root_of_unity(domain_length as u64);
        let twiddles = fft::get_twiddles(root, domain_length);

        return TraceTable {
            op_code : eval_poly(&trace_polys[0], domain_length, &twiddles),
            op_bits: [
                eval_poly(&trace_polys[1], domain_length, &twiddles),
                eval_poly(&trace_polys[2], domain_length, &twiddles),
                eval_poly(&trace_polys[3], domain_length, &twiddles),
                eval_poly(&trace_polys[4], domain_length, &twiddles),
                eval_poly(&trace_polys[5], domain_length, &twiddles),
                eval_poly(&trace_polys[6], domain_length, &twiddles),
                eval_poly(&trace_polys[7], domain_length, &twiddles),
                eval_poly(&trace_polys[8], domain_length, &twiddles),
            ],
            copy_flag   : eval_poly(&trace_polys[9], domain_length, &twiddles),
            stack       : trace_polys[10..].into_iter().map(|p| eval_poly(p, domain_length, &twiddles)).collect()
        };
    }

    pub fn fill_state(&self, state: &mut TraceState, step: usize) {
        state.op_code = self.op_code[step];
        for i in 0..self.op_bits.len() {
            state.op_bits[i] = self.op_bits[i][step];
        }
        state.copy_flag = self.copy_flag[step];
        for i in 0..self.stack.len() {
            state.stack[i] = self.stack[i][step];
        }
    }

    pub fn len(&self) -> usize {
        return self.op_code.len();
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn eval_poly(trace_poly: &[u64], domain_length: usize, twiddles: &[u64]) -> Vec<u64> {
    let mut evaluations = vec![0u64; domain_length];
    evaluations[0..trace_poly.len()].copy_from_slice(trace_poly);
    polys::eval_fft_twiddles(&mut evaluations, &twiddles, true);
    return evaluations;
}