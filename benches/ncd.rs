//! Ordered NCD of each shaped input and its reversal.
#![allow(missing_docs)]
use celandine::compression::{
    CompressionDistanceComponents as Parts, Zlib, normalized_compression_distance,
};
use criterion::{
    BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};
use std::hint::black_box;
use std::time::Duration;

fn input(size: usize, shape: &str) -> Vec<u8> {
    let mut state = 0x1234_5678_9abc_def0u64;
    (0..size)
        .map(|i| match shape {
            "constant" => 0,
            "uniform" => i as u8,
            "skewed" => u8::from(i % 100 == 0),
            "mixed" => {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                (state >> 56) as u8
            }
            _ => unreachable!(),
        })
        .collect()
}

fn benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("ncd_zlib");
    group.sampling_mode(SamplingMode::Flat);
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        for size in [
            16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760, 104857600,
        ] {
            let data = input(size, shape);
            let other: Vec<_> = data.iter().rev().copied().collect();
            group.throughput(Throughput::Bytes(2 * size as u64));
            group.bench_function(BenchmarkId::new(shape, size), |b| {
                b.iter(|| {
                    black_box(
                        normalized_compression_distance(black_box(&data), black_box(&other), &Zlib)
                            .unwrap(),
                    )
                })
            });
        }
    }
    group.finish();
    let mut arithmetic = c.benchmark_group("ncd_components");
    for (name, parts) in [
        ("worked", Parts { x: 3, y: 3, xy: 5 }),
        (
            "negative",
            Parts {
                x: 10,
                y: 10,
                xy: 5,
            },
        ),
        ("zero_denominator", Parts { x: 0, y: 0, xy: 1 }),
        (
            "unit_gap",
            Parts {
                x: usize::MAX,
                y: usize::MAX,
                xy: usize::MAX - 1,
            },
        ),
        (
            "above_one",
            Parts {
                x: 1,
                y: 1,
                xy: usize::MAX,
            },
        ),
    ] {
        arithmetic.bench_function(name, |b| {
            b.iter(|| black_box(black_box(parts).normalized_distance()))
        });
    }
    arithmetic.finish();
}
criterion_group! { name = benches; config = Criterion::default().sample_size(10).warm_up_time(Duration::from_millis(100)).measurement_time(Duration::from_millis(500)); targets = benchmarks }
criterion_main!(benches);
