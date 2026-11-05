//! Direction, weighted groups, identities and independent high-precision values.
use celandine::{
    distribution::{LengthMismatch, joint_probabilities},
    entropy::{
        conditional_entropy as conditional, conditional_entropy_distribution as reuse,
        conditional_entropy_order, joint_entropy, shannon,
    },
    information::mutual_information,
};
use proptest::{prelude::*, test_runner::RngSeed};
fn close(actual: f64, expected: f64) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= 1e-12,
        "{actual:e} vs {expected:e}"
    );
}
#[test]
fn direction_weighting_and_boundaries() {
    for (x, y, expected) in [
        (b"".as_slice(), b"".as_slice(), 0.0_f64),
        (b"A", b"X", 0.0),
        (b"AAAA", b"WXYZ", 0.0),
        (b"ABCD", b"XXXX", 2.0),
        (b"ABCD", b"XXYY", 1.0),
        (b"XXYY", b"ABCD", 0.0),
        (b"AABB", b"XYXY", 1.0),
        (b"AABB", b"XXYY", 0.0),
        (b"AAAABC", b"XXXXYY", 1.0 / 3.0),
        (b"AAAB", b"XXYY", 0.5),
    ] {
        let d = joint_probabilities(x, y).unwrap();
        let value = reuse(&d);
        close(value, expected);
        if expected == 0.0 {
            assert_eq!(value.to_bits(), 0.0_f64.to_bits());
        }
        assert_eq!(value.to_bits(), conditional(x, y).unwrap().to_bits());
    }
    // Two deterministic Y groups: changing only the direction changes entropy.
    assert_ne!(conditional(b"ABCD", b"XXYY"), conditional(b"XXYY", b"ABCD"));
    for (x, y) in [
        (b"".as_slice(), b"X".as_slice()),
        (b"X", b""),
        (b"AB", b"X"),
    ] {
        assert_eq!(
            conditional(x, y),
            Err(LengthMismatch {
                x_len: x.len(),
                y_len: y.len()
            })
        );
    }
    let all: Vec<_> = (0..=255).collect();
    assert_eq!(
        conditional(&all, &all).unwrap().to_bits(),
        0.0_f64.to_bits()
    );
    close(conditional(&all, &[0; 256]).unwrap(), 8.0);
    let x: Vec<_> = (0..65536).map(|i| i as u8).collect();
    let y: Vec<_> = (0..65536).map(|i| (i >> 8) as u8).collect();
    close(conditional(&x, &y).unwrap(), 8.0);
    // Deterministic many-to-one Y(X), but several X values per Y.
    let y: Vec<_> = all.iter().map(|a| a / 2).collect();
    close(conditional(&all, &y).unwrap(), 1.0);
    assert_eq!(conditional(&y, &all).unwrap().to_bits(), 0.0_f64.to_bits());
}
#[test]
fn independent_high_precision_fixtures() {
    let mut rows = 0;
    let mut small = 0;
    for line in include_str!("data/conditional_pair.tsv")
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
                let n: usize = c[2].parse().unwrap();
                x.extend(std::iter::repeat_n(a, n));
                y.extend(std::iter::repeat_n(b, n));
            }
        }
        let expected: f64 = f[1].parse().unwrap();
        let actual = reuse(&joint_probabilities(&x, &y).unwrap());
        close(actual, expected);
        assert!(actual >= 0.0);
        if expected == 0.0 {
            assert_eq!(actual.to_bits(), 0.0_f64.to_bits());
        }
        if f[2] == "1" {
            assert!(
                actual > 0.0 && (actual - expected).abs() <= expected * 1e-12,
                "{actual:e} vs {expected:e}"
            );
            small += 1;
        }
        assert_eq!(actual.to_bits(), conditional(&x, &y).unwrap().to_bits());
        rows += 1;
    }
    assert_eq!(rows, 1397);
    assert!(small > 0);
}
proptest! {
    #![proptest_config(ProptestConfig{cases:256,rng_seed:RngSeed::Fixed(0x434f_4e44),..ProptestConfig::default()})]
    #[test]
    fn identities_and_invariants(pairs in prop::collection::vec((0u8..16,0u8..16),0..257), mask_x in any::<u8>(), mask_y in any::<u8>()) {
        let (x,y):(Vec<_>,Vec<_>)=pairs.into_iter().unzip();let value=conditional(&x,&y).unwrap();
        prop_assert!(value>=0.0 && value<=shannon(&x)+1e-12);
        close(value+shannon(&y),joint_entropy(&x,&y).unwrap());
        close(value+mutual_information(&x,&y).unwrap(),shannon(&x));
        close(value-conditional(&y,&x).unwrap(),shannon(&x)-shannon(&y));
        prop_assert_eq!(conditional(&x,&x).unwrap().to_bits(),0.0_f64.to_bits());
        close(conditional(&x,&vec![0;x.len()]).unwrap(),shannon(&x));
        let xr:Vec<_>=x.iter().rev().copied().collect();let yr:Vec<_>=y.iter().rev().copied().collect();
        prop_assert_eq!(value.to_bits(),conditional(&xr,&yr).unwrap().to_bits());
        let xn:Vec<_>=x.iter().map(|a|a^mask_x).collect();let yn:Vec<_>=y.iter().map(|a|a^mask_y).collect();
        close(value,conditional(&xn,&yn).unwrap());
        close(value,conditional(&[x.as_slice(),&x].concat(),&[y.as_slice(),&y].concat()).unwrap());
    }
    #[test]
    fn first_order_uses_the_same_aligned_observations(data in prop::collection::vec(0u8..8,2..257)) {
        close(conditional_entropy_order(&data,1),conditional(&data[1..],&data[..data.len()-1]).unwrap());
    }
}
