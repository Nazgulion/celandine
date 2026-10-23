//! Fresh complete zlib streams; count and discard output.
#![allow(missing_docs)]
use celandine::compression::{Zlib, compressed_length, compression_ratio};
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
    for ratio in [false, true] {
        let mut group = c.benchmark_group(if ratio {
            "compression_ratio_zlib"
        } else {
            "compression_length_zlib"
        });
        group.sampling_mode(SamplingMode::Flat);
        for shape in ["constant", "uniform", "skewed", "mixed"] {
            for size in [
                16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760, 104857600,
            ] {
                let data = input(size, shape);
                group.throughput(Throughput::Bytes(size as u64));
                group.bench_function(BenchmarkId::new(shape, size), |b| {
                    if ratio {
                        b.iter(|| black_box(compression_ratio(black_box(&data), &Zlib).unwrap()));
                    } else {
                        b.iter(|| black_box(compressed_length(black_box(&data), &Zlib).unwrap()));
                    }
                });
            }
        }
        group.finish();
    }
}
criterion_group! { name = benches; config = Criterion::default().sample_size(10).warm_up_time(Duration::from_millis(100)).measurement_time(Duration::from_millis(500)); targets = benchmarks }
criterion_main!(benches);
