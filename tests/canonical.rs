//! Hand-verifiable cases, representation boundaries, and independent fixtures.

use celandine::distribution::{ByteHistogram, CountOverflow, Distribution};
use celandine::entropy::{shannon, shannon_distribution};

const TOLERANCE: f64 = 1e-12;

#[test]
fn canonical_entropies() {
    let cases: &[(&[u8], f64)] = &[
        (b"", 0.0),
        (b"A", 0.0),
        (b"AAAA", 0.0),
        (b"AB", 1.0),
        (b"ABAB", 1.0),
        (b"ABCD", 2.0),
        (b"ABCABCABC", 3.0_f64.log2()),
        (b"0123456789", 10.0_f64.log2()),
        (b"AAAB", 0.811_278_124_459_132_8),
    ];
    for &(input, expected) in cases {
        let actual = shannon(input);
        assert!(
            (actual - expected).abs() <= TOLERANCE,
            "{input:?}: {actual}"
        );
        assert_eq!(
            actual,
            shannon_distribution(&Distribution::from_bytes(input))
        );
    }
}

#[test]
fn empty_state_is_explicit_and_zero_is_positive() {
    let d = Distribution::from_bytes(b"");
    assert!(d.is_empty());
    assert!(d.histogram().is_empty());
    assert_eq!(d.sample_size(), 0);
    assert_eq!(d.support_size(), 0);
    assert_eq!(d.probabilities().len(), 256);
    assert!(d.probabilities().all(|p| p == 0.0));
    assert_eq!(d.probability(255), 0.0);
    assert_eq!(shannon_distribution(&d).to_bits(), 0.0_f64.to_bits());
    for symbol in 0..=255 {
        assert_eq!(shannon(&[symbol; 32]).to_bits(), 0.0_f64.to_bits());
    }
}

#[test]
fn counts_probabilities_and_support_are_distinct() {
    let counts = ByteHistogram::from_bytes(&[0, 0, 128, 255]);
    assert_eq!(counts.total(), 4);
    assert_eq!(counts.support_size(), 3);
    assert_eq!(counts.count(0), 2);
    assert_eq!(counts.count(128), 1);
    assert_eq!(counts.count(255), 1);
    assert_eq!(counts.count(1), 0);
    assert_eq!(counts.counts().iter().sum::<usize>(), 4);
    let d = Distribution::from_counts(counts);
    assert_eq!(d.sample_size(), 4);
    assert_eq!(d.support_size(), 3);
    assert_eq!(d.probability(0), 0.5);
    assert_eq!(d.probability(128), 0.25);
    assert_eq!(d.probability(255), 0.25);
    assert_eq!(d.probability(1), 0.0);
    for (symbol, probability) in d.probabilities().enumerate() {
        assert_eq!(probability, d.probability(symbol as u8));
    }
    assert_eq!(shannon_distribution(&d), 1.5);
}

#[test]
fn every_uniform_byte_support() {
    for k in 1..=256 {
        let input: Vec<u8> = (0..k).map(|i| i as u8).collect();
        assert!((shannon(&input) - (k as f64).log2()).abs() <= TOLERANCE);
    }
    let full_alphabet: Vec<u8> = (0..=255).collect();
    assert_eq!(shannon(&full_alphabet), 8.0);
}

#[test]
fn checked_counts_reject_overflow_and_accept_maximum_total() {
    let mut counts = [0; 256];
    counts[0] = usize::MAX;
    let histogram = ByteHistogram::try_from_counts(counts).unwrap();
    assert_eq!(histogram.total(), usize::MAX);
    assert_eq!(
        shannon_distribution(&Distribution::from_counts(histogram)),
        0.0
    );
    counts[255] = 1;
    assert_eq!(ByteHistogram::try_from_counts(counts), Err(CountOverflow));
    counts[0] -= 1;
    let d = Distribution::from_counts(ByteHistogram::try_from_counts(counts).unwrap());
    assert_eq!(d.sample_size(), usize::MAX);
    assert_eq!(d.support_size(), 2);
    assert!(d.probability(255) > 0.0);
    assert!(shannon_distribution(&d).is_finite());
    assert!(shannon_distribution(&d) > 0.0);
    assert_eq!(ByteHistogram::try_from_counts([0; 256]).unwrap().total(), 0);
}

#[test]
fn independent_python_reference_vectors() {
    let mut cases = 0;
    for line in include_str!("data/shannon.tsv").lines() {
        if line.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 3);
        let input: Vec<u8> = fields[1]
            .as_bytes()
            .chunks_exact(2)
            .map(|hex| u8::from_str_radix(std::str::from_utf8(hex).unwrap(), 16).unwrap())
            .collect();
        let expected: f64 = fields[2].parse().unwrap();
        let actual = shannon(&input);
        assert!(
            (actual - expected).abs() <= TOLERANCE,
            "{}: {actual} != {expected}",
            fields[0]
        );
        cases += 1;
    }
    assert!(cases >= 20);
}
