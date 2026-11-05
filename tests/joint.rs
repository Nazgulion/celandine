//! Aligned pair counts, marginals, entropy identities and independent references.
use celandine::{
    distribution::{
        ByteHistogram, JointDistribution, LengthMismatch, joint_counts, joint_probabilities,
    },
    entropy::{joint_entropy, joint_entropy_distribution, shannon},
};
use proptest::{prelude::*, test_runner::RngSeed};
fn close(a: f64, b: f64) {
    assert!(a.is_finite() && (a - b).abs() <= 1e-12, "{a} vs {b}");
}
#[test]
fn boundaries_and_pairing() {
    for (x, y, h) in [
        (b"".as_slice(), b"".as_slice(), 0.0_f64),
        (b"A", b"X", 0.0),
        (b"AAAA", b"XXXX", 0.0),
        (b"AABB", b"XYXY", 2.0),
        (b"AABB", b"XXYY", 1.0),
    ] {
        let d = joint_probabilities(x, y).unwrap();
        assert_eq!(d.sample_size(), x.len());
        assert_eq!(d.is_empty(), x.is_empty());
        assert_eq!(joint_entropy_distribution(&d).to_bits(), h.to_bits());
        assert_eq!(joint_entropy(x, y).unwrap().to_bits(), h.to_bits());
        assert_eq!(d.counts().marginal_x(), ByteHistogram::from_bytes(x));
        assert_eq!(d.counts().marginal_y(), ByteHistogram::from_bytes(y));
        assert_eq!(d.marginal_x().histogram(), &ByteHistogram::from_bytes(x));
        assert_eq!(d.marginal_y().histogram(), &ByteHistogram::from_bytes(y));
        assert_eq!(d.probability(255, 255).to_bits(), 0.0f64.to_bits());
        assert_eq!(d.clone(), d);
    }
    for (x, y) in [
        (b"".as_slice(), b"X".as_slice()),
        (b"X", b""),
        (b"AB", b"X"),
    ] {
        let error = LengthMismatch {
            x_len: x.len(),
            y_len: y.len(),
        };
        assert_eq!(joint_counts(x, y), Err(error));
        assert_eq!(joint_probabilities(x, y), Err(error));
        assert_eq!(joint_entropy(x, y), Err(error));
        assert!(error.to_string().contains("equal lengths"));
        let _: &dyn std::error::Error = &error;
    }
    let d = {
        let x = vec![0, 1];
        let y = vec![2, 3];
        joint_probabilities(&x, &y).unwrap()
    };
    assert_eq!(
        d.counts().counts().collect::<Vec<_>>(),
        vec![((0, 2), 1), ((1, 3), 1)]
    );
    assert_eq!(
        d.probabilities().collect::<Vec<_>>(),
        vec![((0, 2), 0.5), ((1, 3), 0.5)]
    );
}
#[test]
fn full_pair_alphabet_and_rare_pair() {
    let x: Vec<_> = (0..65536).map(|i| (i / 256) as u8).collect();
    let y: Vec<_> = (0..65536).map(|i| i as u8).collect();
    let d = joint_probabilities(&x, &y).unwrap();
    assert_eq!(d.support_size(), 65536);
    assert_eq!(joint_entropy_distribution(&d), 16.0);
    assert!(d.counts().counts().all(|(_, c)| c == 1));
    assert!(
        d.marginal_x()
            .histogram()
            .counts()
            .iter()
            .all(|&c| c == 256)
    );
    assert!(
        d.marginal_y()
            .histogram()
            .counts()
            .iter()
            .all(|&c| c == 256)
    );
    let mut x = vec![0; 1 << 20];
    let y = x.clone();
    x[0] = 1;
    let p = 1.0 / (1 << 20) as f64;
    let expected = -p * p.log2() - (1.0 - p) * (1.0 - p).log2();
    close(joint_entropy(&x, &y).unwrap(), expected);
}
fn hex(s: &str) -> Vec<u8> {
    if s == "-" {
        return vec![];
    }
    s.as_bytes()
        .chunks_exact(2)
        .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
        .collect()
}
#[test]
fn independent_reference_fixtures() {
    let mut rows = 0;
    for line in include_str!("data/joint.tsv")
        .lines()
        .filter(|line| !line.starts_with('#'))
    {
        let f: Vec<_> = line.split('\t').collect();
        let x = hex(f[0]);
        let y = hex(f[1]);
        if f[2] == "error" {
            assert_eq!(
                joint_entropy(&x, &y),
                Err(LengthMismatch {
                    x_len: x.len(),
                    y_len: y.len()
                })
            );
            rows += 1;
            continue;
        }
        let expected: Vec<((u8, u8), usize)> = if f[2] == "-" {
            vec![]
        } else {
            f[2].split(',')
                .map(|item| {
                    let c: Vec<_> = item.split(':').collect();
                    (
                        (c[0].parse().unwrap(), c[1].parse().unwrap()),
                        c[2].parse().unwrap(),
                    )
                })
                .collect()
        };
        let c = joint_counts(&x, &y).unwrap();
        assert_eq!(c.counts().collect::<Vec<_>>(), expected);
        assert_eq!(c.total(), x.len());
        assert_eq!(c.support_size(), expected.len());
        assert_eq!(c.marginal_x(), ByteHistogram::from_bytes(&x));
        assert_eq!(c.marginal_y(), ByteHistogram::from_bytes(&y));
        let d = JointDistribution::from_counts(c);
        for ((a, b), count) in expected {
            assert_eq!(d.counts().count(a, b), count);
            close(d.probability(a, b), count as f64 / x.len() as f64);
        }
        close(joint_entropy_distribution(&d), f[3].parse().unwrap());
        assert_eq!(
            joint_entropy(&x, &y).unwrap().to_bits(),
            joint_entropy_distribution(&d).to_bits()
        );
        rows += 1;
    }
    assert_eq!(rows, 1387);
}
proptest! {
    #![proptest_config(ProptestConfig{cases:256,rng_seed:RngSeed::Fixed(20261105),..ProptestConfig::default()})]
    #[test]
    fn identities(pairs in prop::collection::vec((any::<u8>(),any::<u8>()),0..257)) {
        let (x,y):(Vec<_>,Vec<_>)=pairs.iter().copied().unzip();
        let d=joint_probabilities(&x,&y).unwrap();let h=joint_entropy_distribution(&d);
        prop_assert_eq!(d.counts().marginal_x(),ByteHistogram::from_bytes(&x));
        prop_assert_eq!(d.counts().marginal_y(),ByteHistogram::from_bytes(&y));
        prop_assert_eq!(d.counts().counts().map(|(_,c)|c).sum::<usize>(),x.len());
        let entries:Vec<_>=d.counts().counts().collect();
        prop_assert!(entries.windows(2).all(|p|p[0].0<p[1].0));
        close(d.probabilities().map(|(_,p)|p).sum(),if x.is_empty(){0.0}else{1.0});
        close(h,joint_entropy(&y,&x).unwrap());close(joint_entropy(&x,&x).unwrap(),shannon(&x));
        let xr:Vec<_>=x.iter().rev().copied().collect();let yr:Vec<_>=y.iter().rev().copied().collect();
        prop_assert_eq!(joint_counts(&xr,&yr).unwrap(),d.counts().clone());
        let renamed:Vec<_>=x.iter().map(|a|a.wrapping_add(17)).collect();close(h,joint_entropy(&renamed,&y).unwrap());
        close(h,joint_entropy(&[x.as_slice(),&x].concat(),&[y.as_slice(),&y].concat()).unwrap());
        prop_assert!(h>=shannon(&x).max(shannon(&y))-1e-12 && h<=shannon(&x)+shannon(&y)+1e-12);
        prop_assert!((0.0..=16.0+1e-12).contains(&h));
        prop_assert!(h<=(d.support_size().max(1) as f64).log2()+1e-12);
    }
}
