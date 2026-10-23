//! Exhaustive LZ76 counts, with bounded diverse inputs for the scalar baseline.
#![allow(missing_docs)]
use celandine::complexity::lz76_complexity;
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
    let mut group = c.benchmark_group("lz76_bytes");
    group.sampling_mode(SamplingMode::Flat);
    for shape in ["constant", "uniform", "skewed", "mixed"] {
        for size in [
            16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760, 104857600,
        ] {
            // Scalar baseline is quadratic in the worst case. Bound diverse
            // workloads; large short-period inputs exercise the early suffix exit.
            if shape == "mixed" && size > 65536 {
                continue;
            }
            let data = input(size, shape);
            group.throughput(Throughput::Bytes(size as u64));
            group.bench_function(BenchmarkId::new(shape, size), |b| {
                b.iter(|| black_box(lz76_complexity(black_box(&data))))
            });
        }
    }
    group.finish();
}
criterion_group! { name = benches; config = Criterion::default().sample_size(10).warm_up_time(Duration::from_millis(100)).measurement_time(Duration::from_millis(500)); targets = benchmarks }
criterion_main!(benches);
