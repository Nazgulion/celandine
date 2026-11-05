//! Profile values, finite caps, and stateful lazy traversal.
use celandine::entropy::{block_entropy, block_entropy_profile as profile};
use proptest::{prelude::*, test_runner::RngSeed};
use std::collections::VecDeque;

fn close(a: f64, b: f64) {
    assert!(a.is_finite() && (a - b).abs() <= 1e-12, "{a} vs {b}");
}
fn same(a: Option<f64>, b: Option<f64>) {
    match (a, b) {
        (Some(a), Some(b)) => close(a, b),
        (None, None) => {}
        other => panic!("{other:?}"),
    }
}

#[test]
fn worked_profiles_and_limits() {
    for (data, expected) in [
        (b"".as_slice(), vec![]),
        (b"A", vec![0.0]),
        (b"AAAA", vec![0.0; 4]),
        (b"AABB", vec![1.0, 3.0_f64.log2(), 1.0, 0.0]),
        (b"ABCD", vec![2.0, 3.0_f64.log2(), 1.0, 0.0]),
        (
            b"ABABA",
            vec![0.9709505944546686, 1.0, 0.9182958340544896, 1.0, 0.0],
        ),
    ] {
        for maximum in [0, 1, 2, data.len(), data.len() + 1, usize::MAX] {
            let actual: Vec<_> = profile(data, maximum).collect();
            assert_eq!(actual.len(), maximum.min(data.len()));
            for (&a, &b) in actual.iter().zip(&expected) {
                close(a, b);
            }
            let reverse: Vec<_> = profile(data, maximum).rev().collect();
            assert_eq!(reverse, actual.into_iter().rev().collect::<Vec<_>>());
        }
    }
    let n_and_h: Vec<_> = profile(b"AABB", 4).enumerate().rev().collect();
    assert_eq!(n_and_h[0], (3, 0.0));
    let mut p = profile(b"AABB", usize::MAX);
    assert_eq!(p.next(), Some(1.0));
    assert_eq!(p.next_back(), Some(0.0));
    assert_eq!(p.clone().count(), 2);
    assert_eq!(p.clone().last(), Some(1.0));
    same(p.nth(1), Some(1.0));
    assert_eq!(p.next(), None);
    assert_eq!(p.next_back(), None);
    for skip in [4, usize::MAX] {
        let mut p = profile(b"AABB", 4);
        assert_eq!(p.nth(skip), None);
        assert_eq!(p.next_back(), None);
        let mut p = profile(b"AABB", 4);
        assert_eq!(p.nth_back(skip), None);
        assert_eq!(p.next(), None);
    }
}

#[test]
fn independent_high_precision_profiles() {
    let mut count = 0;
    for line in include_str!("data/block_profile.tsv")
        .lines()
        .filter(|s| !s.starts_with('#'))
    {
        let f: Vec<_> = line.split('\t').collect();
        let data: Vec<_> = if f[0] == "-" {
            vec![]
        } else {
            f[0].as_bytes()
                .chunks_exact(2)
                .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
                .collect()
        };
        let maximum = if f[1] == "max" {
            usize::MAX
        } else {
            f[1].parse().unwrap()
        };
        let expected: Vec<f64> = if f[2] == "-" {
            vec![]
        } else {
            f[2].split(',').map(|h| h.parse().unwrap()).collect()
        };
        let actual: Vec<_> = profile(&data, maximum).collect();
        assert_eq!(actual.len(), expected.len());
        for (a, b) in actual.into_iter().zip(expected) {
            close(a, b);
        }
        count += 1;
    }
    assert_eq!(count, 1831);
}

proptest! {
    #![proptest_config(ProptestConfig{cases:256,rng_seed:RngSeed::Fixed(20261105),..ProptestConfig::default()})]
    #[test]
    fn scalar_agreement_and_stateful_traversal(
        data in prop::collection::vec(0u8..8,0..49),
        maximum in prop_oneof![0usize..55,Just(usize::MAX)],
        actions in prop::collection::vec((0u8..8,prop_oneof![0usize..55,Just(usize::MAX)]),0..40),
    ) {
        let expected:Vec<_>=(1..=data.len().min(maximum)).map(|n|block_entropy(&data,n).unwrap()).collect();
        prop_assert_eq!(profile(&data,maximum).map(f64::to_bits).collect::<Vec<_>>(),expected.iter().map(|h|h.to_bits()).collect::<Vec<_>>());
        let mut model:VecDeque<_>=expected.into();
        let mut iter=profile(&data,maximum);
        for (op,skip) in actions {
            prop_assert_eq!(iter.len(),model.len());
            prop_assert_eq!(iter.size_hint(),(model.len(),Some(model.len())));
            match op {
                0=>same(iter.next(),model.pop_front()),
                1=>same(iter.next_back(),model.pop_back()),
                2|3=> {
                    let expected=if skip>=model.len(){model.clear();None}else{
                        for _ in 0..skip {if op==2 {model.pop_front();} else {model.pop_back();}}
                        if op==2 {model.pop_front()} else {model.pop_back()}
                    };
                    same(if op==2 {iter.nth(skip)}else{iter.nth_back(skip)},expected);
                },
                4=>prop_assert_eq!(iter.clone().collect::<Vec<_>>(),model.iter().copied().collect::<Vec<_>>()),
                5=>prop_assert_eq!(iter.clone().count(),model.len()),
                6=>same(iter.clone().last(),model.back().copied()),
                _=>prop_assert_eq!(iter.clone().rev().collect::<Vec<_>>(),model.iter().rev().copied().collect::<Vec<_>>()),
            }
        }
        prop_assert_eq!(iter.len(),model.len());
        prop_assert_eq!(iter.collect::<Vec<_>>(),model.into_iter().collect::<Vec<_>>());
    }
}
