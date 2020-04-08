use crate::math::field::{ add, sub, mul };
use crate::trace::{ TraceState, opcodes };

pub fn evaluate(current: &TraceState, next: &TraceState, op_flags: &[u64; 32], table: &mut Vec<Vec<u64>>, step: usize) {

    let mut i = 0;

    // 8 constraints to enforce that op_bits are binary
    // degree: 2
    for _ in 0..8 {
        let v = current.op_bits[i];
        table[i][step] = sub(mul(v, v), v);
        i += 1;
    }

    let mut op_code = next.op_bits[0];
    op_code = add(op_code, mul(next.op_bits[1],   2));
    op_code = add(op_code, mul(next.op_bits[2],   4));
    op_code = add(op_code, mul(next.op_bits[3],   8));
    op_code = add(op_code, mul(next.op_bits[4],  16));
    op_code = add(op_code, mul(next.op_bits[5],  32));
    op_code = add(op_code, mul(next.op_bits[6],  64));
    op_code = add(op_code, mul(next.op_bits[7], 128));
    
    table[i][step] = sub(next.op_code, op_code);
}