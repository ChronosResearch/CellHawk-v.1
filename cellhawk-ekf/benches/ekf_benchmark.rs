use cellhawk_ekf::CellhawkEKF;
use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use nalgebra::SVector;

fn bench_ekf_update(c: &mut Criterion) {
    let mut ekf = CellhawkEKF::new();
    let meas = SVector::<f64, 3>::new(10.0, 10.0, 0.0);

    c.bench_function("ekf_gnss_update", |b| {
        b.iter(|| ekf.update_gnss(black_box(10.0), black_box(meas)))
    });
}

criterion_group!(benches, bench_ekf_update);
criterion_main!(benches);
