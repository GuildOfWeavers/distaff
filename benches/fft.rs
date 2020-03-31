use criterion::{ black_box, criterion_group, Criterion };
use distaff::{ field, fft};

pub fn fft_in_place(c: &mut Criterion) {

    let size: usize = 1 << 12;
    let mut values = field::rand_vector(size);
    let r = field::get_root_of_unity(size as u64);
    let twiddles = fft::get_twiddles(r, size);

    c.bench_function("FFT (in-place)", |bench| {
        bench.iter(|| fft::fft_in_place(black_box(&mut values), black_box(&twiddles), black_box(1), black_box(1), black_box(0), black_box(1)))
    });
}

criterion_group!(group, fft_in_place);