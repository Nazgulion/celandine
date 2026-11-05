//! Paired-input and reusable-law mutual information costs.
#![allow(missing_docs)]
#[path = "support/joint_input.rs"]
mod joint_input;
use celandine::{
    distribution::joint_probabilities,
    information::{mutual_information, mutual_information_distribution},
};
use criterion::{
    BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};
use std::{hint::black_box, time::Duration};
fn benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("mutual_bytes");
    group.sampling_mode(SamplingMode::Flat);
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        for size in [
            16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760, 104857600,
        ] {
            let (x, y) = joint_input::input(size, shape);
            group.throughput(Throughput::Bytes(2 * size as u64));
            group.bench_function(BenchmarkId::new(shape, size), |b| {
                b.iter(|| black_box(mutual_information(black_box(&x), black_box(&y)).unwrap()))
            });
        }
    }
    group.finish();
    let mut group = c.benchmark_group("mutual_reuse");
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        let (x, y) = joint_input::input(65536, shape);
        let d = joint_probabilities(&x, &y).unwrap();
        group.bench_function(BenchmarkId::new(shape, 65536), |b| {
            b.iter(|| black_box(mutual_information_distribution(black_box(&d))))
        });
    }
    for k in [64, 4096, 65536] {
        let mut x = Vec::new();
        let mut y = Vec::new();
        for (a, b, n) in [(0, 0, k + 1), (0, 1, k - 1), (1, 0, k - 1), (1, 1, k + 1)] {
            x.extend(std::iter::repeat_n(a, n));
            y.extend(std::iter::repeat_n(b, n));
        }
        let d = joint_probabilities(&x, &y).unwrap();
        group.bench_function(BenchmarkId::new("near_independent", 4 * k), |b| {
            b.iter(|| black_box(mutual_information_distribution(black_box(&d))))
        });
    }
    let x: Vec<_> = (0..65536).map(|i| i as u8).collect();
    let d = joint_probabilities(&x, &x).unwrap();
    group.bench_function(BenchmarkId::new("diagonal", 65536), |b| {
        b.iter(|| black_box(mutual_information_distribution(black_box(&d))))
    });
    group.finish();
    let mut group = c.benchmark_group("mutual_edges");
    group.bench_function("empty", |b| {
        b.iter(|| black_box(mutual_information(black_box(b""), black_box(b"")).unwrap()))
    });
    group.bench_function("unequal", |b| {
        b.iter(|| black_box(mutual_information(black_box(b"AB"), black_box(b"X")).unwrap_err()))
    });
    group.finish();
}
criterion_group! {name=benches;config=Criterion::default().sample_size(10).warm_up_time(Duration::from_millis(100)).measurement_time(Duration::from_millis(500));targets=benchmarks}
criterion_main!(benches);
