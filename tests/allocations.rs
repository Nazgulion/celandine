//! Isolated, single-threaded allocation check; the Rust test harness is disabled.

use celandine::complexity::{lz76_complexity, subword_complexity, subword_complexity_profile};
use celandine::distribution::{ByteHistogram, Distribution};
use celandine::distribution::{NgramDistribution, ngram_counts, ngram_probabilities};
use celandine::divergence::{
    jensen_shannon_distance, jensen_shannon_distance_distribution, jensen_shannon_divergence,
    jensen_shannon_divergence_distribution, total_variation_distance,
    total_variation_distance_distribution,
};
use celandine::entropy::{
    collision_entropy, collision_entropy_distribution, hartley_entropy,
    hartley_entropy_distribution, min_entropy, min_entropy_distribution, renyi_entropy,
    renyi_entropy_distribution, shannon, shannon_distribution, shannon_distribution_with_base,
    shannon_with_base, tsallis_entropy, tsallis_entropy_distribution,
};
use celandine::transforms::ngrams;
use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingAllocator;
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

// SAFETY: Every operation forwards the caller's original allocation contract to
// System. Counting uses only atomics and never allocates or alters the pointers.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: Forwarded unchanged from GlobalAlloc's caller.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: Forwarded unchanged from GlobalAlloc's caller.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: Forwarded unchanged from GlobalAlloc's caller.
        unsafe { System.realloc(ptr, layout, new_size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: Forwarded unchanged from GlobalAlloc's caller.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn main() {
    use celandine::transforms::blocks;
    let block_data = vec![0; 10 * 1024 * 1024];
    for size in [0, 1, 3, 8, block_data.len() + 1, usize::MAX] {
        let before = ALLOCATIONS.load(Ordering::Relaxed);
        if let Ok(iter) = blocks(black_box(&block_data), black_box(size)) {
            black_box(iter.remainder());
            black_box(iter.len());
            black_box(iter.clone().nth(usize::MAX));
            black_box(iter.clone().nth_back(usize::MAX));
            black_box(iter.clone().last());
            black_box(iter.clone().count());
            for block in iter.take(32) {
                black_box(block);
            }
        }
        assert_eq!(ALLOCATIONS.load(Ordering::Relaxed), before);
    }

    use celandine::transforms::windows;
    let window_data = vec![0; 10 * 1024 * 1024];
    for width in [0, 1, 8, window_data.len() + 1, usize::MAX] {
        for step in [0, 1, 4, usize::MAX] {
            let before = ALLOCATIONS.load(Ordering::Relaxed);
            if let Ok(iter) = windows(black_box(&window_data), black_box(width), black_box(step)) {
                black_box(iter.len());
                black_box(iter.clone().nth(usize::MAX));
                black_box(iter.clone().nth_back(usize::MAX));
                black_box(iter.clone().last());
                black_box(iter.clone().count());
                for block in iter.take(32) {
                    black_box(block);
                }
            }
            assert_eq!(ALLOCATIONS.load(Ordering::Relaxed), before);
        }
    }

    use celandine::compression::{
        CompressionDistanceComponents as Parts, Compressor, compression_distance_components,
        normalized_compression_distance,
    };
    struct Literal;
    impl Compressor for Literal {
        type Error = std::convert::Infallible;
        fn compressed_len(&self, data: &[u8]) -> Result<usize, Self::Error> {
            Ok(data.len() + 1)
        }
    }
    for (x, y, allocations) in [
        (b"".as_slice(), b"".as_slice(), 0),
        (b"", b"A", 0),
        (b"A", b"", 0),
        (b"AB", b"CD", 1),
    ] {
        let before = ALLOCATIONS.load(Ordering::Relaxed);
        black_box(normalized_compression_distance(black_box(x), black_box(y), &Literal).unwrap());
        assert_eq!(ALLOCATIONS.load(Ordering::Relaxed) - before, allocations);
        let before = ALLOCATIONS.load(Ordering::Relaxed);
        black_box(compression_distance_components(black_box(x), black_box(y), &Literal).unwrap());
        assert_eq!(ALLOCATIONS.load(Ordering::Relaxed) - before, allocations);
    }
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    for parts in [
        Parts { x: 0, y: 0, xy: 0 },
        Parts { x: 3, y: 3, xy: 5 },
        Parts {
            x: usize::MAX,
            y: usize::MAX,
            xy: usize::MAX - 1,
        },
    ] {
        let _ = black_box(black_box(parts).normalized_distance());
    }
    assert_eq!(ALLOCATIONS.load(Ordering::Relaxed), before);

    // Regression: skipping must not evaluate discarded block lengths. A Map
    // adapter ran the counting closure on skipped entries on Rust 1.90.
    let data = b"ABCDEFGHIJKLMNOP";
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    assert_eq!(
        subword_complexity_profile(black_box(data)).nth(data.len()),
        Some(1)
    );
    assert_eq!(
        ALLOCATIONS.load(Ordering::Relaxed) - before,
        0,
        "profile nth skipping allocated"
    );
    assert_eq!(
        subword_complexity_profile(black_box(data)).nth_back(data.len()),
        Some(1)
    );
    assert_eq!(
        ALLOCATIONS.load(Ordering::Relaxed) - before,
        0,
        "profile nth_back skipping allocated"
    );
    assert_eq!(
        subword_complexity_profile(black_box(data)).nth(usize::MAX),
        None
    );
    assert_eq!(
        ALLOCATIONS.load(Ordering::Relaxed) - before,
        0,
        "exhausted profile skipping allocated"
    );
    for size in [
        0, 1, 16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760,
    ] {
        let data: Vec<u8> = (0..size).map(|i| i as u8).collect();
        let before = ALLOCATIONS.load(Ordering::Relaxed);
        black_box(lz76_complexity(black_box(&data))); // Cycling input, including 10 MiB.
        for n in [0, 1, size, size + 1, usize::MAX] {
            black_box(subword_complexity(black_box(&data), black_box(n)));
        }
        let profile = subword_complexity_profile(black_box(&data));
        black_box(profile.len());
        let _ = black_box(profile.clone());
        black_box(profile.clone().take(2).sum::<usize>());
        black_box(profile.clone().nth(size));
        black_box(profile.clone().nth_back(size));
        black_box(profile.clone().nth(usize::MAX));
        black_box(profile.rev().take(1).sum::<usize>());
        for n in [1, 2, 8, size.max(1), usize::MAX] {
            for block in ngrams(black_box(&data), black_box(n)).unwrap() {
                black_box(block);
            }
        }
        assert!(ngrams(black_box(&data), 0).is_err());
        assert!(ngram_counts(black_box(&data), 0).is_err());
        assert!(ngram_probabilities(black_box(&data), 0).is_err());
        black_box(ngram_counts(black_box(&data), usize::MAX).unwrap());
        black_box(ngram_probabilities(black_box(&data), usize::MAX).unwrap());
        let histogram = ByteHistogram::from_bytes(black_box(&data));
        black_box(histogram.support_size());
        let reconstructed = ByteHistogram::try_from_counts(*histogram.counts()).unwrap();
        let d = Distribution::from_counts(reconstructed);
        let other = Distribution::from_bytes(b"AAAB");
        assert_eq!(
            black_box(total_variation_distance(
                black_box(&data),
                black_box(b"AAAB")
            ))
            .is_ok(),
            size > 0
        );
        assert_eq!(
            black_box(total_variation_distance_distribution(
                black_box(&d),
                black_box(&other)
            ))
            .is_ok(),
            size > 0
        );
        assert!(total_variation_distance(black_box(&data), black_box(b"")).is_err());
        assert!(total_variation_distance_distribution(&d, &Distribution::from_bytes(b"")).is_err());
        assert_eq!(
            black_box(jensen_shannon_divergence(
                black_box(&data),
                black_box(b"AAAB")
            ))
            .is_ok(),
            size > 0
        );
        assert_eq!(
            black_box(jensen_shannon_distance(
                black_box(&data),
                black_box(b"AAAB")
            ))
            .is_ok(),
            size > 0
        );
        assert_eq!(
            black_box(jensen_shannon_divergence_distribution(
                black_box(&d),
                black_box(&other)
            ))
            .is_ok(),
            size > 0
        );
        assert_eq!(
            black_box(jensen_shannon_distance_distribution(
                black_box(&d),
                black_box(&other)
            ))
            .is_ok(),
            size > 0
        );
        let empty = Distribution::from_bytes(b"");
        assert!(jensen_shannon_divergence(black_box(&data), black_box(b"")).is_err());
        assert!(jensen_shannon_distance(black_box(&data), black_box(b"")).is_err());
        assert!(jensen_shannon_divergence_distribution(&d, &empty).is_err());
        assert!(jensen_shannon_distance_distribution(&d, &empty).is_err());
        black_box(d.probabilities().sum::<f64>());
        black_box(d.probability(255));
        black_box(shannon_distribution(black_box(&d)));
        black_box(shannon(black_box(&data)));
        for base in [1.0_f64.next_up(), 2.0, std::f64::consts::E, 10.0, f64::MAX] {
            black_box(shannon_with_base(black_box(&data), black_box(base))).unwrap();
            black_box(shannon_distribution_with_base(
                black_box(&d),
                black_box(base),
            ))
            .unwrap();
        }
        for base in [f64::NAN, f64::INFINITY, -1.0, 0.0, 0.5, 1.0] {
            assert!(black_box(shannon_with_base(black_box(&data), black_box(base))).is_err());
            assert!(
                black_box(shannon_distribution_with_base(
                    black_box(&d),
                    black_box(base)
                ))
                .is_err()
            );
        }
        black_box(hartley_entropy_distribution(black_box(&d)));
        black_box(hartley_entropy(black_box(&data)));
        black_box(collision_entropy_distribution(black_box(&d)));
        black_box(collision_entropy(black_box(&data)));
        black_box(min_entropy_distribution(black_box(&d)));
        black_box(min_entropy(black_box(&data)));
        for alpha in [
            0.0,
            0.5,
            1.0 - 1e-8,
            1.0,
            1.0 + 1e-8,
            2.0,
            f64::MAX,
            f64::INFINITY,
        ] {
            black_box(renyi_entropy_distribution(black_box(&d), black_box(alpha))).unwrap();
            black_box(renyi_entropy(black_box(&data), black_box(alpha))).unwrap();
        }
        for q in [
            0.0,
            0.1,
            0.5,
            1.0_f64.next_down(),
            1.0,
            1.0_f64.next_up(),
            2.0,
            f64::MAX,
        ] {
            black_box(tsallis_entropy_distribution(black_box(&d), black_box(q))).unwrap();
            black_box(tsallis_entropy(black_box(&data), black_box(q))).unwrap();
        }
        for q in [-1.0, f64::NAN, f64::INFINITY] {
            assert!(black_box(tsallis_entropy(black_box(&data), black_box(q))).is_err());
            assert!(black_box(tsallis_entropy_distribution(black_box(&d), black_box(q))).is_err());
        }
        assert!(black_box(renyi_entropy(black_box(&data), f64::NAN)).is_err());
        assert!(black_box(renyi_entropy_distribution(black_box(&d), -1.0)).is_err());
        let after = ALLOCATIONS.load(Ordering::Relaxed);
        assert_eq!(after - before, 0, "allocations for {size} bytes");
    }
    // Construction may allocate; moving counts and reading the table must not.
    for data in [&b""[..], b"A", b"ABABA", b"AAAA", &[0, 255, 128, 0]] {
        for n in [1, 2, 8] {
            let counts = ngram_counts(black_box(data), n).unwrap();
            let before = ALLOCATIONS.load(Ordering::Relaxed);
            let d = NgramDistribution::from_counts(counts);
            black_box(d.ngram_len());
            black_box(d.sample_size());
            black_box(d.support_size());
            black_box(d.is_empty());
            black_box(d.counts().counts().map(|(_, c)| c).sum::<usize>());
            black_box(d.probabilities().map(|(_, p)| p).sum::<f64>());
            black_box(d.probability(black_box(b"AB")));
            assert_eq!(ALLOCATIONS.load(Ordering::Relaxed) - before, 0);
        }
    }
    for data in [&b""[..], b"A", b"AAAA", b"101101011", b"010011101101100"] {
        let before = ALLOCATIONS.load(Ordering::Relaxed);
        black_box(lz76_complexity(black_box(data)));
        assert_eq!(ALLOCATIONS.load(Ordering::Relaxed), before);
    }
    println!(
        "Allocation checks passed: existing byte metrics and n-gram extraction (0 B–10 MiB), invalid/oversized n-grams, count reuse/queries, subword fast paths/lazy skipping, and LZ76."
    );
}
