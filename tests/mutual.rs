//! Mutual information identities, boundaries and independent numerical references.
use celandine::{
    distribution::{LengthMismatch, joint_probabilities},
    entropy::{joint_entropy, shannon},
    information::{mutual_information, mutual_information_distribution},
};
use proptest::{prelude::*, test_runner::RngSeed};
fn close(a: f64, b: f64) {
    assert!(a.is_finite() && (a - b).abs() <= 1e-12, "{a:e} vs {b:e}");
}
#[test]
fn worked_and_boundary_cases() {
    for (x, y, expected) in [
        (b"".as_slice(), b"".as_slice(), 0.0f64),
        (b"A", b"X", 0.0),
        (b"AAAA", b"WXYZ", 0.0),
        (b"AABB", b"XYXY", 0.0),
        (b"AABB", b"XXYY", 1.0),
        (b"ABCD", b"WXYZ", 2.0),
    ] {
        let d = joint_probabilities(x, y).unwrap();
        let value = mutual_information_distribution(&d);
        close(value, expected);
        if expected == 0.0 {
            assert_eq!(value.to_bits(), 0.0f64.to_bits());
        }
        assert_eq!(value.to_bits(), mutual_information(x, y).unwrap().to_bits());
    }
    for (x, y) in [
        (b"".as_slice(), b"X".as_slice()),
        (b"X", b""),
        (b"AB", b"X"),
    ] {
        assert_eq!(
            mutual_information(x, y),
            Err(LengthMismatch {
                x_len: x.len(),
                y_len: y.len()
            })
        );
    }
    let all: Vec<_> = (0..=255).collect();
    close(mutual_information(&all, &all).unwrap(), 8.0);
    let x: Vec<_> = (0..65536).map(|i| i as u8).collect();
    let y: Vec<_> = (0..65536).map(|i| (i >> 8) as u8).collect();
    assert_eq!(
        mutual_information(&x, &y).unwrap().to_bits(),
        0.0f64.to_bits()
    );
}
#[test]
fn independent_high_precision_fixtures() {
    let mut rows = 0;
    let mut tiny = 0;
    for line in include_str!("data/mutual.tsv")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let f: Vec<_> = line.split('\t').collect();
        let mut x = Vec::new();
        let mut y = Vec::new();
        if f[0] != "-" {
            for cell in f[0].split(',') {
                let c: Vec<_> = cell.split(':').collect();
                let a: u8 = c[0].parse().unwrap();
                let b: u8 = c[1].parse().unwrap();
                let count: usize = c[2].parse().unwrap();
                x.extend(std::iter::repeat_n(a, count));
                y.extend(std::iter::repeat_n(b, count));
            }
        }
        let expected: f64 = f[1].parse().unwrap();
        let d = joint_probabilities(&x, &y).unwrap();
        let actual = mutual_information_distribution(&d);
        close(actual, expected);
        assert!(actual >= 0.0);
        if expected == 0.0 {
            assert_eq!(actual.to_bits(), 0.0f64.to_bits());
        }
        if f[2] == "1" {
            assert!(
                (actual - expected).abs() <= expected * 1e-12,
                "{actual:e} vs {expected:e}"
            );
            tiny += 1;
        }
        assert_eq!(
            actual.to_bits(),
            mutual_information(&x, &y).unwrap().to_bits()
        );
        rows += 1;
    }
    assert_eq!(rows, 1389);
    assert!(tiny > 0);
}
proptest! {
    #![proptest_config(ProptestConfig{cases:256,rng_seed:RngSeed::Fixed(20261105),..ProptestConfig::default()})]
    #[test]
    fn identities(pairs in prop::collection::vec((0u8..16,0u8..16),0..257)) {
        let (x,y):(Vec<_>,Vec<_>)=pairs.into_iter().unzip();let value=mutual_information(&x,&y).unwrap();
        close(value,mutual_information(&y,&x).unwrap());close(value,shannon(&x)+shannon(&y)-joint_entropy(&x,&y).unwrap());
        close(mutual_information(&x,&x).unwrap(),shannon(&x));
        prop_assert!(value>=0.0 && value<=shannon(&x).min(shannon(&y))+1e-12);
        let xr:Vec<_>=x.iter().rev().copied().collect();let yr:Vec<_>=y.iter().rev().copied().collect();
        prop_assert_eq!(value.to_bits(),mutual_information(&xr,&yr).unwrap().to_bits());
        let renamed:Vec<_>=x.iter().map(|a|a.wrapping_add(241)).collect();close(value,mutual_information(&renamed,&y).unwrap());
        close(value,mutual_information(&[x.as_slice(),&x].concat(),&[y.as_slice(),&y].concat()).unwrap());
    }
}
