//! Requested live heap bytes for one single-threaded, fully contained operation.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};

pub struct PeakAllocator;
static ENABLED: AtomicBool = AtomicBool::new(false);
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static CALLS: AtomicUsize = AtomicUsize::new(0);

fn add(size: usize) {
    let live = LIVE.fetch_add(size, Relaxed).wrapping_add(size);
    PEAK.fetch_max(live, Relaxed);
}

// SAFETY: All original allocation contracts are forwarded unchanged to System.
// Accounting uses nonallocating atomics only; it never changes returned pointers.
unsafe impl GlobalAlloc for PeakAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: The caller supplied a valid nonzero allocation layout.
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() && ENABLED.load(Relaxed) {
            CALLS.fetch_add(1, Relaxed);
            add(layout.size());
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: The caller supplied a valid nonzero allocation layout.
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() && ENABLED.load(Relaxed) {
            CALLS.fetch_add(1, Relaxed);
            add(layout.size());
        }
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        // SAFETY: Original pointer, layout, and new size satisfy the caller's contract.
        let new_ptr = unsafe { System.realloc(ptr, layout, size) };
        if !new_ptr.is_null() && ENABLED.load(Relaxed) {
            CALLS.fetch_add(1, Relaxed);
            // Model the new logical allocation, not allocator-internal transient storage.
            LIVE.fetch_sub(layout.size(), Relaxed);
            add(size);
        }
        new_ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ENABLED.load(Relaxed) {
            LIVE.fetch_sub(layout.size(), Relaxed);
        }
        // SAFETY: The original pointer and allocation layout are forwarded unchanged.
        unsafe { System.dealloc(ptr, layout) };
    }
}

/// All allocations created within the closure must also be dropped within it.
/// Preexisting allocations must neither be resized nor dropped inside the closure.
/// The executable must remain single-threaded throughout measurement.
pub fn measure<T>(operation: impl FnOnce() -> T) -> (T, usize, usize) {
    assert!(!ENABLED.load(Relaxed));
    LIVE.store(0, Relaxed);
    PEAK.store(0, Relaxed);
    CALLS.store(0, Relaxed);
    ENABLED.store(true, Relaxed);
    let result = operation();
    ENABLED.store(false, Relaxed);
    assert_eq!(
        LIVE.load(Relaxed),
        0,
        "measurement must release its allocations"
    );
    (result, PEAK.load(Relaxed), CALLS.load(Relaxed))
}

pub fn self_check() {
    let (_, peak, calls) = measure(|| {
        let small = Layout::from_size_align(64, 8).unwrap();
        let large = Layout::from_size_align(128, 8).unwrap();
        let tiny = Layout::from_size_align(32, 8).unwrap();
        // SAFETY: Nonzero layouts; each successful pointer is used with its current
        // layout and freed once. No accesses use an old pointer after reallocation.
        unsafe {
            let a = PeakAllocator.alloc(small);
            let b = PeakAllocator.alloc_zeroed(tiny);
            assert!(!a.is_null() && !b.is_null());
            let a = PeakAllocator.realloc(a, small, large.size());
            assert!(!a.is_null());
            assert_eq!(LIVE.load(Relaxed), 160);
            let a = PeakAllocator.realloc(a, large, tiny.size());
            assert!(!a.is_null());
            assert_eq!(LIVE.load(Relaxed), 64);
            PeakAllocator.dealloc(a, tiny);
            PeakAllocator.dealloc(b, tiny);
        }
    });
    assert_eq!((peak, calls), (160, 4));
    assert_eq!(measure(|| ()).1, 0);
}
