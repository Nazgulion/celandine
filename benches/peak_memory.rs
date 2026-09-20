//! Isolated development memory probe; run with scripts/measure_memory.py.
//! Reports requested peak heap bytes separately from process RSS collected by GNU time.

#[path = "support/peak_allocator.rs"]
mod peak_allocator;

use celandine::distribution::{ByteHistogram, Distribution, ngram_counts, ngram_probabilities};
use celandine::entropy::*;
use celandine::transforms::ngrams;
use std::hint::black_box;

#[global_allocator]
static ALLOCATOR: peak_allocator::PeakAllocator = peak_allocator::PeakAllocator;

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

fn entropy(data: &[u8]) -> usize {
    let h = ByteHistogram::from_bytes(black_box(data));
    let d = Distribution::from_counts(ByteHistogram::try_from_counts(*h.counts()).unwrap());
    black_box(shannon(black_box(data)));
    black_box(hartley_entropy(black_box(data)));
    black_box(collision_entropy(black_box(data)));
    black_box(min_entropy(black_box(data)));
    black_box(shannon_distribution(black_box(&d)));
    black_box(hartley_entropy_distribution(black_box(&d)));
    black_box(collision_entropy_distribution(black_box(&d)));
    black_box(min_entropy_distribution(black_box(&d)));
    for order in [
        0.0,
        0.5,
        1.0_f64.next_down(),
        1.0,
        1.0_f64.next_up(),
        2.0,
        f64::MAX,
    ] {
        black_box(renyi_entropy(black_box(data), black_box(order))).unwrap();
        black_box(tsallis_entropy(black_box(data), black_box(order))).unwrap();
        black_box(renyi_entropy_distribution(black_box(&d), black_box(order))).unwrap();
        black_box(tsallis_entropy_distribution(
            black_box(&d),
            black_box(order),
        ))
        .unwrap();
    }
    for base in [1.0_f64.next_up(), 2.0, std::f64::consts::E, 10.0, f64::MAX] {
        black_box(shannon_with_base(black_box(data), black_box(base))).unwrap();
        black_box(shannon_distribution_with_base(
            black_box(&d),
            black_box(base),
        ))
        .unwrap();
    }
    d.support_size()
}

fn run(operation: &str, data: &[u8], n: usize) -> (usize, usize, usize) {
    if operation == "reuse" {
        let d = ngram_probabilities(data, n).unwrap();
        return peak_allocator::measure(|| {
            for pair in black_box(&d).probabilities() {
                black_box(pair);
            }
            d.support_size()
        });
    }
    peak_allocator::measure(|| match operation {
        "input" => {
            black_box(data);
            0
        }
        "entropy" => entropy(data),
        "extract" => {
            for block in ngrams(black_box(data), black_box(n)).unwrap() {
                black_box(block);
            }
            0
        }
        "counts" => {
            let counts = ngram_counts(black_box(data), black_box(n)).unwrap();
            black_box(&counts);
            counts.support_size()
        }
        "probabilities" => {
            let d = ngram_probabilities(black_box(data), black_box(n)).unwrap();
            black_box(&d);
            d.support_size()
        }
        _ => unreachable!(),
    })
}

fn main() {
    let args: Vec<_> = std::env::args()
        .skip(1)
        .filter(|arg| arg != "--bench")
        .collect();
    if args.is_empty() || args == ["--self-check"] {
        peak_allocator::self_check();
        let (support, peak, calls) = run("counts", b"ABABA", 2);
        assert_eq!(support, 2);
        assert!(peak > 0 && calls > 0);
        assert_eq!(run("extract", b"ABABA", 2).1, 0);
        assert_eq!(run("reuse", b"ABABA", 2).1, 0);
        println!(
            "Peak-memory probe self-check passed (allocate, zero, grow, shrink, free, reset, n-grams)."
        );
        return;
    }
    assert_eq!(
        args.len(),
        4,
        "expected: operation shape input_bytes ngram_length"
    );
    let operation = &args[0];
    let shape = &args[1];
    assert!(matches!(
        operation.as_str(),
        "input" | "entropy" | "extract" | "counts" | "probabilities" | "reuse"
    ));
    assert!(matches!(
        shape.as_str(),
        "constant" | "uniform" | "skewed" | "mixed"
    ));
    let size: usize = args[2].parse().unwrap();
    let n: usize = args[3].parse().unwrap();
    assert!(size <= 100 * 1024 * 1024 && n > 0);
    let mut data = input(size, shape);
    // Force writes to each 4 KiB page even for zero-filled buffers, then read all
    // bytes. Input RSS must not be hidden by lazily mapped shared zero pages.
    for byte in data.iter_mut().step_by(4096) {
        *byte = black_box(*byte);
    }
    let checksum = data
        .iter()
        .fold(0u64, |sum, &byte| sum.wrapping_add(u64::from(byte)));
    black_box(checksum);
    let (support, peak, calls) = run(operation, black_box(&data), n);
    println!("{operation},{shape},{size},{n},{support},{peak},{calls},{checksum}");
}
