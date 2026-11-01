//! Overlapping-block entropy: construction, reuse and bounded large-input tiers.
#![allow(missing_docs)]
use celandine::{
    distribution::ngram_probabilities,
    entropy::{block_entropy, block_entropy_distribution},
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
    let mut group = c.benchmark_group("block_entropy_bytes");
    group.sampling_mode(SamplingMode::Flat);
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        for size in [
            16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760, 104857600,
        ] {
            let data = input(size, shape);
            let orders: &[usize] = if [16, 256, 4096, 65536].contains(&size) {
                &[1, 2, 8]
            } else if size > 65536 && matches!(shape, "constant" | "uniform") {
                &[1, 2]
            } else {
                &[1]
            };
            for &n in orders {
                group.throughput(Throughput::Bytes(size as u64));
                group.bench_function(BenchmarkId::new(format!("{shape}_n{n}"), size), |b| {
                    b.iter(|| black_box(block_entropy(black_box(&data), black_box(n)).unwrap()))
                });
            }
        }
    }
    for shape in ["constant", "mixed"] {
        let data = input(65536, shape);
        for n in [128, 4096] {
            group.throughput(Throughput::Bytes(data.len() as u64));
            group.bench_function(BenchmarkId::new(format!("{shape}_n{n}"), data.len()), |b| {
                b.iter(|| black_box(block_entropy(black_box(&data), black_box(n)).unwrap()))
            });
        }
    }
    group.finish();
    let mut group = c.benchmark_group("block_entropy_distribution");
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        let data = input(65536, shape);
        let d = ngram_probabilities(&data, 8).unwrap();
        group.bench_function(shape, |b| {
            b.iter(|| black_box(block_entropy_distribution(black_box(&d))))
        });
    }
    group.finish();
    let mut group = c.benchmark_group("block_entropy_edges");
    let data = input(65536, "mixed");
    for (label, n) in [("whole", data.len()), ("oversized", usize::MAX)] {
        group.bench_function(label, |b| {
            b.iter(|| black_box(block_entropy(black_box(&data), black_box(n)).unwrap()))
        });
    }
    group.finish();
}
criterion_group! {name=benches;config=Criterion::default().sample_size(10).warm_up_time(Duration::from_millis(100)).measurement_time(Duration::from_millis(500));targets=benchmarks}
criterion_main!(benches);
