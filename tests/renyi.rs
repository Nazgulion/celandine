//! Rényi canonical values, high-precision references, and mathematical invariants.

use celandine::distribution::{ByteHistogram, Distribution};
use celandine::entropy::{
    InvalidRenyiOrder, hartley_entropy_distribution, renyi_entropy, renyi_entropy_distribution,
    shannon_distribution,
};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;

const TOL: f64 = 1e-12;
const ORDERS: &[f64] = &[
    0.0,
    f64::from_bits(1),
    f64::MIN_POSITIVE,
    0.1,
    0.5,
    0.75_f64.next_down(),
    0.75,
    1.0 - 1e-8,
    1.0 - 1e-12,
    1.0_f64.next_down(),
    1.0,
    1.0_f64.next_up(),
    1.0 + 1e-12,
    1.0 + 1e-8,
    1.25,
    1.25_f64.next_up(),
    2.0,
    16.0,
    1e6,
    1e100,
    f64::MAX,
    f64::INFINITY,
];

fn distribution(counts: &[usize]) -> Distribution {
    let mut table = [0; 256];
    table[..counts.len()].copy_from_slice(counts);
    Distribution::from_counts(ByteHistogram::try_from_counts(table).unwrap())
}

fn close(actual: f64, expected: f64, context: &str) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= TOL,
        "{context}: {actual:.17e} != {expected:.17e}"
    );
}

#[test]
fn canonical_values() {
    for (data, expected) in [
        (&b""[..], 0.0),
        (&b"A"[..], 0.0),
        (&b"AAAA"[..], 0.0),
        (&b"AB"[..], 1.0),
        (&b"ABAB"[..], 1.0),
        (&b"ABCD"[..], 2.0),
        (&b"ABCABCABC"[..], 3.0_f64.log2()),
        (&b"0123456789"[..], std::f64::consts::LOG2_10),
    ] {
        for &alpha in ORDERS {
            close(renyi_entropy(data, alpha).unwrap(), expected, "canonical");
        }
    }
    close(
        renyi_entropy(b"AAAB", 0.5).unwrap(),
        2.0 * ((3.0_f64.sqrt() + 1.0) / 2.0).log2(),
        "AAAB half",
    );
    close(
        renyi_entropy(b"AAAB", 2.0).unwrap(),
        (8.0_f64 / 5.0).log2(),
        "AAAB two",
    );
    close(
        renyi_entropy(b"AAAB", f64::INFINITY).unwrap(),
        (4.0_f64 / 3.0).log2(),
        "AAAB infinity",
    );
}

#[test]
fn validation_precedes_empty_and_constant_conventions() {
    for data in [&b""[..], &b"A"[..], &b"AAAB"[..]] {
        let d = Distribution::from_bytes(data);
        for alpha in [
            -f64::from_bits(1),
            -1.0,
            f64::NEG_INFINITY,
            f64::NAN,
            -f64::NAN,
        ] {
            assert_eq!(renyi_entropy(data, alpha), Err(InvalidRenyiOrder));
            assert_eq!(
                renyi_entropy_distribution(&d, alpha),
                Err(InvalidRenyiOrder)
            );
        }
        assert_eq!(renyi_entropy(data, -0.0), renyi_entropy(data, 0.0));
    }
    for data in [&b""[..], &b"A"[..], &b"AAAA"[..]] {
        for &alpha in ORDERS {
            assert_eq!(
                renyi_entropy(data, alpha).unwrap().to_bits(),
                0.0_f64.to_bits()
            );
        }
    }
}

#[test]
fn uniform_laws_at_every_support_and_order() {
    for k in 1..=256 {
        let d = distribution(&vec![7; k]);
        for &alpha in ORDERS {
            close(
                renyi_entropy_distribution(&d, alpha).unwrap(),
                (k as f64).log2(),
                "uniform",
            );
        }
    }
}

#[test]
fn large_counts_preserve_tiny_positive_entropy() {
    let d = distribution(&[usize::MAX - 1, 1]);
    // -ln(1-epsilon)/ln(2) ~= epsilon/ln(2), with negligible O(epsilon^2).
    let expected = (1.0 / usize::MAX as f64) / std::f64::consts::LN_2;
    let infinity = renyi_entropy_distribution(&d, f64::INFINITY).unwrap();
    assert!((infinity / expected - 1.0).abs() < 1e-9);
    for &alpha in ORDERS {
        let h = renyi_entropy_distribution(&d, alpha).unwrap();
        assert!(
            h.is_finite() && h > 0.0 && h <= 1.0 + TOL,
            "alpha={alpha}: {h}"
        );
    }
}

#[test]
fn independent_decimal_fixtures() {
    let mut rows = 0;
    let mut skipped = 0;
    for line in include_str!("data/renyi.tsv")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let columns: Vec<_> = line.split('\t').collect();
        let name = columns[0];
        let alpha: f64 = columns[1].parse().unwrap();
        let counts: Vec<u128> = if columns[2] == "-" {
            vec![]
        } else {
            columns[2].split(',').map(|c| c.parse().unwrap()).collect()
        };
        if counts.iter().sum::<u128>() > usize::MAX as u128 {
            skipped += 1;
            continue; // Explicitly 64-bit fixtures are not representable on 32-bit.
        }
        let counts: Vec<_> = counts.into_iter().map(|c| c as usize).collect();
        let d = distribution(&counts);
        let expected: f64 = columns[3].parse().unwrap();
        let actual = renyi_entropy_distribution(&d, alpha).unwrap();
        close(actual, expected, &format!("{name} alpha={alpha}"));
        // Tiny entropies must not merely pass the absolute tolerance as zero.
        // Exact order one inherits Shannon's documented absolute-error contract.
        if expected > 0.0 && alpha != 1.0 {
            assert!(
                (actual / expected - 1.0).abs() < 1e-8,
                "relative: {name} alpha={alpha}"
            );
        }
        if d.sample_size() <= 4096 {
            let bytes: Vec<_> = counts
                .iter()
                .enumerate()
                .flat_map(|(i, &n)| std::iter::repeat_n(i as u8, n))
                .collect();
            assert_eq!(
                renyi_entropy(&bytes, alpha).unwrap().to_bits(),
                actual.to_bits()
            );
        }
        rows += 1;
    }
    assert_eq!(rows + skipped, 345);
    assert!(rows >= 276);
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        rng_seed: RngSeed::Fixed(0x7265_6e79_6900),
        ..ProptestConfig::default()
    })]

    #[test]
    fn order_monotonicity_and_bounds(
        counts in prop::collection::vec(0usize..1_000_001, 1..257),
        a_bits in 0u64..=0x7ff0_0000_0000_0000,
        b_bits in 0u64..=0x7ff0_0000_0000_0000,
    ) {
        let d = distribution(&counts);
        let upper = hartley_entropy_distribution(&d);
        let lower = renyi_entropy_distribution(&d, f64::INFINITY).unwrap();
        let mut previous = upper;
        for &alpha in ORDERS {
            let h = renyi_entropy_distribution(&d, alpha).unwrap();
            prop_assert!(h.is_finite() && (-TOL..=8.0 + TOL).contains(&h));
            prop_assert!(h >= lower - TOL && h <= previous + TOL, "order {alpha}: {h}, previous {previous}");
            previous = h;
        }
        let a = f64::from_bits(a_bits.min(b_bits));
        let b = f64::from_bits(a_bits.max(b_bits));
        let ha = renyi_entropy_distribution(&d, a).unwrap();
        let hb = renyi_entropy_distribution(&d, b).unwrap();
        prop_assert!(ha.is_finite() && hb.is_finite());
        prop_assert!(ha <= upper + TOL && hb >= lower - TOL && ha + TOL >= hb);
    }

    #[test]
    fn special_orders_match_their_definitions(
        counts in prop::collection::vec(0usize..1_000_001, 1..257),
    ) {
        let d = distribution(&counts);
        prop_assert_eq!(renyi_entropy_distribution(&d, 0.0).unwrap().to_bits(), hartley_entropy_distribution(&d).to_bits());
        prop_assert_eq!(renyi_entropy_distribution(&d, 1.0).unwrap().to_bits(), shannon_distribution(&d).to_bits());
        if !d.is_empty() {
            let collision = -d.probabilities().map(|p| p * p).sum::<f64>().log2();
            prop_assert!((renyi_entropy_distribution(&d, 2.0).unwrap() - collision).abs() <= TOL);
        }
        for a in [1.0_f64.next_down(), 1.0_f64.next_up()] {
            prop_assert!((renyi_entropy_distribution(&d, a).unwrap() - shannon_distribution(&d)).abs() <= TOL);
        }
    }

    #[test]
    fn permutation_relabeling_repetition_and_api_agreement(
        data in prop::collection::vec(any::<u8>(), 0..1025),
        mask in any::<u8>(), repetitions in 1usize..8,
        alpha in prop::sample::select(ORDERS.to_vec()),
    ) {
        let h = renyi_entropy(&data, alpha).unwrap();
        let d = Distribution::from_bytes(&data);
        prop_assert_eq!(h.to_bits(), renyi_entropy_distribution(&d, alpha).unwrap().to_bits());
        prop_assert_eq!(h.to_bits(), renyi_entropy(&data, alpha).unwrap().to_bits());
        let mut sorted = data.clone();
        sorted.sort_unstable();
        prop_assert_eq!(h.to_bits(), renyi_entropy(&sorted, alpha).unwrap().to_bits());
        let relabeled: Vec<_> = data.iter().map(|b| b ^ mask).collect();
        prop_assert!((h - renyi_entropy(&relabeled, alpha).unwrap()).abs() <= TOL);
        prop_assert!((h - renyi_entropy(&data.repeat(repetitions), alpha).unwrap()).abs() <= TOL);
    }
}
