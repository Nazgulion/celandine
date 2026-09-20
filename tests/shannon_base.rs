//! Explicit units, parameter validation, independent reference values, and invariants.

use celandine::distribution::{ByteHistogram, Distribution};
use celandine::entropy::{
    InvalidLogBase, shannon, shannon_distribution, shannon_distribution_with_base,
    shannon_with_base,
};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;

fn distribution(counts: &[usize]) -> Distribution {
    let mut table = [0; 256];
    table[..counts.len()].copy_from_slice(counts);
    Distribution::from_counts(ByteHistogram::try_from_counts(table).unwrap())
}

#[test]
fn canonical_units_and_zero_conventions() {
    for (base, expected) in [
        (2.0, 2.0),
        (std::f64::consts::E, 1.386_294_361_119_890_6),
        (10.0, 0.602_059_991_327_962_4),
        (4.0, 1.0),
        (256.0, 0.25),
    ] {
        assert!((shannon_with_base(b"ABCD", base).unwrap() - expected).abs() <= 1e-12);
    }
    for base in [
        1.0_f64.next_up(),
        1.5,
        2.0,
        std::f64::consts::E,
        10.0,
        f64::MAX,
    ] {
        for data in [&b""[..], b"A", b"AAAA"] {
            let actual = shannon_with_base(data, base).unwrap();
            assert_eq!(actual.to_bits(), 0.0_f64.to_bits());
            assert_eq!(
                actual.to_bits(),
                shannon_distribution_with_base(&Distribution::from_bytes(data), base)
                    .unwrap()
                    .to_bits()
            );
        }
        let h = shannon_with_base(b"AB", base).unwrap();
        assert!(h.is_finite() && h > 0.0);
    }
}

#[test]
fn invalid_bases_are_rejected_even_without_uncertainty() {
    for base in [
        f64::NAN,
        f64::NEG_INFINITY,
        f64::INFINITY,
        -2.0,
        -0.0,
        0.0,
        f64::from_bits(1),
        0.5,
        1.0_f64.next_down(),
        1.0,
    ] {
        for data in [&b""[..], b"AAAA", b"AB"] {
            assert_eq!(shannon_with_base(data, base), Err(InvalidLogBase));
            assert_eq!(
                shannon_distribution_with_base(&Distribution::from_bytes(data), base),
                Err(InvalidLogBase)
            );
        }
    }
    assert_eq!(
        InvalidLogBase.to_string(),
        "logarithm base must be finite and greater than one"
    );
}

#[test]
fn uniform_support_matches_logarithmic_count_in_each_unit() {
    for k in 1..=256 {
        let d = distribution(&vec![7; k]);
        for base in [
            1.0_f64.next_up(),
            1.0001,
            2.0,
            std::f64::consts::E,
            10.0,
            f64::MAX,
        ] {
            // Independent uniform-law identity using natural logarithms.
            let expected = (k as f64).ln() / base.ln();
            let actual = shannon_distribution_with_base(&d, base).unwrap();
            assert!(((actual - expected) * base.log2()).abs() <= 1e-12);
        }
    }
}

#[test]
fn independent_exact_count_and_decimal_fixtures() {
    let mut checked = 0;
    let mut skipped = 0;
    for line in include_str!("data/shannon_base.tsv")
        .lines()
        .filter(|s| !s.starts_with('#'))
    {
        let fields: Vec<_> = line.split('\t').collect();
        let counts: Vec<u128> = if fields[1] == "-" {
            vec![]
        } else {
            fields[1].split(',').map(|s| s.parse().unwrap()).collect()
        };
        let total = counts.iter().sum::<u128>();
        if total > usize::MAX as u128 {
            skipped += 1;
            continue;
        }
        let counts: Vec<_> = counts.into_iter().map(|c| c as usize).collect();
        let d = distribution(&counts);
        let base = f64::from_bits(u64::from_str_radix(fields[2], 16).unwrap());
        let expected: f64 = fields[3].parse().unwrap();
        let actual = shannon_distribution_with_base(&d, base).unwrap();
        assert!(actual.is_finite());
        // Error is assessed in equivalent bits: bases near one amplify absolute error.
        assert!(
            ((actual - expected) * base.log2()).abs() <= 1e-12,
            "{} base {base}: {actual} != {expected}",
            fields[0]
        );
        if expected == 0.0 {
            assert_eq!(actual.to_bits(), 0.0_f64.to_bits());
        } else {
            assert!(actual > 0.0);
            if total <= (1u128 << 53) {
                assert!((actual / expected - 1.0).abs() <= 1e-12);
            }
        }
        if total <= 4096 {
            let data: Vec<_> = counts
                .iter()
                .enumerate()
                .flat_map(|(b, &c)| std::iter::repeat_n(b as u8, c))
                .collect();
            assert_eq!(
                actual.to_bits(),
                shannon_with_base(&data, base).unwrap().to_bits()
            );
        }
        checked += 1;
    }
    assert_eq!(checked + skipped, 144);
    assert!(checked >= 108);
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256, rng_seed: RngSeed::Fixed(0x4241_5345), ..ProptestConfig::default()
    })]

    #[test]
    fn conversion_bounds_and_base_two_compatibility(
        data in prop::collection::vec(any::<u8>(), 0..1025),
        bits in (1.0_f64.to_bits() + 1)..=f64::MAX.to_bits(),
    ) {
        let base = f64::from_bits(bits);
        let d = Distribution::from_bytes(&data);
        let h = shannon_with_base(&data, base).unwrap();
        prop_assert!(h.is_finite() && h >= 0.0);
        prop_assert_eq!(h.to_bits(), shannon_distribution_with_base(&d, base).unwrap().to_bits());
        prop_assert_eq!(shannon_with_base(&data, 2.0).unwrap().to_bits(), shannon(&data).to_bits());
        prop_assert_eq!(shannon_distribution_with_base(&d, 2.0).unwrap().to_bits(), shannon_distribution(&d).to_bits());
        let bound = (d.support_size().max(1) as f64).ln();
        prop_assert!(h * base.ln() <= bound + 1e-12);
        prop_assert!((h * base.log2() - shannon(&data)).abs() <= 1e-12);
    }

    #[test]
    fn increasing_base_decreases_value_and_relabeling_preserves_it(
        data in prop::collection::vec(0u8..8, 0..1025), mask in any::<u8>(),
        a in (1.0_f64.to_bits() + 1)..=f64::MAX.to_bits(),
        b in (1.0_f64.to_bits() + 1)..=f64::MAX.to_bits(),
    ) {
        let low = f64::from_bits(a.min(b));
        let high = f64::from_bits(a.max(b));
        let h_low = shannon_with_base(&data, low).unwrap();
        let h_high = shannon_with_base(&data, high).unwrap();
        prop_assert!(h_low >= h_high);
        let renamed: Vec<_> = data.iter().map(|v| v ^ mask).collect();
        let h_renamed = shannon_with_base(&renamed, low).unwrap();
        prop_assert!(((h_low - h_renamed) * low.log2()).abs() <= 1e-12);
    }
}
