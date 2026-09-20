//! Explicit-base byte and distribution timings with existing bit-API controls.
#![allow(missing_docs)]

use celandine::distribution::Distribution;
use celandine::entropy::{
    shannon, shannon_distribution, shannon_distribution_with_base, shannon_with_base,
};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::{hint::black_box, time::Duration};

fn input(size: usize, shape: &str) -> Vec<u8> {
    let mut state = 0x1234_5678_9abc_def0_u64;
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
    let mut group = c.benchmark_group("shannon_base_bytes");
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        for size in [16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760] {
            let data = input(size, shape);
            group.throughput(Throughput::Bytes(size as u64));
            group.bench_with_input(
                BenchmarkId::new(format!("{shape}_original"), size),
                &data,
                |b, data| {
                    b.iter(|| black_box(shannon(black_box(data))));
                },
            );
            for (name, base) in [
                ("bits", 2.0),
                ("nats", std::f64::consts::E),
                ("decimal", 10.0),
            ] {
                group.bench_with_input(
                    BenchmarkId::new(format!("{shape}_{name}"), size),
                    &data,
                    |b, data| {
                        b.iter(|| {
                            black_box(shannon_with_base(black_box(data), black_box(base)).unwrap())
                        });
                    },
                );
            }
        }
    }
    group.finish();

    let mut group = c.benchmark_group("shannon_base_distribution");
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        let d = Distribution::from_bytes(&input(65536, shape));
        group.bench_function(format!("{shape}_original"), |b| {
            b.iter(|| black_box(shannon_distribution(black_box(&d))));
        });
        for (name, base) in [
            ("near_one", 1.0_f64.next_up()),
            ("bits", 2.0),
            ("nats", std::f64::consts::E),
            ("decimal", 10.0),
            ("maximum", f64::MAX),
        ] {
            group.bench_function(format!("{shape}_{name}"), |b| {
                b.iter(|| {
                    black_box(
                        shannon_distribution_with_base(black_box(&d), black_box(base)).unwrap(),
                    )
                });
            });
        }
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_millis(100))
        .measurement_time(Duration::from_millis(500));
    targets = benchmarks
}
criterion_main!(benches);
