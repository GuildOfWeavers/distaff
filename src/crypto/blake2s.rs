use std::slice;

// CONSTANTS
// ================================================================================================
const IV: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19
];
const SIGMA: [u64; 20] = [
    0x0706050403020100, 0x0f0e0d0c0b0a0908, 0x060d0f0908040a0e, 0x0305070b02000c01, 0x0d0f0205000c080b,
    0x0409010706030e0a, 0x0e0b0c0d01030907, 0x080f00040a050602, 0x0f0a040207050009, 0x0d0308060c0b010e,
    0x03080b000a060c02, 0x09010e0f05070d04, 0x0a040d0e0f01050c, 0x0b08020903060700, 0x0903010c0e070b0d,
    0x0a020608040f0005, 0x0800030b090e0f06, 0x050a0401070d020c, 0x050106070408020a, 0x000d0c030e090b0f
];

// PUBLIC FUNCTIONS
// ================================================================================================
pub fn hash(values: &[u64], result: &mut [u64]) {

    let values = unsafe { slice::from_raw_parts(values.as_ptr() as *const u32, values.len() * 2) };
    let result: &mut [u32; 8] = unsafe { &mut *(result as *const _ as *mut [u32; 8]) };

    // initialize the context
    result[0] = 0x6b08e647; // IV[0] ^ 0x01010000 ^ 0 ^ 32
    result[1..8].copy_from_slice(&IV[1..8]);
    let mut t = 0u64;

    // run intermediate compressions
    let mut m = [0u32; 16];
    let mut v = [0u32; 16];

    let mut n = values.len();
    while n > 16 {
        m.copy_from_slice(&values[(t as usize)..((t as usize) + 16)]);
        t += 64;
        compress(&mut v, result, &m, t, false);
        n -= 16;
    }

    // run final compression
    if n > 0 {
        m[0..n].copy_from_slice(&values[(t as usize)..((t as usize) + n)]);
    }
    for i in n..15 {
        m[i] = 0;
    }
    t += (n as u64) * 4;
    compress(&mut v, result, &m, t, true);
}

// HELPER FUNCTIONS
// ================================================================================================
fn compress(v: &mut [u32; 16], h: &mut [u32; 8], m: &[u32; 16], t: u64, last: bool) {
    
    v[0..8].copy_from_slice(h);
    v[8..16].copy_from_slice(&IV);
    v[12] = v[12] ^ (t as u32);
    v[13] = v[13] ^ ((t >> 32) as u32);
    if last {
        v[14] = !v[14];
    }

    let mut i = 0;
    for _ in 0..10 {
        mix(v, m,  0, 16, 32, 48,  SIGMA[i] as u8,        (SIGMA[i] >>  8) as u8);
        mix(v, m,  4, 20, 36, 52, (SIGMA[i] >> 16) as u8, (SIGMA[i] >> 24) as u8);
        mix(v, m,  8, 24, 40, 56, (SIGMA[i] >> 32) as u8, (SIGMA[i] >> 40) as u8);
        mix(v, m, 12, 28, 44, 60, (SIGMA[i] >> 48) as u8, (SIGMA[i] >> 56) as u8);

        i += 1;
        mix(v, m,  0, 20, 40, 60,  SIGMA[i] as u8,        (SIGMA[i] >>  8) as u8);
        mix(v, m,  4, 24, 44, 48, (SIGMA[i] >> 16) as u8, (SIGMA[i] >> 24) as u8);
        mix(v, m,  8, 28, 32, 52, (SIGMA[i] >> 32) as u8, (SIGMA[i] >> 40) as u8);
        mix(v, m, 12, 16, 36, 56, (SIGMA[i] >> 48) as u8, (SIGMA[i] >> 56) as u8);

        i += 1;
    }

    for i in 0..8 {
        h[i] = h[i] ^ v[i] ^ v[i + 8];
    }
}

fn mix(v: &mut [u32; 16], m: &[u32; 16], a: u8, b: u8, c: u8, d: u8, xi: u8, yi: u8) {

    let a = (a / 4) as usize;
    let b = (b / 4) as usize;
    let c = (c / 4) as usize;
    let d = (d / 4) as usize;
    let xi = xi as usize;
    let yi = yi as usize;

    v[a] = v[a].wrapping_add(v[b]).wrapping_add(m[xi]);
    v[d] = (v[d] ^ v[a]).rotate_right(16);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(12);
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(m[yi]);
    v[d] = (v[d] ^ v[a]).rotate_right(8);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(7);
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {

    #[test]
    fn hash() {
        let values: Vec<u64> = vec![
             506097522914230528, 1084818905618843912, 1663540288323457296, 2242261671028070680,
            2820983053732684064, 3399704436437297448, 3978425819141910832, 4557147201846524216
        ];
        let mut result = vec![0u64; 4];
        super::hash(&values, &mut result);
        let expected: Vec<u64> = vec![
            10411853493597827926, 5881077485475197633, 14930829144967047688, 4515679653252488733
        ];

        assert_eq!(expected, result);
    }
}