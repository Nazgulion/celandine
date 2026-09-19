//! Min-entropy verified through individual surprises, exact frequencies, and invariants.

use celandine::distribution::{ByteHistogram, Distribution};
use celandine::entropy::{
    collision_entropy_distribution, hartley_entropy_distribution, min_entropy,
    min_entropy_distribution, renyi_entropy_distribution, shannon_distribution,
};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use std::collections::BTreeMap;

const TOL: f64 = 1e-12;

fn distribution(counts: &[usize]) -> Distribution {
    let mut table = [0; 256];
    table[..counts.len()].copy_from_slice(counts);
    Distribution::from_counts(ByteHistogram::try_from_counts(table).unwrap())
}

#[test]
fn canonical_values_and_positive_zero() {
    for (data, expected) in [
        (&b""[..], 0.0),
        (&b"A"[..], 0.0),
        (&b"AAAA"[..], 0.0),
        (&b"AB"[..], 1.0),
        (&b"ABAB"[..], 1.0),
        (&b"ABCD"[..], 2.0),
        (&b"ABCABCABC"[..], 3.0_f64.log2()),
        (&b"0123456789"[..], std::f64::consts::LOG2_10),
        (&b"AAAB"[..], (4.0_f64 / 3.0).log2()),
        (&[0, 0, 128, 255][..], 1.0),
    ] {
        let h = min_entropy(data);
        assert!((h - expected).abs() <= TOL, "{data:?}: {h}");
        assert_eq!(
            h.to_bits(),
            min_entropy_distribution(&Distribution::from_bytes(data)).to_bits()
        );
        if expected == 0.0 {
            assert_eq!(h.to_bits(), 0.0_f64.to_bits());
        }
    }
    for symbol in 0..=255 {
        assert_eq!(min_entropy(&[symbol; 32]).to_bits(), 0.0_f64.to_bits());
    }
    assert_eq!(
        min_entropy_distribution(&distribution(&[])).to_bits(),
        0.0_f64.to_bits()
    );
}

#[test]
fn every_uniform_support_has_logarithmic_entropy() {
    for k in 1..=256 {
        let h = min_entropy_distribution(&distribution(&vec![7; k]));
        assert!((h - (k as f64).log2()).abs() <= TOL);
    }
}

#[test]
fn independent_fraction_decimal_fixtures() {
    let mut checked = 0;
    let mut skipped = 0;
    for line in include_str!("data/min_entropy.tsv")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let fields: Vec<_> = line.split('\t').collect();
        let counts: Vec<u128> = if fields[1] == "-" {
            vec![]
        } else {
            fields[1].split(',').map(|c| c.parse().unwrap()).collect()
        };
        if counts.iter().sum::<u128>() > usize::MAX as u128 {
            skipped += 1; // 64-bit fixtures are explicitly skipped on narrower targets.
            continue;
        }
        let counts: Vec<_> = counts.into_iter().map(|c| c as usize).collect();
        let d = distribution(&counts);
        let expected: f64 = fields[2].parse().unwrap();
        let h = min_entropy_distribution(&d);
        assert!(
            h.is_finite() && (h - expected).abs() <= TOL,
            "{}: {h} != {expected}",
            fields[0]
        );
        if expected > 0.0 {
            assert!(
                (h / expected - 1.0).abs() <= 1e-8,
                "relative error: {}",
                fields[0]
            );
        }
        if d.sample_size() <= 4096 {
            let data: Vec<_> = counts
                .iter()
                .enumerate()
                .flat_map(|(b, &n)| std::iter::repeat_n(b as u8, n))
                .collect();
            assert_eq!(h.to_bits(), min_entropy(&data).to_bits());
        }
        checked += 1;
    }
    assert_eq!(checked + skipped, 48);
    assert!(checked >= 13);
}

#[test]
fn same_maximum_and_total_ignore_other_frequency_changes() {
    let a = distribution(&[6, 3, 1]);
    let b = distribution(&[6, 2, 2]);
    assert_eq!(
        min_entropy_distribution(&a).to_bits(),
        min_entropy_distribution(&b).to_bits()
    );
    assert!((min_entropy_distribution(&a) - (5.0_f64 / 3.0).log2()).abs() <= TOL);
    assert_ne!(
        collision_entropy_distribution(&a),
        collision_entropy_distribution(&b)
    );
}

#[test]
fn tiny_entropy_survives_counts_near_usize_maximum() {
    let d = distribution(&[usize::MAX - 1, 1]);
    let h = min_entropy_distribution(&d);
    // -ln(1-epsilon)/ln(2) = epsilon/ln(2) + O(epsilon^2).
    let first_order = (1.0 / usize::MAX as f64) / std::f64::consts::LN_2;
    assert!(h > 0.0 && (h / first_order - 1.0).abs() < 1e-9);
    assert_eq!(
        min_entropy_distribution(&distribution(&[usize::MAX])).to_bits(),
        0.0_f64.to_bits()
    );
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        rng_seed: RngSeed::Fixed(0x4d49_4e00),
        ..ProptestConfig::default()
    })]

    #[test]
    fn minimum_symbol_surprise_and_best_fixed_guess(
        data in prop::collection::vec(any::<u8>(), 1..513),
    ) {
        // Independent map tally, then minimize each individual symbol's surprise.
        let mut counts = BTreeMap::<u8, usize>::new();
        for &b in &data { *counts.entry(b).or_default() += 1; }
        let expected = counts.values().map(|&c| -(c as f64 / data.len() as f64).log2())
            .fold(f64::INFINITY, f64::min);
        let h = min_entropy(&data);
        prop_assert!((h - expected).abs() <= TOL);
        let best_guess = *counts.values().max().unwrap() as f64 / data.len() as f64;
        prop_assert!((2.0_f64.powf(-h) - best_guess).abs() <= TOL);
    }

    #[test]
    fn wide_counts_preserve_entropy_bounds_and_recover_largest_probability(
        entries in prop::collection::vec((any::<usize>(), 0u32..usize::BITS), 1..257),
        dominant in any::<bool>(),
    ) {
        // Division bounds the total on both 32- and 64-bit targets.
        let mut counts: Vec<_> = entries.iter().map(|&(c, shift)| (c >> shift) / 512).collect();
        if dominant { counts[0] = usize::MAX - counts[1..].iter().sum::<usize>(); }
        let d = distribution(&counts);
        let h = min_entropy_distribution(&d);
        prop_assert!(h.is_finite() && (-TOL..=8.0 + TOL).contains(&h));
        prop_assert_eq!(h.to_bits(), renyi_entropy_distribution(&d, f64::INFINITY).unwrap().to_bits());
        prop_assert!(h <= collision_entropy_distribution(&d) + TOL);
        prop_assert!(h <= shannon_distribution(&d) + TOL);
        prop_assert!(h <= hartley_entropy_distribution(&d) + TOL);
        if d.is_empty() { prop_assert_eq!(h.to_bits(), 0.0_f64.to_bits()); }
        else {
            let maximum = *counts.iter().max().unwrap();
            let p_max = maximum as f64 / d.sample_size() as f64;
            prop_assert!((2.0_f64.powf(-h) - p_max).abs() <= TOL);
            if maximum < d.sample_size() {
                prop_assert!(h > 0.0); // A rounded probability of one must not erase entropy.
            }
            prop_assert!(h <= renyi_entropy_distribution(&d, 10000.0).unwrap() + TOL);
        }
    }

    #[test]
    fn permutation_relabeling_and_repetition_preserve_entropy(
        data in prop::collection::vec(any::<u8>(), 0..1025),
        mask in any::<u8>(), repeats in 1usize..8,
    ) {
        let h = min_entropy(&data);
        let mut sorted = data.clone();
        sorted.sort_unstable();
        prop_assert_eq!(h.to_bits(), min_entropy(&sorted).to_bits());
        prop_assert_eq!(h.to_bits(), min_entropy(&data).to_bits());
        let relabeled: Vec<_> = data.iter().map(|b| b ^ mask).collect();
        prop_assert!((h - min_entropy(&relabeled)).abs() <= TOL);
        prop_assert!((h - min_entropy(&data.repeat(repeats))).abs() <= TOL);
        prop_assert_eq!(h.to_bits(), min_entropy_distribution(&Distribution::from_bytes(&data)).to_bits());
    }
}
