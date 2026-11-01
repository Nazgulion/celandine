//! Explicit K=256 byte scans and reusable statistical-complexity laws.
#![allow(missing_docs)]
use celandine::{
    complexity::{
        normalized_jensen_shannon_complexity as complexity,
        normalized_jensen_shannon_complexity_distribution as reuse,
    },
    distribution::{ByteHistogram, Distribution},
};
use criterion::{
    BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};
use std::{hint::black_box, time::Duration};
fn input(size: usize, shape: &str) -> Vec<u8> {
    let mut state = 0x1234_5678_9abc_def0u64;
    (0..size)
        .map(|i| match shape {
            "constant" => 0,
            "uniform" => i as u8,
            "skewed" => u8::from(i % 100 == 0),
            _ => {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                (state >> 56) as u8
            }
        })
        .collect()
}
fn benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistical_complexity_bytes");
    group.sampling_mode(SamplingMode::Flat);
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        for size in [
            16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760, 104857600,
        ] {
            let data = input(size, shape);
            group.throughput(Throughput::Bytes(size as u64));
            group.bench_function(BenchmarkId::new(shape, size), |b| {
                b.iter(|| black_box(complexity(black_box(&data), black_box(256)).unwrap()))
            });
        }
    }
    group.finish();
    let mut group = c.benchmark_group("statistical_complexity_distribution");
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        let d = Distribution::from_bytes(&input(65536, shape));
        group.bench_function(shape, |b| {
            b.iter(|| black_box(reuse(black_box(&d), black_box(256)).unwrap()))
        });
    }
    for (label, counts, k) in [
        ("rare", [usize::MAX - 1, 1], 2),
        ("near_uniform", [usize::MAX / 2, usize::MAX / 2 + 1], 2),
        ("worked", [3, 1], 2),
        ("partial", [1, 1], 4),
    ] {
        let mut table = [0; 256];
        table[..2].copy_from_slice(&counts);
        let d = Distribution::from_counts(ByteHistogram::try_from_counts(table).unwrap());
        group.bench_function(label, |b| {
            b.iter(|| black_box(reuse(black_box(&d), black_box(k)).unwrap()))
        });
    }
    group.finish();
}
criterion_group! {name=benches;config=Criterion::default().sample_size(10).warm_up_time(Duration::from_millis(100)).measurement_time(Duration::from_millis(500));targets=benchmarks}
criterion_main!(benches);
