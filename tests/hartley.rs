//! Canonical, independent-reference, and property tests for Hartley entropy.

use celandine::distribution::{ByteHistogram, Distribution};
use celandine::entropy::{hartley_entropy, hartley_entropy_distribution, shannon_distribution};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use std::collections::BTreeSet;

const TOLERANCE: f64 = 1e-12;

#[test]
fn canonical_entropies() {
    let cases: &[(&[u8], f64)] = &[
        (b"", 0.0),
        (b"A", 0.0),
        (b"AAAA", 0.0),
        (b"AB", 1.0),
        (b"ABAB", 1.0),
        (b"AAAB", 1.0),
        (b"ABCD", 2.0),
        (b"ABCABCABC", 1.584_962_500_721_156),
        (b"0123456789", std::f64::consts::LOG2_10),
        (&[0, 0, 128, 255], 1.584_962_500_721_156),
    ];
    for &(data, expected) in cases {
        let actual = hartley_entropy(data);
        assert!((actual - expected).abs() <= TOLERANCE, "{data:?}: {actual}");
        assert_eq!(
            actual.to_bits(),
            hartley_entropy_distribution(&Distribution::from_bytes(data)).to_bits()
        );
    }
}

#[test]
fn empty_and_constant_results_are_positive_zero() {
    assert_eq!(hartley_entropy(b"").to_bits(), 0.0_f64.to_bits());
    let empty = Distribution::from_counts(ByteHistogram::try_from_counts([0; 256]).unwrap());
    assert_eq!(
        hartley_entropy_distribution(&empty).to_bits(),
        0.0_f64.to_bits()
    );
    for symbol in 0..=255 {
        assert_eq!(hartley_entropy(&[symbol; 32]).to_bits(), 0.0_f64.to_bits());
    }
}

#[test]
fn counts_near_usize_limit_retain_rare_symbols() {
    let mut counts = [0; 256];
    counts[0] = usize::MAX - 255;
    let constant = Distribution::from_counts(ByteHistogram::try_from_counts(counts).unwrap());
    assert_eq!(hartley_entropy_distribution(&constant), 0.0);
    counts[255] = 1;
    let rare = Distribution::from_counts(ByteHistogram::try_from_counts(counts).unwrap());
    assert_eq!(hartley_entropy_distribution(&rare), 1.0);
    counts[1..].fill(1);
    let full = Distribution::from_counts(ByteHistogram::try_from_counts(counts).unwrap());
    assert_eq!(full.sample_size(), usize::MAX);
    assert_eq!(hartley_entropy_distribution(&full), 8.0);
}

#[test]
fn every_support_size_matches_independent_python_reference() {
    let mut rows = 0;
    for line in include_str!("data/hartley.tsv")
        .lines()
        .filter(|line| !line.starts_with('#'))
    {
        let (support, expected) = line.split_once('\t').unwrap();
        let support: usize = support.parse().unwrap();
        let expected: f64 = expected.parse().unwrap();
        assert_eq!(support, rows); // Ensure exhaustive coverage, including empty.
        let mut counts = [0; 256];
        let data: Vec<_> = (0..support)
            .flat_map(|symbol| {
                let byte = (symbol as u8) ^ 0xa5;
                let count = symbol % 11 + 1;
                counts[usize::from(byte)] = count;
                std::iter::repeat_n(byte, count)
            })
            .collect();
        let d = Distribution::from_counts(ByteHistogram::try_from_counts(counts).unwrap());
        for actual in [hartley_entropy(&data), hartley_entropy_distribution(&d)] {
            assert!(
                (actual - expected).abs() <= TOLERANCE,
                "support {support}: {actual} != {expected}"
            );
        }
        rows += 1;
    }
    assert_eq!(rows, 257);
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        rng_seed: RngSeed::Fixed(0x48a7_1e00),
        ..ProptestConfig::default()
    })]

    #[test]
    fn support_reference_bounds_and_shannon_inequality(
        bytes in prop::collection::vec(any::<u8>(), 0..4097),
        alphabet in 1usize..257,
    ) {
        // Vary support rather than nearly always generating the full alphabet.
        let data: Vec<_> = bytes.iter().map(|&b| (usize::from(b) % alphabet) as u8).collect();
        let support = data.iter().copied().collect::<BTreeSet<_>>().len();
        let reference = if support == 0 { 0.0 } else { (support as f64).ln() / std::f64::consts::LN_2 };
        let d = Distribution::from_bytes(&data);
        let h = hartley_entropy(&data);
        prop_assert!(h.is_finite() && (0.0..=8.0).contains(&h));
        prop_assert!((h - reference).abs() <= TOLERANCE);
        prop_assert!(shannon_distribution(&d) <= h + TOLERANCE);
        prop_assert_eq!(h.to_bits(), hartley_entropy_distribution(&d).to_bits());
        prop_assert_eq!(h.to_bits(), hartley_entropy(&data).to_bits());
    }

    #[test]
    fn permutation_relabeling_and_repetition_preserve_entropy(
        data in prop::collection::vec(any::<u8>(), 0..1025),
        repeats in 1usize..8,
        mask in any::<u8>(),
    ) {
        let h = hartley_entropy(&data).to_bits();
        let mut sorted = data.clone();
        sorted.sort_unstable();
        prop_assert_eq!(h, hartley_entropy(&sorted).to_bits());
        sorted.reverse();
        prop_assert_eq!(h, hartley_entropy(&sorted).to_bits());
        let relabeled: Vec<_> = data.iter().map(|&b| b ^ mask).collect();
        prop_assert_eq!(h, hartley_entropy(&relabeled).to_bits());
        prop_assert_eq!(h, hartley_entropy(&data.repeat(repeats)).to_bits());
    }

    #[test]
    fn frequencies_do_not_matter_when_support_is_fixed(
        entries in prop::collection::vec((any::<bool>(), 1usize..100_001, 1usize..100_001), 256),
    ) {
        let mut left = [0; 256];
        let mut right = [0; 256];
        for (symbol, &(present, a, b)) in entries.iter().enumerate() {
            if present {
                left[symbol] = a;
                right[symbol] = b;
            }
        }
        let a = Distribution::from_counts(ByteHistogram::try_from_counts(left).unwrap());
        let b = Distribution::from_counts(ByteHistogram::try_from_counts(right).unwrap());
        prop_assert_eq!(hartley_entropy_distribution(&a).to_bits(), hartley_entropy_distribution(&b).to_bits());
    }

    #[test]
    fn uniform_frequencies_equal_shannon(k in 1usize..257, repetitions in 1usize..33) {
        let data: Vec<_> = (0..k).map(|i| i as u8).collect();
        let d = Distribution::from_bytes(&data.repeat(repetitions));
        prop_assert!((hartley_entropy_distribution(&d) - shannon_distribution(&d)).abs() <= TOLERANCE);
    }

    #[test]
    fn appending_observations_cannot_reduce_support_entropy(
        data in prop::collection::vec(any::<u8>(), 0..513),
        extra in prop::collection::vec(any::<u8>(), 0..257),
    ) {
        let initial = hartley_entropy(&data);
        let combined = [data, extra].concat();
        prop_assert!(hartley_entropy(&combined) + TOLERANCE >= initial);
    }
}
