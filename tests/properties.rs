//! Seeded mathematical and representation property tests.

use celandine::distribution::{ByteHistogram, Distribution};
use celandine::entropy::{shannon, shannon_distribution};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use std::collections::BTreeMap;

const TOLERANCE: f64 = 1e-12;

// A different counting structure and log base provide a second Rust cross-check.
fn reference(data: &[u8]) -> f64 {
    let mut counts = BTreeMap::<u8, usize>::new();
    for &symbol in data {
        *counts.entry(symbol).or_default() += 1;
    }
    counts
        .values()
        .map(|&count| {
            let p = count as f64 / data.len() as f64;
            -p * p.ln() / std::f64::consts::LN_2
        })
        .sum()
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        rng_seed: RngSeed::Fixed(0x005e_91f0),
        ..ProptestConfig::default()
    })]

    #[test]
    fn histogram_and_entropy_invariants(data in prop::collection::vec(any::<u8>(), 0..4097)) {
        let d = Distribution::from_bytes(&data);
        let h = shannon_distribution(&d);
        prop_assert_eq!(d.sample_size(), data.len());
        prop_assert_eq!(d.histogram().counts().iter().sum::<usize>(), data.len());
        let mut distinct = data.clone();
        distinct.sort_unstable();
        distinct.dedup();
        prop_assert_eq!(d.support_size(), distinct.len());
        for (symbol, &count) in d.histogram().counts().iter().enumerate() {
            prop_assert_eq!(count, data.iter().filter(|&&b| usize::from(b) == symbol).count());
        }
        prop_assert!(d.probabilities().all(|p| p.is_finite() && (0.0..=1.0).contains(&p)));
        let expected_mass = if data.is_empty() { 0.0 } else { 1.0 };
        prop_assert!((d.probabilities().sum::<f64>() - expected_mass).abs() <= TOLERANCE);
        prop_assert!(h.is_finite() && h >= 0.0);
        prop_assert!(h <= (d.support_size().max(1) as f64).log2() + TOLERANCE);
        prop_assert!((h - reference(&data)).abs() <= TOLERANCE);
        prop_assert_eq!(h.to_bits(), shannon(&data).to_bits());
        prop_assert_eq!(h.to_bits(), shannon_distribution(&d).to_bits());
    }

    #[test]
    fn permutation_relabeling_and_repetition(
        data in prop::collection::vec(any::<u8>(), 0..2049),
        repeats in 1usize..8,
        mask in any::<u8>(),
    ) {
        let h = shannon(&data);
        let mut sorted = data.clone();
        sorted.sort_unstable();
        prop_assert_eq!(h.to_bits(), shannon(&sorted).to_bits());
        sorted.reverse();
        prop_assert_eq!(h.to_bits(), shannon(&sorted).to_bits());
        let relabeled: Vec<_> = data.iter().map(|&byte| byte ^ mask).collect();
        prop_assert!((h - shannon(&relabeled)).abs() <= TOLERANCE);
        prop_assert!((h - shannon(&data.repeat(repeats))).abs() <= TOLERANCE);
    }

    #[test]
    fn constant_sequences(symbol in any::<u8>(), len in 1usize..8193) {
        prop_assert_eq!(shannon(&vec![symbol; len]).to_bits(), 0.0_f64.to_bits());
    }

    #[test]
    fn uniform_empirical_distributions(k in 1usize..257, repeats in 1usize..33) {
        let data: Vec<_> = (0..k).map(|i| i as u8).collect();
        prop_assert!((shannon(&data.repeat(repeats)) - (k as f64).log2()).abs() <= TOLERANCE);
    }

    #[test]
    fn supplied_counts_agree_with_expanded_samples(counts in prop::collection::vec(0usize..65, 256)) {
        let counts: [usize; 256] = counts.try_into().unwrap();
        let histogram = ByteHistogram::try_from_counts(counts).unwrap();
        let data: Vec<_> = counts.iter().enumerate()
            .flat_map(|(symbol, &count)| std::iter::repeat_n(symbol as u8, count)).collect();
        prop_assert_eq!(&histogram, &ByteHistogram::from_bytes(&data));
        prop_assert_eq!(shannon_distribution(&Distribution::from_counts(histogram)), shannon(&data));
    }
}
