//! Construction, entropy and reuse of aligned byte laws.
#![allow(missing_docs)]
#[path = "support/joint_input.rs"]
mod joint_input;
use celandine::{
    distribution::{joint_counts, joint_probabilities},
    entropy::{joint_entropy, joint_entropy_distribution},
};
use criterion::{
    BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};
use std::{hint::black_box, time::Duration};
fn benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("joint_bytes");
    group.sampling_mode(SamplingMode::Flat);
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        for size in [
            16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760, 104857600,
        ] {
            let (x, y) = joint_input::input(size, shape);
            group.throughput(Throughput::Bytes(2 * size as u64));
            group.bench_function(BenchmarkId::new(shape, size), |b| {
                b.iter(|| black_box(joint_entropy(black_box(&x), black_box(&y)).unwrap()))
            });
        }
    }
    group.finish();
    let mut group = c.benchmark_group("joint_counts");
    group.sampling_mode(SamplingMode::Flat);
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        for size in [16, 65536] {
            let (x, y) = joint_input::input(size, shape);
            group.throughput(Throughput::Bytes(2 * size as u64));
            group.bench_function(BenchmarkId::new(shape, size), |b| {
                b.iter(|| black_box(joint_counts(black_box(&x), black_box(&y)).unwrap()))
            });
        }
    }
    group.finish();
    let mut group = c.benchmark_group("joint_reuse");
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        let (x, y) = joint_input::input(65536, shape);
        let d = joint_probabilities(&x, &y).unwrap();
        group.bench_function(BenchmarkId::new(shape, 65536), |b| {
            b.iter(|| black_box(joint_entropy_distribution(black_box(&d))))
        });
        group.bench_function(BenchmarkId::new(format!("{shape}_marginals"), 65536), |b| {
            b.iter(|| {
                black_box(black_box(&d).marginal_x());
                black_box(black_box(&d).marginal_y());
            })
        });
    }
    group.finish();
    let mut group = c.benchmark_group("joint_edges");
    group.bench_function("empty", |b| {
        b.iter(|| black_box(joint_entropy(black_box(b""), black_box(b"")).unwrap()))
    });
    group.bench_function("unequal", |b| {
        b.iter(|| black_box(joint_entropy(black_box(b"AB"), black_box(b"X")).unwrap_err()))
    });
    group.finish();
}
criterion_group! {name=benches;config=Criterion::default().sample_size(10).warm_up_time(Duration::from_millis(100)).measurement_time(Duration::from_millis(500));targets=benchmarks}
criterion_main!(benches);
