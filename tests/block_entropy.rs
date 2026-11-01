//! Independent block-entropy values and finite-sample invariants.
use celandine::distribution::ngram_probabilities;
use celandine::entropy::{block_entropy, block_entropy_distribution, shannon};
use celandine::transforms::InvalidNgramLength;
use proptest::{prelude::*, test_runner::RngSeed};

fn close(actual: f64, expected: f64) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= 1e-12,
        "{actual} vs {expected}"
    );
}

#[test]
fn canonical_units_overlap_and_empty_cases() {
    assert_eq!(block_entropy(b"ABABA", 2), Ok(1.0));
    assert_eq!(block_entropy(b"AABBA", 2), Ok(2.0));
    // Non-overlapping AA/BB would give 1, while AA/AB/BB gives log2(3).
    close(block_entropy(b"AABB", 2).unwrap(), 3.0_f64.log2());
    close(
        block_entropy(b"ABAB", 2).unwrap(),
        3.0_f64.log2() - 2.0 / 3.0,
    );
    assert_eq!(block_entropy(b"ABABA", 5), Ok(0.0));
    for data in [b"".as_slice(), b"A", b"AAAA", b"ABABA"] {
        assert_eq!(block_entropy(data, 0), Err(InvalidNgramLength));
        for n in [data.len().max(1), data.len() + 1, usize::MAX] {
            assert_eq!(block_entropy(data, n).unwrap().to_bits(), 0.0_f64.to_bits());
        }
        for n in 1..8 {
            let d = ngram_probabilities(data, n).unwrap();
            close(
                block_entropy(data, n).unwrap(),
                block_entropy_distribution(&d),
            );
            if d.support_size() <= 1 {
                assert_eq!(block_entropy_distribution(&d).to_bits(), 0.0_f64.to_bits());
            }
        }
    }
    assert!(block_entropy(b"ABABA", 2).unwrap() > block_entropy(b"ABABA", 5).unwrap());
}

#[test]
fn independent_high_precision_fixtures() {
    let mut checked = 0;
    for row in include_str!("data/block_entropy.tsv")
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
        let n = if f[1] == "max" {
            usize::MAX
        } else {
            f[1].parse().unwrap()
        };
        if f[2] == "error" {
            assert_eq!(block_entropy(&data, n), Err(InvalidNgramLength));
        } else {
            let expected = f[2].parse().unwrap();
            close(block_entropy(&data, n).unwrap(), expected);
            close(
                block_entropy_distribution(&ngram_probabilities(&data, n).unwrap()),
                expected,
            );
        }
        checked += 1;
    }
    assert!(checked > 1000);
}

#[test]
fn rare_transition_and_support_beyond_byte_alphabet() {
    let mut data = vec![b'A'; 65536];
    data.push(b'B');
    let epsilon = 1.0 / 65536.0_f64;
    let expected = -(1.0 - epsilon) * (1.0 - epsilon).log2() - epsilon * epsilon.log2();
    close(block_entropy(&data, 2).unwrap(), expected);
    // Four-byte support can exceed the byte alphabet.
    let data: Vec<_> = (0..512)
        .flat_map(|i| [(i % 256) as u8, (i / 256) as u8])
        .collect();
    let d = ngram_probabilities(&data, 4).unwrap();
    assert!(d.support_size() > 256);
    assert!(block_entropy_distribution(&d) > 8.0);
}

proptest! {
    #![proptest_config(ProptestConfig { cases:256, rng_seed:RngSeed::Fixed(20261101), ..ProptestConfig::default() })]
    #[test]
    fn bounds_reuse_renaming_and_reversal(data in prop::collection::vec(any::<u8>(),0..1024), n in 1usize..1100) {
        let h = block_entropy(&data,n).unwrap();
        let d = ngram_probabilities(&data,n).unwrap();
        prop_assert!(h.is_finite() && h >= 0.0);
        prop_assert!(h <= (d.support_size().max(1) as f64).log2()+1e-12);
        prop_assert!((h-block_entropy_distribution(&d)).abs() <= 1e-12);
        let renamed:Vec<_> = data.iter().map(|b| b.wrapping_add(137)).collect();
        let reversed:Vec<_> = data.iter().rev().copied().collect();
        prop_assert!((h-block_entropy(&renamed,n).unwrap()).abs() <= 1e-12);
        prop_assert!((h-block_entropy(&reversed,n).unwrap()).abs() <= 1e-12);
        prop_assert_eq!(block_entropy(&data,1).unwrap(), shannon(&data));
    }
}
