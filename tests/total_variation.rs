//! Total variation checked against rational fixtures, events, and metric laws.

use celandine::distribution::{ByteHistogram, Distribution};
use celandine::divergence::{
    EmptyDistribution, total_variation_distance as tv,
    total_variation_distance_distribution as tv_distribution,
};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;

fn distribution(counts: &[usize]) -> Distribution {
    let mut table = [0; 256];
    table[..counts.len()].copy_from_slice(counts);
    Distribution::from_counts(ByteHistogram::try_from_counts(table).unwrap())
}

#[test]
fn canonical_values_and_empty_errors() {
    for (p, q, expected) in [
        (&b"AAAB"[..], &b"AB"[..], 0.25),
        (b"AABB", b"ABAB", 0.0),
        (b"A", b"AAAA", 0.0),
        (b"AAA", b"BB", 1.0),
        (&[0, 255], &[0], 0.5),
    ] {
        assert_eq!(tv(p, q), Ok(expected));
        assert_eq!(tv(q, p), Ok(expected));
        if expected == 0.0 {
            assert_eq!(tv(p, q).unwrap().to_bits(), 0.0_f64.to_bits());
        }
    }
    for (p, q) in [(&b""[..], &b""[..]), (b"", b"A"), (b"A", b"")] {
        assert_eq!(tv(p, q), Err(EmptyDistribution));
        assert_eq!(
            tv_distribution(&Distribution::from_bytes(p), &Distribution::from_bytes(q)),
            Err(EmptyDistribution)
        );
    }
    let error: &dyn std::error::Error = &EmptyDistribution;
    assert_eq!(
        error.to_string(),
        "comparison requires two nonempty distributions"
    );
    assert!(error.source().is_none());
}

#[test]
fn large_counts_preserve_tiny_differences_and_do_not_overflow() {
    let n = usize::MAX;
    let p = distribution(&[n - 1, 1]);
    let q = distribution(&[n - 2, 1]);
    // 1/(n-1) - 1/n = 1/(n*(n-1)); far below the resolution near one.
    let expected = 1.0 / ((n as u128) * ((n - 1) as u128)) as f64;
    assert_eq!(tv_distribution(&p, &q), Ok(expected));
    assert_eq!(tv_distribution(&q, &p), Ok(expected));
    assert_eq!(
        tv_distribution(&distribution(&[n, 0]), &distribution(&[0, n])),
        Ok(1.0)
    );
    let uniform = distribution(&[n / 256; 256]);
    assert_eq!(tv_distribution(&uniform, &distribution(&[1; 256])), Ok(0.0));
}

#[test]
fn independent_exact_rational_fixtures() {
    let mut checked = 0;
    let mut skipped = 0;
    for line in include_str!("data/total_variation.tsv")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let fields: Vec<_> = line.split('\t').collect();
        let parse = |text: &str| -> Vec<u128> {
            if text == "-" {
                vec![]
            } else {
                text.split(',').map(|c| c.parse().unwrap()).collect()
            }
        };
        let p = parse(fields[1]);
        let q = parse(fields[2]);
        if p.iter().sum::<u128>() > usize::MAX as u128
            || q.iter().sum::<u128>() > usize::MAX as u128
        {
            skipped += 1;
            continue;
        }
        let p: Vec<_> = p.into_iter().map(|c| c as usize).collect();
        let q: Vec<_> = q.into_iter().map(|c| c as usize).collect();
        let actual = tv_distribution(&distribution(&p), &distribution(&q));
        assert_eq!(
            actual,
            tv_distribution(&distribution(&q), &distribution(&p))
        );
        if fields[3] == "error" {
            assert_eq!(actual, Err(EmptyDistribution));
        } else {
            let expected = fields[3].parse::<f64>().unwrap() / fields[4].parse::<f64>().unwrap();
            let actual = actual.unwrap();
            assert!((0.0..=1.0).contains(&actual));
            if expected == 0.0 {
                assert_eq!(actual.to_bits(), 0.0_f64.to_bits());
            } else {
                assert!(
                    (actual / expected - 1.0).abs() <= 1e-15,
                    "{}: {actual} vs {expected}",
                    fields[0]
                );
            }
        }
        checked += 1;
    }
    assert_eq!(checked + skipped, 148);
    assert!(checked >= 64);
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        rng_seed: RngSeed::Fixed(0x7076_2026),
        ..ProptestConfig::default()
    })]

    #[test]
    fn maximum_event_gap_defines_distance(
        p in prop::collection::vec(0u8..8, 1..65),
        q in prop::collection::vec(0u8..8, 1..65),
    ) {
        // Enumerate every event on eight symbols, counting membership directly.
        let expected = (0u16..256).map(|mask| {
            let left = p.iter().filter(|&&b| mask & (1 << b) != 0).count() as f64 / p.len() as f64;
            let right = q.iter().filter(|&&b| mask & (1 << b) != 0).count() as f64 / q.len() as f64;
            (left - right).abs()
        }).fold(0.0, f64::max);
        prop_assert!((tv(&p, &q).unwrap() - expected).abs() <= 1e-15);
    }

    #[test]
    fn metric_laws_and_invariance(
        p in prop::collection::vec(any::<u8>(), 1..513),
        q in prop::collection::vec(any::<u8>(), 1..513),
        r in prop::collection::vec(any::<u8>(), 1..513),
        mask in any::<u8>(), repeats in 1usize..5,
    ) {
        let d = tv(&p, &q).unwrap();
        prop_assert!((0.0..=1.0).contains(&d));
        prop_assert_eq!(tv(&q, &p), Ok(d));
        prop_assert_eq!(tv(&p, &p).unwrap().to_bits(), 0.0_f64.to_bits());
        prop_assert!(d <= tv(&p, &r).unwrap() + tv(&r, &q).unwrap() + 1e-15);
        let mut sorted = p.clone();
        sorted.sort_unstable();
        prop_assert_eq!(tv(&sorted, &q), Ok(d));
        let relabel = |x: &[u8]| x.iter().map(|b| b ^ mask).collect::<Vec<_>>();
        prop_assert_eq!(tv(&relabel(&p), &relabel(&q)), Ok(d));
        prop_assert!((tv(&p.repeat(repeats), &q).unwrap() - d).abs() <= 1e-15);
        prop_assert_eq!(tv_distribution(&Distribution::from_bytes(&p), &Distribution::from_bytes(&q)), Ok(d));
    }

    #[test]
    fn wide_counts_match_overlap_complement(
        pairs in prop::collection::vec((any::<usize>(), any::<usize>()), 1..257),
        dominant in any::<bool>(),
    ) {
        let mut p: Vec<_> = pairs.iter().map(|&(a, _)| a / 512).collect();
        let mut q: Vec<_> = pairs.iter().map(|&(_, b)| b / 512).collect();
        p[0] += 1;
        q[0] += 1;
        if dominant {
            p[0] = usize::MAX - p[1..].iter().sum::<usize>();
            q[0] = usize::MAX - q[1..].iter().sum::<usize>();
        }
        let n = p.iter().sum::<usize>() as u128;
        let m = q.iter().sum::<usize>() as u128;
        // Independent identity: TV = 1 - sum_i min(p_i,q_i), with exact complement.
        let overlap: u128 = p.iter().zip(&q).map(|(&a, &b)| ((a as u128)*m).min((b as u128)*n)).sum();
        let expected = (n*m - overlap) as f64 / (n*m) as f64;
        let d = tv_distribution(&distribution(&p), &distribution(&q)).unwrap();
        prop_assert_eq!(d, expected);
        prop_assert_eq!(tv_distribution(&distribution(&q), &distribution(&p)), Ok(d));
    }
}
