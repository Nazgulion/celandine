//! Scalar counts, complete short profiles, and bounded profile prefixes.
#![allow(missing_docs)]
use celandine::complexity::{subword_complexity, subword_complexity_profile};
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
    for n in [1, 2, 8] {
        let mut group = c.benchmark_group(format!("subword_n{n}"));
        group.sampling_mode(SamplingMode::Flat);
        for shape in ["constant", "uniform", "skewed", "mixed"] {
            for size in [
                16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760, 104857600,
            ] {
                if n != 1
                    && !([256, 4096, 65536].contains(&size)
                        || (n == 2 && size >= 10485760 && matches!(shape, "constant" | "uniform")))
                {
                    continue;
                }
                let data = input(size, shape);
                group.throughput(Throughput::Bytes(size as u64));
                group.bench_function(BenchmarkId::new(shape, size), |b| {
                    b.iter(|| black_box(subword_complexity(black_box(&data), black_box(n))))
                });
            }
        }
        group.finish();
    }
    for (name, sizes, limit) in [
        ("subword_profile", &[16, 64, 256, 1024][..], usize::MAX),
        ("subword_prefix_n0_to_n8", &[65536][..], 9),
    ] {
        let mut group = c.benchmark_group(name);
        group.sampling_mode(SamplingMode::Flat);
        for shape in ["constant", "uniform", "skewed", "mixed"] {
            for &size in sizes {
                let data = input(size, shape);
                // Fold consumes every requested value, without allocating an output Vec.
                group.bench_function(BenchmarkId::new(shape, size), |b| {
                    b.iter(|| {
                        black_box(
                            subword_complexity_profile(black_box(&data))
                                .take(black_box(limit))
                                .fold(0usize, usize::wrapping_add),
                        )
                    })
                });
            }
        }
        group.finish();
    }
}
criterion_group! { name = benches; config = Criterion::default().sample_size(10).warm_up_time(Duration::from_millis(100)).measurement_time(Duration::from_millis(500)); targets = benchmarks }
criterion_main!(benches);
