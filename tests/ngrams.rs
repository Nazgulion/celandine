//! Verify positional blocks, exact counts, empirical probabilities, and borrowing.

use celandine::distribution::{
    ByteHistogram, Distribution, NgramDistribution, ngram_counts, ngram_probabilities,
};
use celandine::transforms::{InvalidNgramLength, ngrams};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;

#[test]
fn overlapping_occurrences_and_reused_counts() {
    let blocks: Vec<_> = ngrams(b"ABABA", 2).unwrap().collect();
    assert_eq!(blocks, vec![b"AB", b"BA", b"AB", b"BA"]);
    let c = ngram_counts(b"ABABA", 2).unwrap();
    assert_eq!(c.ngram_len(), 2);
    assert_eq!(c.total(), 4);
    assert_eq!(c.support_size(), 2);
    assert!(!c.is_empty());
    assert_eq!(
        c.counts().collect::<Vec<_>>(),
        vec![(&b"AB"[..], 2), (&b"BA"[..], 2)]
    );
    for absent in [&b"AA"[..], b"", b"A", b"ABA"] {
        assert_eq!(c.count(absent), 0);
    }
    let d = NgramDistribution::from_counts(c);
    assert_eq!(d.sample_size(), 4);
    assert_eq!(d.support_size(), 2);
    assert_eq!(d.ngram_len(), 2);
    assert!(!d.is_empty());
    assert_eq!(
        d.probabilities().collect::<Vec<_>>(),
        vec![(&b"AB"[..], 0.5), (&b"BA"[..], 0.5)]
    );
    for absent in [&b"AA"[..], b"", b"A", b"ABA"] {
        assert_eq!(d.probability(absent).to_bits(), 0.0_f64.to_bits());
    }
    assert_eq!(d, ngram_probabilities(b"ABABA", 2).unwrap());
    let repeated = ngram_counts(b"AAAA", 2).unwrap();
    assert_eq!(repeated.total(), 3);
    assert_eq!(repeated.count(b"AA"), 3);
}

#[test]
fn invalid_empty_and_extreme_lengths() {
    for data in [&b""[..], b"A", b"AB"] {
        assert_eq!(ngrams(data, 0).unwrap_err(), InvalidNgramLength);
        assert_eq!(ngram_counts(data, 0).unwrap_err(), InvalidNgramLength);
        assert_eq!(
            ngram_probabilities(data, 0).unwrap_err(),
            InvalidNgramLength
        );
        for n in [data.len() + 1, usize::MAX] {
            assert_eq!(ngrams(data, n).unwrap().len(), 0);
            let d = ngram_probabilities(data, n).unwrap();
            assert!(d.is_empty());
            assert!(d.counts().is_empty());
            assert_eq!(d.ngram_len(), n);
            assert_eq!(d.sample_size(), 0);
            assert_eq!(d.support_size(), 0);
            assert_eq!(d.counts().counts().len(), 0);
            assert_eq!(d.probabilities().len(), 0);
            assert_eq!(d.probability(data).to_bits(), 0.0_f64.to_bits());
        }
        if !data.is_empty() {
            let d = ngram_probabilities(data, data.len()).unwrap();
            assert_eq!(d.sample_size(), 1);
            assert_eq!(d.probability(data), 1.0);
        }
    }
    assert_eq!(
        InvalidNgramLength.to_string(),
        "n-gram length must be greater than zero"
    );
}

#[test]
fn iteration_is_borrowed_exact_double_ended_and_fused() {
    let data = [0, 255, 128, 0, 255];
    let mut blocks = ngrams(&data, 2).unwrap();
    assert_eq!(blocks.len(), 4);
    assert_eq!(blocks.next().unwrap().as_ptr(), data.as_ptr());
    assert_eq!(blocks.next_back().unwrap().as_ptr(), data[3..].as_ptr());
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks.nth(1), Some(&data[2..4]));
    assert_eq!(blocks.len(), 0);
    assert_eq!(blocks.next(), None);
    assert_eq!(blocks.next_back(), None);
    let d = ngram_probabilities(&data, 2).unwrap();
    assert_eq!(
        d.probabilities().map(|(g, _)| g).collect::<Vec<_>>(),
        vec![&[0, 255][..], &[128, 0], &[255, 128]]
    );
    for (block, _) in d.counts().counts() {
        assert!((0..data.len()).any(|i| block.as_ptr() == data[i..].as_ptr()));
    }
}

#[test]
fn order_is_preserved_locally_and_utf8_is_not_decoded() {
    assert_eq!(
        ByteHistogram::from_bytes(b"AABB"),
        ByteHistogram::from_bytes(b"ABAB")
    );
    assert_ne!(
        ngram_counts(b"AABB", 2).unwrap(),
        ngram_counts(b"ABAB", 2).unwrap()
    );
    let text = "éé".as_bytes();
    let c = ngram_counts(text, 2).unwrap();
    assert_eq!(c.total(), 3);
    assert_eq!(c.count(&[0xc3, 0xa9]), 2);
    assert_eq!(c.count(&[0xa9, 0xc3]), 1); // Spans character boundaries.
}

#[test]
fn exhaustive_binary_strings_match_explicit_position_comparisons() {
    for length in 0..=7 {
        for bits in 0..(1usize << length) {
            let data: Vec<u8> = (0..length).map(|i| ((bits >> i) & 1) as u8).collect();
            for n in 1..=length + 2 {
                let d = ngram_probabilities(&data, n).unwrap();
                // Independent scalar definition: compare every byte at every valid start.
                let mut total = 0;
                let mut support = 0;
                for pattern in 0..(1usize << n) {
                    let block: Vec<u8> = (0..n).map(|i| ((pattern >> i) & 1) as u8).collect();
                    let expected = (0..length)
                        .filter(|&start| {
                            start + n <= length && (0..n).all(|j| data[start + j] == block[j])
                        })
                        .count();
                    assert_eq!(d.counts().count(&block), expected);
                    assert_eq!(
                        d.probability(&block),
                        expected as f64 / d.sample_size().max(1) as f64
                    );
                    total += expected;
                    support += usize::from(expected != 0);
                }
                assert_eq!(d.sample_size(), total);
                assert_eq!(d.support_size(), support);
            }
        }
    }
}

fn hex(s: &str) -> Vec<u8> {
    if s == "-" {
        return vec![];
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn independent_exact_python_fixtures() {
    let mut checked = 0;
    for line in include_str!("data/ngrams.tsv")
        .lines()
        .filter(|line| !line.starts_with('#'))
    {
        let fields: Vec<_> = line.split('\t').collect();
        let data = hex(fields[0]);
        let n = fields[1].parse().unwrap();
        let expected: Vec<_> = if fields[2] == "-" {
            vec![]
        } else {
            fields[2].split(',').map(hex).collect()
        };
        assert_eq!(ngrams(&data, n).unwrap().collect::<Vec<_>>(), expected);
        let d = ngram_probabilities(&data, n).unwrap();
        assert_eq!(d.sample_size(), expected.len());
        let entries: Vec<_> = if fields[3] == "-" {
            vec![]
        } else {
            fields[3].split(',').collect()
        };
        assert_eq!(d.support_size(), entries.len());
        for ((block, p), entry) in d.probabilities().zip(entries) {
            let parts: Vec<_> = entry.split(':').collect();
            assert_eq!(block, hex(parts[0]));
            assert_eq!(d.counts().count(block), parts[1].parse::<usize>().unwrap());
            let (numerator, denominator) = parts[2].split_once('/').unwrap();
            let expected = numerator.parse::<f64>().unwrap() / denominator.parse::<f64>().unwrap();
            assert!((p - expected).abs() <= 1e-12);
            assert_eq!(p.to_bits(), d.probability(block).to_bits());
        }
        checked += 1;
    }
    assert_eq!(checked, 433);
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        rng_seed: RngSeed::Fixed(0x4e47_5241_4d53),
        ..ProptestConfig::default()
    })]

    #[test]
    fn conservation_sorted_support_and_unigram_agreement(
        data in prop::collection::vec(any::<u8>(), 0..513), n in 1usize..530,
    ) {
        let d = ngram_probabilities(&data, n).unwrap();
        let expected = if n <= data.len() { data.len() - n + 1 } else { 0 };
        prop_assert_eq!(d.sample_size(), expected);
        prop_assert_eq!(d.counts().counts().map(|(_, c)| c).sum::<usize>(), expected);
        prop_assert!(d.support_size() <= expected);
        let mut previous = None;
        let mut mass = 0.0;
        for (g, p) in d.probabilities() {
            prop_assert_eq!(g.len(), n);
            prop_assert!(p > 0.0 && p <= 1.0);
            if let Some(last) = previous { prop_assert!(last < g); }
            previous = Some(g);
            mass += p;
        }
        let expected_mass = if expected > 0 { 1.0 } else { 0.0 };
        prop_assert!((mass - expected_mass).abs() <= 1e-12);
        let unigram = ngram_probabilities(&data, 1).unwrap();
        let bytes = Distribution::from_bytes(&data);
        prop_assert_eq!(unigram.support_size(), bytes.support_size());
        for b in 0..=255 {
            prop_assert_eq!(unigram.counts().count(&[b]), bytes.histogram().count(b));
            prop_assert_eq!(unigram.probability(&[b]).to_bits(), bytes.probability(b).to_bits());
        }
    }

    #[test]
    fn reversal_and_bijective_relabeling_preserve_corresponding_counts(
        data in prop::collection::vec(0u8..8, 0..257), n in 1usize..20, mask in any::<u8>(),
    ) {
        let reversed: Vec<_> = data.iter().rev().copied().collect();
        let renamed: Vec<_> = data.iter().map(|b| b ^ mask).collect();
        let original = ngram_counts(&data, n).unwrap();
        let rev = ngram_counts(&reversed, n).unwrap();
        let relabeled = ngram_counts(&renamed, n).unwrap();
        prop_assert_eq!(original.support_size(), rev.support_size());
        prop_assert_eq!(original.support_size(), relabeled.support_size());
        for (g, count) in original.counts() {
            let r: Vec<_> = g.iter().rev().copied().collect();
            let renamed: Vec<_> = g.iter().map(|b| b ^ mask).collect();
            prop_assert_eq!(rev.count(&r), count);
            prop_assert_eq!(relabeled.count(&renamed), count);
        }
    }
}
