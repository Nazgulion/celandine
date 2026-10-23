//! Paired byte inputs and reusable-distribution Jensen-Shannon benchmarks.
#![allow(missing_docs)]

use celandine::distribution::{ByteHistogram, Distribution};
use celandine::divergence::{
    EmptyDistribution, jensen_shannon_distance, jensen_shannon_distance_distribution,
    jensen_shannon_divergence, jensen_shannon_divergence_distribution,
};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;

fn input(size: usize, shape: &str, right: bool) -> Vec<u8> {
    let mut state = if right {
        0xfedc_ba98_7654_3210u64
    } else {
        0x1234_5678_9abc_def0u64
    };
    (0..size)
        .map(|i| match shape {
            "equal" => 0,
            "disjoint" => u8::from(right),
            "skewed" => u8::from(i % if right { 2 } else { 100 } == 0),
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

type ByteComparison = fn(&[u8], &[u8]) -> Result<f64, EmptyDistribution>;
type DistributionComparison = fn(&Distribution, &Distribution) -> Result<f64, EmptyDistribution>;

fn metric(c: &mut Criterion, name: &str, compare: ByteComparison, reuse: DistributionComparison) {
    let mut group = c.benchmark_group(format!("{name}_bytes"));
    for shape in ["equal", "disjoint", "skewed", "mixed"] {
        for size in [
            16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760, 104857600,
        ] {
            let p = input(size, shape, false);
            let q = input(size, shape, true);
            // Throughput counts the bytes read from BOTH inputs.
            group.throughput(Throughput::Bytes(2 * size as u64));
            group.bench_function(BenchmarkId::new(shape, size), |b| {
                b.iter(|| black_box(compare(black_box(&p), black_box(&q)).unwrap()));
            });
        }
    }
    group.finish();
    let mut group = c.benchmark_group(format!("{name}_distribution"));
    for shape in ["equal", "disjoint", "skewed", "mixed"] {
        let p = Distribution::from_bytes(&input(65536, shape, false));
        let q = Distribution::from_bytes(&input(65536, shape, true));
        group.bench_function(shape, |b| {
            b.iter(|| black_box(reuse(black_box(&p), black_box(&q)).unwrap()));
        });
    }
    let make = |n| {
        let mut counts = [0; 256];
        counts[0] = n - 1;
        counts[1] = 1;
        Distribution::from_counts(ByteHistogram::try_from_counts(counts).unwrap())
    };
    let p = make(usize::MAX);
    let q = make(usize::MAX - 1);
    group.bench_function("tiny_difference", |b| {
        b.iter(|| black_box(reuse(black_box(&p), black_box(&q)).unwrap()));
    });
    group.finish();
}

fn benchmarks(c: &mut Criterion) {
    metric(
        c,
        "jensen_shannon_divergence",
        jensen_shannon_divergence,
        jensen_shannon_divergence_distribution,
    );
    metric(
        c,
        "jensen_shannon_distance",
        jensen_shannon_distance,
        jensen_shannon_distance_distribution,
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20)
        .warm_up_time(Duration::from_millis(300))
        .measurement_time(Duration::from_secs(1));
    targets = benchmarks
}
criterion_main!(benches);
