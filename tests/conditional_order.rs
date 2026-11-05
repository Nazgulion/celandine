//! Matched context/target sampling, independent values and conditional bounds.
use celandine::distribution::ngram_probabilities;
use celandine::entropy::{
    block_entropy, conditional_entropy_order as entropy,
    conditional_entropy_order_distribution as reuse, shannon,
};
use proptest::{prelude::*, test_runner::RngSeed};
use std::collections::HashMap;

fn close(actual: f64, expected: f64) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= 1e-12,
        "{actual} vs {expected}"
    );
}

// Count prefixes and joint observations independently, on matching positions.
fn chain_rule(data: &[u8], order: usize) -> f64 {
    let mut joint = HashMap::new();
    let mut context = HashMap::new();
    for t in order..data.len() {
        *joint.entry(&data[t - order..=t]).or_insert(0usize) += 1;
        *context.entry(&data[t - order..t]).or_insert(0usize) += 1;
    }
    let h = |counts: HashMap<&[u8], usize>| {
        let total = counts.values().sum::<usize>() as f64;
        counts
            .values()
            .map(|&c| {
                let p = c as f64 / total;
                -p * p.log2()
            })
            .sum::<f64>()
    };
    h(joint) - h(context)
}

#[test]
fn worked_examples_and_boundary_marginals() {
    assert_eq!(entropy(b"ABABAB", 0), 1.0);
    assert_eq!(entropy(b"ABABAB", 1).to_bits(), 0.0_f64.to_bits());
    assert_eq!(entropy(b"AAB", 1), 1.0);
    close(entropy(b"AABA", 1), 2.0 / 3.0);
    // Whole-sample block differences need not be conditional entropies.
    let unmatched = block_entropy(b"ABABAB", 2).unwrap() - block_entropy(b"ABABAB", 1).unwrap();
    assert!(unmatched < 0.0);
    // Changing target sets can increase the estimate with order.
    assert!(entropy(b"AAB", 1) > entropy(b"AAB", 0));
    assert_ne!(entropy(b"AAB", 1), entropy(b"BAA", 1));
}

#[test]
fn empty_extreme_and_deterministic_cases() {
    for data in [b"".as_slice(), b"A", b"AAAA", b"ABABAB", b"ABCDEF"] {
        for order in [
            data.len().saturating_sub(1),
            data.len(),
            data.len() + 1,
            usize::MAX,
        ] {
            assert_eq!(entropy(data, order).to_bits(), 0.0_f64.to_bits());
        }
        for n in [1, 2, 3, 6, usize::MAX] {
            let d = ngram_probabilities(data, n).unwrap();
            close(reuse(&d), entropy(data, n - 1));
            if d.is_empty() {
                assert_eq!(reuse(&d).to_bits(), 0.0_f64.to_bits());
            }
        }
    }
    for order in 0..12 {
        assert_eq!(entropy(b"AAAAAAAA", order).to_bits(), 0.0_f64.to_bits());
    }
}

#[test]
fn all_successors_and_rare_transition() {
    // Context 0 has all 256 successors; other contexts are deterministic.
    let mut data = vec![0];
    for byte in 0..=255 {
        data.extend([byte, 0]);
    }
    let expected = (257.0_f64 * 257.0_f64.log2() - 2.0) / 512.0;
    close(entropy(&data, 1), expected);
    close(reuse(&ngram_probabilities(&data, 2).unwrap()), expected);
    close(
        reuse(&ngram_probabilities(&(0..=255).collect::<Vec<_>>(), 1).unwrap()),
        8.0,
    );
    let mut rare = vec![b'A'; 65536];
    rare.extend(b"BA");
    let p = 1.0 / 65536.0_f64;
    let expected = (-(1.0 - p) * (1.0 - p).log2() - p * p.log2()) * 65536.0 / 65537.0;
    let actual = entropy(&rare, 1);
    assert!((actual / expected - 1.0).abs() < 1e-12);
}

#[test]
fn independent_100_digit_fixtures() {
    let mut checked = 0;
    for row in include_str!("data/conditional_order.tsv")
        .lines()
        .filter(|s| !s.starts_with('#'))
    {
        let f: Vec<_> = row.split('\t').collect();
        let data: Vec<_> = if f[0] == "-" {
            vec![]
        } else {
            f[0].as_bytes()
                .chunks_exact(2)
                .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
                .collect()
        };
        let order = if f[1] == "max" {
            usize::MAX
        } else {
            f[1].parse().unwrap()
        };
        let expected = f[2].parse().unwrap();
        close(entropy(&data, order), expected);
        if let Some(n) = order.checked_add(1) {
            close(reuse(&ngram_probabilities(&data, n).unwrap()), expected);
        }
        checked += 1;
    }
    assert_eq!(checked, 2419);
}

proptest! {
    #![proptest_config(ProptestConfig { cases:256, rng_seed:RngSeed::Fixed(20261105), ..ProptestConfig::default() })]
    #[test]
    fn matched_chain_rule_bounds_and_renaming(raw in prop::collection::vec(any::<u8>(),0..256), order in prop_oneof![3 => 0usize..5, 1 => 5usize..280], alphabet in 1usize..257) {
        let data:Vec<_>=raw.iter().map(|&b|(usize::from(b)%alphabet) as u8).collect();
        let h=entropy(&data,order);
        prop_assert!(h.is_finite() && (0.0..=8.0+1e-12).contains(&h));
        prop_assert!(h<=shannon(&data[order.min(data.len())..])+1e-12);
        prop_assert!((h-chain_rule(&data,order)).abs()<=1e-12);
        prop_assert!((h-reuse(&ngram_probabilities(&data,order+1).unwrap())).abs()<=1e-12);
        let renamed:Vec<_>=data.iter().map(|b|b.wrapping_add(137)).collect();
        prop_assert!((h-entropy(&renamed,order)).abs()<=1e-12);
        prop_assert_eq!(entropy(&data,0),shannon(&data));
    }
}
