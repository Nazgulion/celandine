//! Borrowed block traversal and bounded random-access iterator operations.
#![allow(missing_docs)]
use celandine::transforms::blocks;
use criterion::{
    BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};
use std::{hint::black_box, time::Duration};
fn benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("blocks_extract");
    group.sampling_mode(SamplingMode::Flat);
    for size in [
        16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760, 104857600,
    ] {
        let data: Vec<_> = (0..size).map(|i| i as u8).collect();
        for (label, width) in [
            ("width_1", 1),
            ("width_3", 3),
            ("width_8", 8),
            ("width_64", 64),
        ] {
            group.throughput(Throughput::Bytes(size as u64));
            group.bench_function(BenchmarkId::new(label, size), |b| {
                b.iter(|| {
                    for block in blocks(black_box(&data), black_box(width)).unwrap() {
                        black_box(block);
                    }
                })
            });
        }
    }
    group.finish();
    let mut group = c.benchmark_group("blocks_access");
    for size in [16, 65536, 104857600] {
        let data = vec![0; size];
        for method in ["nth_max", "nth_back_max", "last", "count", "remainder"] {
            group.bench_function(BenchmarkId::new(method, size), |b| {
                b.iter(|| {
                    let mut iter = blocks(black_box(&data), black_box(3)).unwrap();
                    match method {
                        "nth_max" => {
                            black_box(iter.nth(black_box(usize::MAX)));
                        }
                        "nth_back_max" => {
                            black_box(iter.nth_back(black_box(usize::MAX)));
                        }
                        "last" => {
                            black_box(iter.last());
                        }
                        "count" => {
                            black_box(iter.count());
                        }
                        _ => {
                            black_box(iter.remainder());
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
