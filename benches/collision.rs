//! Reproducible byte-path and reusable-distribution performance baselines.

// Criterion's entry-point macro generates an undocumented public function.
#![allow(missing_docs)]

use celandine::distribution::Distribution;
use celandine::entropy::{collision_entropy, collision_entropy_distribution};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;

fn input(size: usize, shape: &str) -> Vec<u8> {
    let mut state = 0x1234_5678_9abc_def0_u64;
    (0..size)
        .map(|i| match shape {
            "constant" => 0,
            "uniform" => i as u8,
            "skewed" => u8::from(i % 100 == 0),
            "mixed" => {
                // Fixed-seed xorshift64: reproducible input, not a source model.
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
    let mut group = c.benchmark_group("collision_bytes");
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        for size in [
            16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760, 104857600,
        ] {
            let data = input(size, shape);
            group.throughput(Throughput::Bytes(size as u64));
            group.bench_with_input(BenchmarkId::new(shape, size), &data, |b, data| {
                b.iter(|| black_box(collision_entropy(black_box(data))));
            });
        }
    }
    group.finish();

    let mut group = c.benchmark_group("collision_distribution");
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        let distribution = Distribution::from_bytes(&input(65536, shape));
        group.bench_function(shape, |b| {
            b.iter(|| black_box(collision_entropy_distribution(black_box(&distribution))));
        });
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(30)
        .warm_up_time(Duration::from_millis(300))
        .measurement_time(Duration::from_secs(1));
    targets = benchmarks
}
criterion_main!(benches);
