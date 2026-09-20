//! Isolated, single-threaded allocation check; the Rust test harness is disabled.

use celandine::distribution::{ByteHistogram, Distribution};
use celandine::distribution::{NgramDistribution, ngram_counts, ngram_probabilities};
use celandine::entropy::{
    collision_entropy, collision_entropy_distribution, hartley_entropy,
    hartley_entropy_distribution, min_entropy, min_entropy_distribution, renyi_entropy,
    renyi_entropy_distribution, shannon, shannon_distribution, tsallis_entropy,
    tsallis_entropy_distribution,
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
    for size in [
        0, 1, 16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760,
    ] {
        let data: Vec<u8> = (0..size).map(|i| i as u8).collect();
        let before = ALLOCATIONS.load(Ordering::Relaxed);
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
        black_box(d.probabilities().sum::<f64>());
        black_box(d.probability(255));
        black_box(shannon_distribution(black_box(&d)));
        black_box(shannon(black_box(&data)));
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
    println!(
        "Allocation checks passed: existing byte metrics and n-gram extraction (0 B–10 MiB), invalid/oversized n-grams, and count reuse/queries."
    );
}
