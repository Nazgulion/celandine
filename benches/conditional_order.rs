//! Conditional entropy by order: construction, reuse and bounded large-input tiers.
#![allow(missing_docs)]
use celandine::{
    distribution::ngram_probabilities,
    entropy::{conditional_entropy_order, conditional_entropy_order_distribution},
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
    let mut group = c.benchmark_group("conditional_order_bytes");
    group.sampling_mode(SamplingMode::Flat);
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        for size in [
            16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760, 104857600,
        ] {
            let data = input(size, shape);
            let orders: &[usize] = if [16, 256, 4096, 65536].contains(&size) {
                &[0, 1, 7]
            } else if size > 65536 && matches!(shape, "constant" | "uniform") {
                &[0, 1]
            } else {
                &[0]
            };
            for &n in orders {
                group.throughput(Throughput::Bytes(size as u64));
                group.bench_function(BenchmarkId::new(format!("{shape}_k{n}"), size), |b| {
                    b.iter(|| black_box(conditional_entropy_order(black_box(&data), black_box(n))))
                });
            }
        }
    }
    for shape in ["constant", "mixed"] {
        let data = input(65536, shape);
        for n in [127, 4095] {
            group.throughput(Throughput::Bytes(data.len() as u64));
            group.bench_function(BenchmarkId::new(format!("{shape}_k{n}"), data.len()), |b| {
                b.iter(|| black_box(conditional_entropy_order(black_box(&data), black_box(n))))
            });
        }
    }
    group.finish();
    let mut group = c.benchmark_group("conditional_entropy_order_distribution");
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        let data = input(65536, shape);
        for order in [1, 7] {
            let d = ngram_probabilities(&data, order + 1).unwrap();
            group.bench_function(format!("{shape}_k{order}"), |b| {
                b.iter(|| black_box(conditional_entropy_order_distribution(black_box(&d))))
            });
        }
    }
    group.finish();
    let mut group = c.benchmark_group("conditional_order_edges");
    let data = input(65536, "mixed");
    for (label, n) in [
        ("one_observation", data.len() - 1),
        ("oversized", usize::MAX),
    ] {
        group.bench_function(label, |b| {
            b.iter(|| black_box(conditional_entropy_order(black_box(&data), black_box(n))))
        });
    }
    group.finish();
}
criterion_group! {name=benches;config=Criterion::default().sample_size(10).warm_up_time(Duration::from_millis(100)).measurement_time(Duration::from_millis(500));targets=benchmarks}
criterion_main!(benches);
