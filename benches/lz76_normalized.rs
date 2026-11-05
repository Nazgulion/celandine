//! Explicit LZ76 scales, summary construction, and allocation-free cached queries.
#![allow(missing_docs)]
use celandine::complexity::{Lz76Summary, lz76_complexity_asymptotic, lz76_phrase_density};
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
    for operation in ["density", "asymptotic", "summary", "reuse"] {
        let mut group = c.benchmark_group(format!("lz76_normalized_{operation}"));
        group.sampling_mode(SamplingMode::Flat);
        for shape in ["constant", "uniform", "skewed", "mixed"] {
            for size in [16, 4096, 65536, 10485760, 104857600] {
                if shape == "mixed" && size > 65536 {
                    continue;
                }
                let data = input(size, shape);
                let summary = Lz76Summary::from_bytes(&data);
                // Cached queries do not read input: no byte-throughput claim.
                if operation != "reuse" {
                    group.throughput(Throughput::Bytes(size as u64));
                }
                group.bench_function(BenchmarkId::new(shape, size), |b| {
                    b.iter(|| match operation {
                        "density" => {
                            black_box(lz76_phrase_density(black_box(&data)));
                        }
                        "asymptotic" => {
                            black_box(
                                lz76_complexity_asymptotic(black_box(&data), black_box(256))
                                    .unwrap(),
                            );
                        }
                        "summary" => {
                            black_box(Lz76Summary::from_bytes(black_box(&data)));
                        }
                        "reuse" => {
                            let s = black_box(&summary);
                            black_box((s.phrase_density(), s.asymptotic(black_box(256)).unwrap()));
                        }
                        _ => unreachable!(),
                    })
                });
            }
        }
        group.finish();
    }
}

criterion_group! { name = benches; config = Criterion::default().sample_size(10).warm_up_time(Duration::from_millis(100)).measurement_time(Duration::from_millis(500)); targets = benchmarks }
criterion_main!(benches);
