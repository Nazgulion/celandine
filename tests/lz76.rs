//! Verify exhaustive LZ76 counts independently of the greedy source scanner.
use celandine::complexity::lz76_complexity as lz;
use proptest::prelude::*;
use proptest::test_runner::RngSeed;

// Minimum over ALL production histories, not a greedy longest-match parse.
// A production copies its prefix and appends one byte. An earlier occurrence
// of that prefix must end before the candidate's penultimate byte.
fn minimum_history(data: &[u8]) -> Vec<usize> {
    let mut minimum = vec![0; data.len() + 1];
    for end in 1..=data.len() {
        minimum[end] = end;
        for start in 0..end {
            let copied = &data[start..end - 1];
            let possible = copied.is_empty()
                || data[..end - 2]
                    .windows(copied.len())
                    .any(|word| word == copied);
            if possible {
                minimum[end] = minimum[end].min(minimum[start] + 1);
            }
        }
    }
    minimum
}

#[test]
fn canonical_overlapping_and_final_phrase_conventions() {
    for (data, expected) in [
        (&b""[..], 0),
        (b"A", 1),
        (b"AA", 2),
        (b"AAAA", 2),
        (b"AB", 2),
        (b"ABAB", 3),
        (b"ABABAB", 3),
        (b"ABCD", 4),
        (b"ABCABCABC", 4),
        (b"AAB", 2),
        (b"BAA", 3),
        (b"101101011", 5),
        (b"010011101101100", 6),
        (&[255, 0, 255, 0], 3),
        ("éé".as_bytes(), 3),
    ] {
        assert_eq!(lz(data), expected, "{data:?}");
    }
    assert_eq!(lz(&(0..=255).collect::<Vec<u8>>()), 256);
    assert_eq!(lz(&vec![0; 100_000]), 2);
    // Same byte counts, reversed order: LZ76 does not have reversal invariance.
    assert_ne!(lz(b"AAB"), lz(b"BAA"));
}

#[test]
fn exhaustive_binary_prefixes_match_minimum_over_all_histories() {
    for n in 0..=10 {
        for bits in 0..1usize << n {
            let data: Vec<_> = (0..n).map(|i| ((bits >> i) & 1) as u8).collect();
            let expected = minimum_history(&data);
            for end in 0..=n {
                assert_eq!(lz(&data[..end]), expected[end], "{data:?} prefix {end}");
            }
        }
    }
}

#[test]
fn independent_python_histories_and_phrase_boundaries() {
    let hex = |s: &str| {
        if s == "-" {
            vec![]
        } else {
            (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
                .collect::<Vec<_>>()
        }
    };
    let mut count = 0;
    for line in include_str!("data/lz76.tsv")
        .lines()
        .filter(|s| !s.starts_with('#'))
    {
        let parts: Vec<_> = line.split('\t').collect();
        let data = hex(parts[0]);
        let words: Vec<_> = if parts[1] == "-" {
            vec![]
        } else {
            parts[1].split(',').map(hex).collect()
        };
        assert_eq!(words.concat(), data);
        assert_eq!(words.len(), parts[2].parse::<usize>().unwrap());
        assert_eq!(lz(&data), words.len());
        let mut end = 0;
        for (i, word) in words.iter().enumerate() {
            end += word.len();
            assert_eq!(lz(&data[..end]), i + 1);
        }
        count += 1;
    }
    assert_eq!(count, 1035);
}

proptest! {
    #![proptest_config(ProptestConfig { cases:256, rng_seed:RngSeed::Fixed(0x4c5a_3736), ..ProptestConfig::default() })]
    #[test]
    fn exact_history_bounds_and_bijective_renaming(data in prop::collection::vec(any::<u8>(),0..65), mask in any::<u8>()) {
        let expected=minimum_history(&data);
        prop_assert_eq!(lz(&data),expected[data.len()]);
        prop_assert!(lz(&data)<=data.len());
        prop_assert_eq!(lz(&data),lz(&data.iter().map(|b| b^mask).collect::<Vec<_>>()));
        if data.len()>=2 { prop_assert!(lz(&data)>=2); }
    }
    #[test]
    fn appending_one_symbol_increases_count_by_at_most_one(data in prop::collection::vec(0u8..4,0..129), byte in 0u8..4) {
        let before=lz(&data);
        let mut after=data;after.push(byte);
        prop_assert!((before..=before+1).contains(&lz(&after)));
    }
    #[test]
    fn constant_strings_and_distinct_alphabets(n in 0usize..4097, byte in any::<u8>(), k in 0usize..257) {
        prop_assert_eq!(lz(&vec![byte;n]),n.min(2));
        prop_assert_eq!(lz(&(0..k).map(|b| b as u8).collect::<Vec<_>>()),k);
    }
}
