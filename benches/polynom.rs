use criterion::{ black_box, criterion_group, Criterion };
use distaff::{ Field, FiniteField, polynom, quartic };

pub fn eval(c: &mut Criterion) {
    let p = Field::rand_vector(1024);
    let x = Field::rand();
    c.bench_function("Poly eval", |bench| {
        bench.iter(|| polynom::eval(black_box(&p), black_box(x)))
    });
}

pub fn evaluate_quartic_batch(c: &mut Criterion) {
    let n: usize = 1 << 10;
    let x = Field::rand();
    let polys = quartic::to_quartic_vec(Field::rand_vector(n * 4));
    c.bench_function("Poly evaluation (quartic batch)", |bench| {
        bench.iter(|| quartic::evaluate_batch(black_box(&polys), black_box(x)))
    });
}

pub fn interpolate_quartic_batch(c: &mut Criterion) {
    let n: usize = 1 << 10;
    let r = Field::get_root_of_unity(n * 4);
    let xs = quartic::to_quartic_vec(Field::get_power_series(r, n * 4));
    let ys = quartic::to_quartic_vec(Field::rand_vector(n * 4));
    c.bench_function("Poly interpolation (quartic batch)", |bench| {
        bench.iter(|| quartic::interpolate_batch(black_box(&xs), black_box(&ys)))
    });
}


criterion_group!(group, eval, evaluate_quartic_batch, interpolate_quartic_batch);