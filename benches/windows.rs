//! Borrowed window traversal and bounded random-access iterator operations.
#![allow(missing_docs)]
use celandine::transforms::windows;
use criterion::{
    BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};
use std::{hint::black_box, time::Duration};
fn benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("windows_extract");
    group.sampling_mode(SamplingMode::Flat);
    for size in [
        16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760, 104857600,
    ] {
        let data: Vec<_> = (0..size).map(|i| i as u8).collect();
        for (label, step) in [
            ("overlap", 1),
            ("half_overlap", 4),
            ("adjacent", 8),
            ("gaps", 16),
        ] {
            group.throughput(Throughput::Bytes(size as u64));
            group.bench_function(BenchmarkId::new(label, size), |b| {
                b.iter(|| {
                    for block in windows(black_box(&data), black_box(8), black_box(step)).unwrap() {
                        black_box(block);
                    }
                })
            });
        }
    }
    group.finish();
    let mut group = c.benchmark_group("windows_access");
    for size in [16, 65536, 104857600] {
        let data = vec![0; size];
        for method in ["nth_max", "nth_back_max", "last", "count"] {
            group.bench_function(BenchmarkId::new(method, size), |b| {
                b.iter(|| {
                    let mut iter = windows(black_box(&data), black_box(3), black_box(2)).unwrap();
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
                        _ => {
                            black_box(iter.count());
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
