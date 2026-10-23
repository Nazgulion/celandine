//! Independent numerical references and laws for Jensen-Shannon comparisons.
use celandine::distribution::{ByteHistogram, Distribution};
use celandine::divergence::{
    EmptyDistribution, jensen_shannon_distance as distance,
    jensen_shannon_distance_distribution as distance_distribution,
    jensen_shannon_divergence as divergence,
    jensen_shannon_divergence_distribution as divergence_distribution,
    total_variation_distance_distribution,
};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;

fn distribution(counts: &[usize]) -> Distribution {
    let mut table = [0; 256];
    table[..counts.len()].copy_from_slice(counts);
    Distribution::from_counts(ByteHistogram::try_from_counts(table).unwrap())
}

fn pairwise_sum(values: &[f64]) -> f64 {
    match values.len() {
        0 => 0.0,
        1 => values[0],
        n => pairwise_sum(&values[..n / 2]) + pairwise_sum(&values[n / 2..]),
    }
}

#[test]
fn canonical_values_equal_weights_and_errors() {
    let j = divergence(b"A", b"AB").unwrap();
    assert!((j - 0.31127812445913283).abs() < 1e-14);
    assert!((distance(b"A", b"AB").unwrap() - 0.5579230452841438).abs() < 1e-14);
    assert!((divergence(&[b'A'; 100], b"AB").unwrap() - j).abs() < 1e-14);
    for (p, q) in [(&b""[..], &b""[..]), (b"", b"AB"), (b"AB", b"")] {
        assert_eq!(divergence(p, q), Err(EmptyDistribution));
        assert_eq!(distance(p, q), Err(EmptyDistribution));
        let p = Distribution::from_bytes(p);
        let q = Distribution::from_bytes(q);
        assert_eq!(divergence_distribution(&p, &q), Err(EmptyDistribution));
        assert_eq!(distance_distribution(&p, &q), Err(EmptyDistribution));
    }
    assert_eq!(
        divergence(b"AABB", b"ABAB").unwrap().to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(
        distance(b"AAAB", b"AAAAAABB").unwrap().to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(divergence(b"AAA", b"B"), Ok(1.0));
    assert_eq!(distance(b"AAA", b"B"), Ok(1.0));
    // Divergence is not a metric; its square root is.
    assert!(divergence(b"A", b"B").unwrap() > 2.0 * j);
    assert!(distance(b"A", b"B").unwrap() < 2.0 * j.sqrt());
}

#[test]
fn extreme_counts_and_full_disjoint_support() {
    let n = usize::MAX;
    let p = distribution(&[n - 1, 1]);
    let q = distribution(&[n - 2, 1]);
    let j = divergence_distribution(&p, &q).unwrap();
    assert!(j > 0.0 && j.is_finite());
    assert_eq!(divergence_distribution(&q, &p), Ok(j));
    assert_eq!(distance_distribution(&p, &q), Ok(j.sqrt()));
    let p = distribution(&[n / 256; 256]);
    assert_eq!(
        divergence_distribution(&p, &distribution(&[1; 256])),
        Ok(0.0)
    );
    let mut left = [0; 256];
    let mut right = [0; 256];
    for i in 0..128 {
        left[i] = n / 128;
        right[128 + i] = i + 1;
    }
    assert_eq!(
        divergence_distribution(&distribution(&left), &distribution(&right)),
        Ok(1.0)
    );
}

#[test]
fn independent_160_digit_fixtures() {
    let mut checked = 0;
    let mut skipped = 0;
    for line in include_str!("data/jensen_shannon.tsv")
        .lines()
        .filter(|s| !s.starts_with('#'))
    {
        let f: Vec<_> = line.split('\t').collect();
        let parse = |s: &str| -> Vec<u128> {
            if s == "-" {
                vec![]
            } else {
                s.split(',').map(|c| c.parse().unwrap()).collect()
            }
        };
        let p = parse(f[1]);
        let q = parse(f[2]);
        if p.iter().sum::<u128>() > usize::MAX as u128
            || q.iter().sum::<u128>() > usize::MAX as u128
        {
            skipped += 1;
            continue;
        }
        let p: Vec<_> = p.into_iter().map(|c| c as usize).collect();
        let q: Vec<_> = q.into_iter().map(|c| c as usize).collect();
        let pd = distribution(&p);
        let qd = distribution(&q);
        let j = divergence_distribution(&pd, &qd);
        let d = distance_distribution(&pd, &qd);
        assert_eq!(j, divergence_distribution(&qd, &pd));
        if f[3] == "error" {
            assert_eq!(j, Err(EmptyDistribution));
            assert_eq!(d, Err(EmptyDistribution));
        } else {
            for (actual, expected) in [(j.unwrap(), f[3]), (d.unwrap(), f[4])] {
                let expected: f64 = expected.parse().unwrap();
                assert!(actual.is_finite() && (0.0..=1.0 + 1e-14).contains(&actual));
                if expected == 0.0 {
                    assert_eq!(actual.to_bits(), 0.0_f64.to_bits());
                } else {
                    assert!(
                        (actual / expected - 1.0).abs() <= 1e-12,
                        "{}: {actual:e} vs {expected:e}",
                        f[0]
                    );
                }
            }
            assert_eq!(d.unwrap(), j.unwrap().sqrt());
        }
        if pd.sample_size() <= 4096 && qd.sample_size() <= 4096 {
            let bytes = |counts: &[usize]| {
                counts
                    .iter()
                    .enumerate()
                    .flat_map(|(b, &c)| std::iter::repeat_n(b as u8, c))
                    .collect::<Vec<_>>()
            };
            assert_eq!(divergence(&bytes(&p), &bytes(&q)), j);
            assert_eq!(distance(&bytes(&p), &bytes(&q)), d);
        }
        checked += 1;
    }
    assert_eq!(checked + skipped, 171);
    assert!(checked >= 64);
}

proptest! {
    #![proptest_config(ProptestConfig {cases:256, rng_seed:RngSeed::Fixed(0x2026_1023), ..ProptestConfig::default()})]
    #[test]
    fn mixture_entropy_reference_and_invariants(
        p in prop::collection::vec(any::<u8>(),1..513),
        q in prop::collection::vec(any::<u8>(),1..513),
        r in prop::collection::vec(any::<u8>(),1..513),
        mask in any::<u8>(), repeats in 1usize..5,
    ) {
        let j=divergence(&p,&q).unwrap();
        let d=distance(&p,&q).unwrap();
        prop_assert_eq!(divergence(&q,&p),Ok(j));
        prop_assert_eq!(distance(&q,&p),Ok(d));
        prop_assert_eq!(divergence(&p,&p).unwrap().to_bits(),0.0_f64.to_bits());
        prop_assert!(d <= distance(&p,&r).unwrap()+distance(&r,&q).unwrap()+1e-14);
        // Independent entropy identity on well-conditioned, small sample counts.
        // Pairwise reference summation avoids the ordinary sum's accumulated
        // rounding (retained in jensen_shannon.proptest-regressions).
        let entropy = |v: &[f64]| pairwise_sum(&v.iter().filter(|&&v| v>0.0).map(|v| -v*v.log2()).collect::<Vec<_>>());
        let probabilities = |x: &[u8]| (0..=255).map(|b| x.iter().filter(|&&x| x==b).count() as f64 / x.len() as f64).collect::<Vec<_>>();
        let a=probabilities(&p); let b=probabilities(&q);
        let mixture: Vec<_>=a.iter().zip(&b).map(|(a,b)| (a+b)/2.0).collect();
        let reference=entropy(&mixture)-(entropy(&a)+entropy(&b))/2.0;
        prop_assert!((j-reference).abs() < 2e-14);
        let relabel=|x:&[u8]| x.iter().map(|b| b^mask).collect::<Vec<_>>();
        prop_assert!((j-divergence(&relabel(&p),&relabel(&q)).unwrap()).abs()<1e-14);
        let mut reversed=p.clone(); reversed.reverse();
        prop_assert_eq!(divergence(&reversed,&q),Ok(j));
        prop_assert!((divergence(&p.repeat(repeats),&q).unwrap()-j).abs()<1e-14);
    }

    #[test]
    fn wide_count_bounds_and_shared_label_symmetry(
        pairs in prop::collection::vec((any::<usize>(),any::<usize>()),1..257),
    ) {
        let mut p: Vec<_>=pairs.iter().map(|&(a,_)| a/512).collect();
        let mut q: Vec<_>=pairs.iter().map(|&(_,b)| b/512).collect();
        p[0]+=1; q[0]+=1;
        let pd=distribution(&p); let qd=distribution(&q);
        let j=divergence_distribution(&pd,&qd).unwrap();
        let tv=total_variation_distance_distribution(&pd,&qd).unwrap();
        prop_assert!(j.is_finite() && (0.0..=1.0+1e-14).contains(&j));
        prop_assert_eq!(divergence_distribution(&qd,&pd),Ok(j));
        prop_assert!(tv*tv/(2.0*std::f64::consts::LN_2) <= j+1e-14);
        prop_assert!(j <= tv+1e-14);
    }
}
