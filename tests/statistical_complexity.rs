//! Independent normalization, alphabet and numerical checks.
use celandine::complexity::{
    StatisticalComplexityError as Error, normalized_jensen_shannon_complexity as complexity,
    normalized_jensen_shannon_complexity_distribution as reuse,
};
use celandine::distribution::{ByteHistogram, Distribution};
use proptest::{prelude::*, test_runner::RngSeed};

fn distribution(counts: &[usize]) -> Distribution {
    let mut table = [0; 256];
    table[..counts.len()].copy_from_slice(counts);
    Distribution::from_counts(ByteHistogram::try_from_counts(table).unwrap())
}

#[test]
fn endpoints_alphabet_semantics_and_errors() {
    for k in 2..=256 {
        assert_eq!(complexity(b"AAAA", k).unwrap().to_bits(), 0.0_f64.to_bits());
        let uniform: Vec<_> = (0..k).map(|i| i as u8).collect();
        assert_eq!(
            complexity(&uniform, k).unwrap().to_bits(),
            0.0_f64.to_bits()
        );
    }
    assert_eq!(complexity(b"AABB", 2), Ok(0.0));
    assert!(complexity(b"AABB", 4).unwrap() > 0.0);
    assert_eq!(complexity(b"AAAB", 2), complexity(b"BAAA", 2));
    assert_eq!(complexity(b"AAAB", 2), complexity(&[255, 255, 255, 0], 2));
    assert_eq!(complexity(b"ABC", 2), Err(Error::AlphabetTooSmall));
    for k in [0, 1, 257, usize::MAX] {
        for input in [b"".as_slice(), b"A", b"ABC"] {
            assert_eq!(complexity(input, k), Err(Error::InvalidAlphabetSize));
            assert_eq!(
                reuse(&Distribution::from_bytes(input), k),
                Err(Error::InvalidAlphabetSize)
            );
        }
    }
    assert_eq!(complexity(b"", 2), Err(Error::EmptyDistribution));
    for error in [
        Error::InvalidAlphabetSize,
        Error::EmptyDistribution,
        Error::AlphabetTooSmall,
    ] {
        assert!(!error.to_string().is_empty());
    }
}

#[test]
fn independent_160_digit_references() {
    let mut checked = 0;
    let mut skipped = 0;
    for line in include_str!("data/statistical_complexity.tsv")
        .lines()
        .filter(|s| !s.starts_with('#'))
    {
        let f: Vec<_> = line.split('\t').collect();
        let counts: Vec<u128> = if f[1] == "-" {
            vec![]
        } else {
            f[1].split(',').map(|s| s.parse().unwrap()).collect()
        };
        if counts.iter().sum::<u128>() > usize::MAX as u128 {
            skipped += 1;
            continue;
        }
        let counts: Vec<_> = counts.into_iter().map(|c| c as usize).collect();
        let actual = reuse(&distribution(&counts), f[2].parse().unwrap());
        match f[3] {
            "alphabet_error" => assert_eq!(actual, Err(Error::InvalidAlphabetSize)),
            "empty_error" => assert_eq!(actual, Err(Error::EmptyDistribution)),
            "support_error" => assert_eq!(actual, Err(Error::AlphabetTooSmall)),
            _ => {
                let expected: f64 = f[3].parse().unwrap();
                let actual = actual.unwrap();
                if expected == 0.0 {
                    assert_eq!(actual.to_bits(), 0.0_f64.to_bits(), "{}", f[0]);
                } else {
                    assert!(
                        actual > 0.0 && (actual - expected).abs() <= 5e-13 * expected,
                        "{}: {actual:e} vs {expected:e}",
                        f[0]
                    );
                }
            }
        }
        checked += 1;
    }
    assert_eq!(checked + skipped, 853);
    if usize::BITS == 64 {
        assert_eq!(skipped, 0);
    }
}

// This checks a cancellation-sensitive limit, not an implementation detail.
#[test]
fn regression_nearly_certain_law_retains_majority_entropy() {
    let n = usize::MAX;
    let actual = reuse(&distribution(&[n - 1, 1]), 2).unwrap();
    let epsilon = 1.0 / n as f64;
    // As epsilon -> 0, J/Jmax -> 1 and H ~ epsilon*(log2(1/epsilon)+1/ln2).
    let limit = epsilon * ((n as f64).log2() + 1.0 / std::f64::consts::LN_2);
    assert!(
        (actual / limit - 1.0).abs() < 1e-6,
        "{actual:e} vs {limit:e}"
    );
}

proptest! {
    #![proptest_config(ProptestConfig {cases:256,rng_seed:RngSeed::Fixed(20261101),..ProptestConfig::default()})]
    #[test]
    fn byte_reuse_permutation_and_repetition(
        raw in prop::collection::vec(any::<u8>(),1..1024), k in 2usize..257,
    ) {
        let data:Vec<_>=raw.iter().map(|&b|(usize::from(b)%k) as u8).collect();
        let c=complexity(&data,k).unwrap();
        prop_assert_eq!(reuse(&Distribution::from_bytes(&data),k).unwrap(),c);
        prop_assert!(c.is_finite() && (0.0..=1.0+1e-14).contains(&c));
        let mut reverse=data.clone();reverse.reverse();
        prop_assert_eq!(complexity(&reverse,k).unwrap(),c);
        let renamed:Vec<_>=data.iter().map(|b|b.wrapping_add(173)).collect();
        prop_assert!((complexity(&renamed,k).unwrap()-c).abs()<=1e-13*c.max(1e-100));
        let repeated=data.repeat(3);
        prop_assert!((complexity(&repeated,k).unwrap()-c).abs()<=1e-13*c.max(1e-100));
    }
}
