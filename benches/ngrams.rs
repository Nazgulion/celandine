//! Baseline extraction, construction, and reused n-gram probability iteration.
#![allow(missing_docs)]

use celandine::distribution::{ngram_counts, ngram_probabilities};
use celandine::transforms::ngrams;
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
    let mut group = c.benchmark_group("ngrams_extract");
    for size in [
        16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760, 104857600,
    ] {
        let data = input(size, "mixed");
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("n4", size), &data, |b, data| {
            b.iter(|| {
                for block in ngrams(black_box(data), black_box(4)).unwrap() {
                    black_box(block);
                }
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("ngrams_counts");
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        for size in [
            16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760, 104857600,
        ] {
            // Large runs bound support to avoid enormous mixed n8 tables.
            if size > 65536 && !matches!(shape, "constant" | "uniform") {
                continue;
            }
            let data = input(size, shape);
            for n in [1, 2, 8] {
                if size > 65536 && n != 2 {
                    continue;
                }
                group.throughput(Throughput::Bytes(size as u64));
                group.bench_with_input(
                    BenchmarkId::new(format!("{shape}_n{n}"), size),
                    &data,
                    |b, data| {
                        b.iter(|| black_box(ngram_counts(black_box(data), black_box(n)).unwrap()));
                    },
                );
            }
        }
    }
    for shape in ["constant", "mixed"] {
        let data = input(65536, shape);
        for n in [128, 4096] {
            group.throughput(Throughput::Bytes(data.len() as u64));
            group.bench_function(format!("{shape}_n{n}/65536"), |b| {
                b.iter(|| black_box(ngram_counts(black_box(&data), black_box(n)).unwrap()));
            });
        }
    }
    group.finish();

    let mut group = c.benchmark_group("ngrams_probabilities");
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        let data = input(65536, shape);
        group.bench_function(format!("{shape}_construct_n4"), |b| {
            b.iter(|| black_box(ngram_probabilities(black_box(&data), black_box(4)).unwrap()));
        });
        let d = ngram_probabilities(&data, 4).unwrap();
        group.bench_function(format!("{shape}_reuse_n4"), |b| {
            b.iter(|| {
                for pair in black_box(&d).probabilities() {
                    black_box(pair);
                }
            });
        });
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
