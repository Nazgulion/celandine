//! Tsallis definitions, independent Decimal fixtures, and algebraic properties.

use celandine::distribution::{ByteHistogram, Distribution};
use celandine::entropy::{
    InvalidTsallisOrder, renyi_entropy_distribution, shannon_distribution, tsallis_entropy,
    tsallis_entropy_distribution,
};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use std::f64::consts::LN_2;

const TOL: f64 = 1e-12;
const ORDERS: &[f64] = &[
    0.0,
    f64::from_bits(1),
    f64::MIN_POSITIVE,
    0.1,
    0.5_f64.next_down(),
    0.5,
    0.5_f64.next_up(),
    0.9,
    1.0 - 1e-8,
    1.0_f64.next_down(),
    1.0,
    1.0_f64.next_up(),
    1.0 + 1e-8,
    1.1,
    2.0,
    4.0,
    16.0,
    128.0,
    1e6,
    1e18,
    1e20,
    1e100,
    f64::MAX,
];

fn distribution(counts: &[usize]) -> Distribution {
    let mut table = [0; 256];
    table[..counts.len()].copy_from_slice(counts);
    Distribution::from_counts(ByteHistogram::try_from_counts(table).unwrap())
}

fn close(actual: f64, expected: f64) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= TOL,
        "actual={actual:.17e}, expected={expected:.17e}"
    );
}

#[test]
fn canonical_values_and_scaling() {
    for (data, support, different) in [
        (&b""[..], 0, 0.0),
        (&b"A"[..], 1, 0.0),
        (&b"AAAA"[..], 1, 0.0),
        (&b"AB"[..], 2, 0.5),
        (&b"ABAB"[..], 2, 0.5),
        (&b"ABCD"[..], 4, 0.75),
        (&b"ABCABCABC"[..], 3, 2.0 / 3.0),
        (&b"0123456789"[..], 10, 0.9),
        (&b"AAAB"[..], 2, 0.375),
        (&[0, 0, 128, 255][..], 3, 0.625),
    ] {
        let d = Distribution::from_bytes(data);
        close(tsallis_entropy(data, 2.0).unwrap(), different / LN_2);
        close(
            tsallis_entropy(data, 0.0).unwrap(),
            (support as f64 - 1.0).max(0.0) / LN_2,
        );
        for &q in ORDERS {
            assert_eq!(
                tsallis_entropy(data, q).unwrap().to_bits(),
                tsallis_entropy_distribution(&d, q).unwrap().to_bits()
            );
        }
        assert_eq!(
            tsallis_entropy(data, 1.0).unwrap().to_bits(),
            shannon_distribution(&d).to_bits()
        );
    }
    close(
        tsallis_entropy(b"AAAB", 0.5).unwrap(),
        (3.0_f64.sqrt() - 1.0) / LN_2,
    );
    // The scale is fixed across orders: uniform binary input need not give 1.
    close(tsallis_entropy(b"AB", 0.0).unwrap(), 1.0 / LN_2);
}

#[test]
fn validation_precedes_empty_and_constant_handling() {
    for data in [&b""[..], &b"A"[..], &b"AAAB"[..]] {
        let d = Distribution::from_bytes(data);
        for q in [
            -f64::from_bits(1),
            -1.0,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NAN,
            -f64::NAN,
        ] {
            assert_eq!(tsallis_entropy(data, q), Err(InvalidTsallisOrder));
            assert_eq!(
                tsallis_entropy_distribution(&d, q),
                Err(InvalidTsallisOrder)
            );
        }
        assert_eq!(tsallis_entropy(data, -0.0), tsallis_entropy(data, 0.0));
    }
}

#[test]
fn empty_and_all_constant_symbols_return_positive_zero() {
    for &q in ORDERS {
        assert_eq!(
            tsallis_entropy(b"", q).unwrap().to_bits(),
            0.0_f64.to_bits()
        );
        for byte in 0..=255 {
            assert_eq!(
                tsallis_entropy(&[byte; 16], q).unwrap().to_bits(),
                0.0_f64.to_bits()
            );
        }
        assert_eq!(
            tsallis_entropy_distribution(&distribution(&[usize::MAX]), q)
                .unwrap()
                .to_bits(),
            0.0_f64.to_bits()
        );
    }
}

#[test]
fn every_uniform_support_matches_closed_forms() {
    for k in 1..=256 {
        let d = distribution(&vec![7; k]);
        for (q, expected) in [
            (0.0, (k - 1) as f64 / LN_2),
            (0.5, 2.0 * ((k as f64).sqrt() - 1.0) / LN_2),
            (1.0, (k as f64).log2()),
            (2.0, (1.0 - 1.0 / k as f64) / LN_2),
            (4.0, (1.0 - 1.0 / (k as f64).powi(3)) / 3.0 / LN_2),
        ] {
            close(tsallis_entropy_distribution(&d, q).unwrap(), expected);
        }
    }
    assert!(tsallis_entropy_distribution(&distribution(&[1; 256]), 0.0).unwrap() > 8.0);
}

#[test]
fn independent_decimal_fixtures() {
    let mut checked = 0;
    let mut skipped = 0;
    for line in include_str!("data/tsallis.tsv")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let fields: Vec<_> = line.split('\t').collect();
        let q: f64 = fields[1].parse().unwrap();
        let counts: Vec<u128> = if fields[2] == "-" {
            vec![]
        } else {
            fields[2].split(',').map(|c| c.parse().unwrap()).collect()
        };
        if counts.iter().sum::<u128>() > usize::MAX as u128 {
            skipped += 1;
            continue;
        }
        let counts: Vec<_> = counts.into_iter().map(|c| c as usize).collect();
        let d = distribution(&counts);
        let expected: f64 = fields[3].parse().unwrap();
        let h = tsallis_entropy_distribution(&d, q).unwrap();
        assert!(
            h.is_finite() && (h - expected).abs() <= TOL,
            "{} q={q}: {h:.17e} != {expected:.17e}",
            fields[0]
        );
        // Order one deliberately inherits Shannon's absolute-error contract.
        if expected > 0.0 && q != 1.0 {
            assert!(
                (h / expected - 1.0).abs() <= 1e-8,
                "relative error {} q={q}: {h:.17e} != {expected:.17e}",
                fields[0]
            );
        }
        if d.sample_size() <= 4096 {
            let data: Vec<_> = counts
                .iter()
                .enumerate()
                .flat_map(|(b, &n)| std::iter::repeat_n(b as u8, n))
                .collect();
            assert_eq!(h.to_bits(), tsallis_entropy(&data, q).unwrap().to_bits());
        }
        checked += 1;
    }
    assert_eq!(checked + skipped, 532);
    assert!(checked >= 288);
}

#[test]
fn near_one_preserves_parameter_and_huge_orders_preserve_subnormals() {
    let d = Distribution::from_bytes(b"AAAB");
    let h = shannon_distribution(&d);
    let below = tsallis_entropy_distribution(&d, 1.0 - 1e-8).unwrap();
    let above = tsallis_entropy_distribution(&d, 1.0 + 1e-8).unwrap();
    assert!(below > h && h > above);
    assert!((below - h).abs() < 1e-7 && (above - h).abs() < 1e-7);
    for q in [1.0_f64.next_down(), 1.0_f64.next_up()] {
        close(tsallis_entropy_distribution(&d, q).unwrap(), h);
    }
    let extreme = distribution(&[usize::MAX - 1, 1]);
    let n = usize::MAX as f64;
    let expected_two = (2.0 / n) * (1.0 - 1.0 / n) / LN_2;
    let actual_two = tsallis_entropy_distribution(&extreme, 2.0).unwrap();
    assert!((actual_two / expected_two - 1.0).abs() < 1e-9);
    for d in [&d, &extreme] {
        let tiny = tsallis_entropy_distribution(d, f64::MAX).unwrap();
        assert!(tiny.is_subnormal() && tiny > 0.0);
        assert!((tiny / ((1.0 / f64::MAX) / LN_2) - 1.0).abs() < 1e-12);
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        rng_seed: RngSeed::Fixed(0x5453_414c),
        ..ProptestConfig::default()
    })]

    #[test]
    fn permutation_relabeling_repetition_and_api_agreement(
        data in prop::collection::vec(any::<u8>(), 0..1025), mask in any::<u8>(),
        repeats in 1usize..8, q in prop::sample::select(ORDERS.to_vec()),
    ) {
        let h = tsallis_entropy(&data, q).unwrap();
        prop_assert_eq!(h.to_bits(), tsallis_entropy(&data, q).unwrap().to_bits());
        prop_assert_eq!(h.to_bits(), tsallis_entropy_distribution(&Distribution::from_bytes(&data), q).unwrap().to_bits());
        let mut sorted = data.clone();
        sorted.sort_unstable();
        prop_assert_eq!(h.to_bits(), tsallis_entropy(&sorted, q).unwrap().to_bits());
        let relabeled: Vec<_> = data.iter().map(|b| b^mask).collect();
        prop_assert!((h-tsallis_entropy(&relabeled, q).unwrap()).abs() <= TOL);
        prop_assert!((h-tsallis_entropy(&data.repeat(repeats), q).unwrap()).abs() <= TOL);
    }

    #[test]
    fn wide_counts_bounds_ordering_and_exact_different_pair_probability(
        entries in prop::collection::vec((any::<usize>(), 0u32..usize::BITS), 1..257),
        dominant in any::<bool>(),
        a_bits in 0u64..=0x7fef_ffff_ffff_ffff, b_bits in 0u64..=0x7fef_ffff_ffff_ffff,
    ) {
        let mut counts: Vec<_> = entries.iter().map(|&(c,s)| (c>>s)/512).collect();
        if dominant { counts[0] = usize::MAX-counts[1..].iter().sum::<usize>(); }
        let d = distribution(&counts);
        let k = d.support_size();
        let mut previous = (k as f64-1.0).max(0.0)/LN_2;
        let near_zero=tsallis_entropy_distribution(&d,f64::from_bits(1)).unwrap();
        prop_assert!((near_zero-previous).abs()<=TOL);
        for &q in ORDERS {
            let h = tsallis_entropy_distribution(&d, q).unwrap();
            let upper = if k<=1 { 0.0 } else if q==1.0 { (k as f64).log2() }
                else { -((1.0-q)*(k as f64).ln()).exp_m1()/(q-1.0)/LN_2 };
            prop_assert!(h.is_finite() && h>=0.0 && h<=upper+TOL, "q={q}, h={h}, upper={upper}");
            prop_assert!(h<=previous+TOL, "q={q}, h={h}, previous={previous}");
            if k>1 { prop_assert!(h>0.0); }
            previous=h;
        }
        let a=f64::from_bits(a_bits.min(b_bits));
        let b=f64::from_bits(a_bits.max(b_bits));
        prop_assert!(tsallis_entropy_distribution(&d,a).unwrap()+TOL >= tsallis_entropy_distribution(&d,b).unwrap());
        if !d.is_empty() {
            let n=d.sample_size() as u128;
            let same: u128=counts.iter().map(|&c| (c as u128)*(c as u128)).sum();
            let different=n*n-same;
            let expected=different as f64/(n*n) as f64/LN_2;
            let actual=tsallis_entropy_distribution(&d,2.0).unwrap();
            prop_assert!((actual-expected).abs()<=TOL);
            if different>0 { prop_assert!((actual/expected-1.0).abs()<1e-8); }
        }
    }

    #[test]
    fn renyi_power_sum_relation(
        counts in prop::collection::vec(0usize..1_000_001, 1..257),
        q in prop::sample::select(vec![0.0, 0.1, 0.5, 0.9, 1.0-1e-8, 1.0+1e-8, 1.1, 2.0, 16.0]),
    ) {
        let d=distribution(&counts);
        let r=renyi_entropy_distribution(&d,q).unwrap();
        let expected=((1.0-q)*r*LN_2).exp_m1()/(1.0-q)/LN_2;
        prop_assert!((tsallis_entropy_distribution(&d,q).unwrap()-expected).abs()<=TOL);
    }

    #[test]
    fn independent_product_composition(
        left in prop::collection::vec(1usize..101, 1..17),
        right in prop::collection::vec(1usize..101, 1..17),
        q in prop::sample::select(vec![0.0, 0.5, 1.0, 1.0+1e-8, 2.0, 4.0]),
    ) {
        let product: Vec<_>=left.iter().flat_map(|a| right.iter().map(move |b| a*b)).collect();
        let x=tsallis_entropy_distribution(&distribution(&left),q).unwrap();
        let y=tsallis_entropy_distribution(&distribution(&right),q).unwrap();
        let joint=tsallis_entropy_distribution(&distribution(&product),q).unwrap();
        prop_assert!((joint-(x+y+(1.0-q)*LN_2*x*y)).abs()<=TOL);
    }
}
