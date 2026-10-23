//! Exact finite-block counts, independent definitions, and iterator contracts.
use celandine::complexity::{subword_complexity as p, subword_complexity_profile as profile};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;

// Independent definition: count positions whose block has not appeared earlier.
fn reference(data: &[u8], n: usize) -> usize {
    if n > data.len() {
        return 0;
    }
    (0..=data.len() - n)
        .filter(|&i| !(0..i).any(|j| (0..n).all(|k| data[i + k] == data[j + k])))
        .count()
}

#[test]
fn canonical_empty_extreme_and_order_sensitive_cases() {
    for (data, expected) in [
        (&b""[..], &[1][..]),
        (b"A", &[1, 1]),
        (b"AAAA", &[1, 1, 1, 1, 1]),
        (b"ABABA", &[1, 2, 2, 2, 2, 1]),
        (b"ABCD", &[1, 4, 3, 2, 1]),
        (b"AABB", &[1, 2, 3, 2, 1]),
        (b"ABAB", &[1, 2, 2, 2, 1]),
        (&[0, 255, 0, 255], &[1, 2, 2, 2, 1]),
        ("éé".as_bytes(), &[1, 2, 2, 2, 1]),
    ] {
        assert_eq!(profile(data).collect::<Vec<_>>(), expected);
        for (n, &count) in expected.iter().enumerate() {
            assert_eq!(p(data, n), count);
        }
        assert_eq!(p(data, data.len() + 1), 0);
        assert_eq!(p(data, usize::MAX), 0);
    }
    // Repetition can introduce a boundary block (BA), unlike a bijective rename.
    assert_eq!(p(b"AB", 2), 1);
    assert_eq!(p(b"ABAB", 2), 2);
}

#[test]
fn profile_is_exact_double_ended_cloneable_and_fused() {
    let mut values = profile(b"AABB");
    assert_eq!(values.len(), 5);
    assert_eq!(values.size_hint(), (5, Some(5)));
    assert_eq!(values.next(), Some(1));
    assert_eq!(values.next_back(), Some(1));
    assert_eq!(values.clone().collect::<Vec<_>>(), [2, 3, 2]);
    assert_eq!(values.nth(1), Some(3));
    assert_eq!(values.len(), 1);
    assert_eq!(values.next_back(), Some(2));
    assert_eq!(values.next(), None);
    assert_eq!(values.next_back(), None);
    assert_eq!(values.len(), 0);
    assert_eq!(profile(b"AABB").nth_back(2), Some(3));
    assert_eq!(profile(b"AABB").nth(usize::MAX), None);
    assert_eq!(profile(b"AABB").nth_back(usize::MAX), None);
    assert_eq!(profile(b"").rev().collect::<Vec<_>>(), [1]);
    assert_eq!(profile(b"ABABA").take(3).collect::<Vec<_>>(), [1, 2, 2]);
}

#[test]
fn exhaustive_binary_inputs_match_position_definition() {
    for length in 0..=9 {
        for bits in 0..(1usize << length) {
            let data: Vec<_> = (0..length).map(|i| ((bits >> i) & 1) as u8).collect();
            for n in 0..=length + 2 {
                assert_eq!(p(&data, n), reference(&data, n));
            }
        }
    }
}

#[test]
fn independent_python_profiles() {
    let mut count = 0;
    for line in include_str!("data/subword.tsv")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let (hex, counts) = line.split_once('\t').unwrap();
        let data: Vec<_> = if hex == "-" {
            vec![]
        } else {
            (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect()
        };
        let expected: Vec<usize> = counts.split(',').map(|s| s.parse().unwrap()).collect();
        for (n, &value) in expected.iter().enumerate() {
            assert_eq!(p(&data, n), value);
        }
        assert_eq!(
            profile(&data).collect::<Vec<_>>(),
            expected[..data.len() + 1]
        );
        count += 1;
    }
    assert_eq!(count, 264);
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, rng_seed: RngSeed::Fixed(0x0053_5542_574f_5244), ..ProptestConfig::default() })]
    #[test]
    fn positional_reference_bounds_and_invariances(
        data in prop::collection::vec(any::<u8>(), 0..65),
        n in prop_oneof![0usize..70, Just(usize::MAX)], mask in any::<u8>(),
    ) {
        let count = p(&data,n);
        prop_assert_eq!(count, reference(&data,n));
        prop_assert_eq!(count, p(&data.iter().rev().copied().collect::<Vec<_>>(),n));
        prop_assert_eq!(count, p(&data.iter().map(|b| b ^ mask).collect::<Vec<_>>(),n));
        if n > 0 && n <= data.len() { prop_assert!((1..=data.len()-n+1).contains(&count)); }
        let expected: Vec<_> = (0..=data.len()).map(|n| reference(&data,n)).collect();
        prop_assert_eq!(profile(&data).collect::<Vec<_>>(), expected);
        prop_assert_eq!(p(&data,1), data.iter().copied().collect::<std::collections::BTreeSet<_>>().len());
    }
    #[test]
    fn extending_a_sample_cannot_remove_existing_blocks(
        data in prop::collection::vec(0u8..4, 0..65), tail in prop::collection::vec(0u8..4, 0..33), n in 0usize..70,
    ) {
        let mut extended = data.clone(); extended.extend(tail);
        prop_assert!(p(&extended,n) >= p(&data,n));
    }
}
