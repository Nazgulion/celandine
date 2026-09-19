//! Collision entropy verified through matching pairs, exact counts, and invariants.

use celandine::distribution::{ByteHistogram, Distribution};
use celandine::entropy::{
    collision_entropy, collision_entropy_distribution, hartley_entropy_distribution,
    renyi_entropy_distribution, shannon_distribution,
};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;

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
        (&b"AAAB"[..], (8.0_f64 / 5.0).log2()),
        (&[0, 0, 128, 255][..], (8.0_f64 / 3.0).log2()),
    ] {
        let h = collision_entropy(data);
        assert!((h - expected).abs() <= TOL, "{data:?}: {h}");
        assert_eq!(
            h.to_bits(),
            collision_entropy_distribution(&Distribution::from_bytes(data)).to_bits()
        );
        if expected == 0.0 {
            assert_eq!(h.to_bits(), 0.0_f64.to_bits());
        }
    }
    for symbol in 0..=255 {
        assert_eq!(
            collision_entropy(&[symbol; 32]).to_bits(),
            0.0_f64.to_bits()
        );
    }
    assert_eq!(
        collision_entropy_distribution(&distribution(&[])).to_bits(),
        0.0_f64.to_bits()
    );
}

#[test]
fn every_uniform_support_has_logarithmic_entropy() {
    for k in 1..=256 {
        let h = collision_entropy_distribution(&distribution(&vec![7; k]));
        assert!((h - (k as f64).log2()).abs() <= TOL);
    }
}

#[test]
fn independent_fraction_decimal_fixtures() {
    let mut checked = 0;
    let mut skipped = 0;
    for line in include_str!("data/collision.tsv")
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
        let h = collision_entropy_distribution(&d);
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
            assert_eq!(h.to_bits(), collision_entropy(&data).to_bits());
        }
        checked += 1;
    }
    assert_eq!(checked + skipped, 45);
    assert!(checked >= 10);
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        rng_seed: RngSeed::Fixed(0xc011_1510),
        ..ProptestConfig::default()
    })]

    #[test]
    fn explicit_ordered_pairs_define_matching_probability(
        data in prop::collection::vec(0u8..16, 1..65),
    ) {
        // Independent reference: enumerate pairs including self-pairs, not a histogram.
        let matching = data.iter().flat_map(|a| data.iter().map(move |b| usize::from(a == b))).sum::<usize>();
        let probability = matching as f64 / (data.len() * data.len()) as f64;
        let h = collision_entropy(&data);
        prop_assert!((h + probability.log2()).abs() <= TOL);
        prop_assert!((2.0_f64.powf(-h) - probability).abs() <= TOL);
    }

    #[test]
    fn wide_counts_match_exact_integer_complement_and_entropy_bounds(
        entries in prop::collection::vec((any::<usize>(), 0u32..usize::BITS), 1..257),
        dominant in any::<bool>(),
    ) {
        // Division bounds the total on both 32- and 64-bit targets.
        let mut counts: Vec<_> = entries.iter().map(|&(c, shift)| (c >> shift) / 512).collect();
        if dominant { counts[0] = usize::MAX - counts[1..].iter().sum::<usize>(); }
        let d = distribution(&counts);
        let h = collision_entropy_distribution(&d);
        prop_assert!(h.is_finite() && (-TOL..=8.0 + TOL).contains(&h));
        prop_assert_eq!(h.to_bits(), renyi_entropy_distribution(&d, 2.0).unwrap().to_bits());
        prop_assert!(h <= shannon_distribution(&d) + TOL);
        prop_assert!(h <= hartley_entropy_distribution(&d) + TOL);
        if d.is_empty() { prop_assert_eq!(h.to_bits(), 0.0_f64.to_bits()); }
        else {
            // For 32/64-bit usize, n^2 and sum(c^2) fit exactly in u128.
            let n = d.sample_size() as u128;
            let same: u128 = counts.iter().map(|&c| (c as u128) * (c as u128)).sum();
            let different = n*n - same;
            let q = different as f64 / (n*n) as f64;
            let reference = -(-q).ln_1p() / std::f64::consts::LN_2;
            prop_assert!((h - reference).abs() <= TOL);
            if different > 0 { prop_assert!((h / reference - 1.0).abs() <= 1e-8); }
        }
    }

    #[test]
    fn permutation_relabeling_and_repetition_preserve_entropy(
        data in prop::collection::vec(any::<u8>(), 0..1025),
        mask in any::<u8>(), repeats in 1usize..8,
    ) {
        let h = collision_entropy(&data);
        let mut sorted = data.clone();
        sorted.sort_unstable();
        prop_assert_eq!(h.to_bits(), collision_entropy(&sorted).to_bits());
        prop_assert_eq!(h.to_bits(), collision_entropy(&data).to_bits());
        let relabeled: Vec<_> = data.iter().map(|b| b ^ mask).collect();
        prop_assert!((h - collision_entropy(&relabeled)).abs() <= TOL);
        prop_assert!((h - collision_entropy(&data.repeat(repeats))).abs() <= TOL);
        prop_assert_eq!(h.to_bits(), collision_entropy_distribution(&Distribution::from_bytes(&data)).to_bits());
    }
}
