//! Bounded profiles, short full profiles, and lazy access costs.
#![allow(missing_docs)]
use celandine::entropy::block_entropy_profile;
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
    let mut group = c.benchmark_group("block_profile_prefix");
    group.sampling_mode(SamplingMode::Flat);
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        for size in [
            16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760, 104857600,
        ] {
            let data = input(size, shape);
            let limits: &[usize] = if [16, 256, 4096, 65536].contains(&size) {
                &[1, 2, 8]
            } else if size > 65536 && matches!(shape, "constant" | "uniform") {
                &[1, 2]
            } else {
                &[1]
            };
            for &limit in limits {
                group.throughput(Throughput::Bytes(size as u64));
                group.bench_function(BenchmarkId::new(format!("{shape}_max{limit}"), size), |b| {
                    b.iter(|| {
                        black_box(
                            block_entropy_profile(black_box(&data), black_box(limit)).sum::<f64>(),
                        )
                    })
                });
            }
        }
    }
    group.finish();
    let mut group = c.benchmark_group("block_profile_full");
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        for size in [16, 64, 256] {
            let data = input(size, shape);
            group.throughput(Throughput::Bytes(size as u64));
            group.bench_function(BenchmarkId::new(shape, size), |b| {
                b.iter(|| {
                    black_box(
                        block_entropy_profile(black_box(&data), black_box(usize::MAX)).sum::<f64>(),
                    )
                })
            });
        }
    }
    group.finish();
    let mut group = c.benchmark_group("block_profile_access");
    for size in [16, 65536, 104857600] {
        let data = input(size, "uniform");
        for operation in [
            "construct",
            "count",
            "last",
            "nth",
            "nth_back",
            "zero_limit",
        ] {
            group.bench_function(BenchmarkId::new(operation, size), |b| {
                b.iter(|| {
                    let p = block_entropy_profile(black_box(&data), black_box(usize::MAX));
                    match operation {
                        "construct" => {
                            let _ = black_box(p);
                        }
                        "count" => {
                            black_box(p.clone().count());
                        }
                        "last" => {
                            black_box(p.last());
                        }
                        "nth" => {
                            black_box(p.clone().nth(black_box(size - 1)));
                        }
                        "nth_back" => {
                            black_box(p.clone().nth_back(black_box(size - 1)));
                        }
                        _ => {
                            black_box(
                                block_entropy_profile(black_box(&data), black_box(0)).sum::<f64>(),
                            );
                        }
                    }
                })
            });
        }
    }
    group.finish();
}
criterion_group! {name=benches;config=Criterion::default().sample_size(10).warm_up_time(Duration::from_millis(100)).measurement_time(Duration::from_millis(500));targets=benchmarks}
criterion_main!(benches);
