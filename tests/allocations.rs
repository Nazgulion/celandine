//! Isolated, single-threaded allocation check; the Rust test harness is disabled.

use celandine::distribution::{ByteHistogram, Distribution};
use celandine::entropy::{
    hartley_entropy, hartley_entropy_distribution, renyi_entropy, renyi_entropy_distribution,
    shannon, shannon_distribution,
};
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
        assert!(black_box(renyi_entropy(black_box(&data), f64::NAN)).is_err());
        assert!(black_box(renyi_entropy_distribution(black_box(&d), -1.0)).is_err());
        let after = ALLOCATIONS.load(Ordering::Relaxed);
        assert_eq!(after - before, 0, "allocations for {size} bytes");
    }
    println!(
        "Allocation checks passed: zero allocations for histogram, distribution, Shannon, Hartley, and Rényi (0 B–10 MiB)."
    );
}
