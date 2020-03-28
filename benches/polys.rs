use criterion::{ black_box, criterion_group, Criterion };
use distaff::{ field, polys, quartic };

pub fn eval(c: &mut Criterion) {
    let mut p = vec![0u64; 1024];
    field::rand_fill(&mut p);
    let x = field::rand();
    c.bench_function("Poly eval", |bench| {
        bench.iter(|| polys::eval(black_box(&p), black_box(x)))
    });
}

pub fn evaluate_quartic_batch(c: &mut Criterion) {
    let n: usize = 1 << 10;
    let r = field::get_root_of_unity(n as u64);
    let xs = field::get_power_series(r, n);
    let mut polys = vec![0u64; n * 4];
    field::rand_fill(&mut polys);
    c.bench_function("Poly evaluation (quartic batch)", |bench| {
        bench.iter(|| quartic::evaluate_batch(black_box(&polys), black_box(&xs)))
    });
}

pub fn interpolate_quartic_batch(c: &mut Criterion) {
    let n: usize = 1 << 10;
    let r = field::get_root_of_unity((n * 4) as u64);
    let xs = field::get_power_series(r, n * 4);
    let mut ys = vec![0u64; n * 4];
    field::rand_fill(&mut ys);
    c.bench_function("Poly interpolation (quartic batch)", |bench| {
        bench.iter(|| quartic::interpolate_batch(black_box(&xs), black_box(&ys)))
    });
}


criterion_group!(group, eval, evaluate_quartic_batch, interpolate_quartic_batch);