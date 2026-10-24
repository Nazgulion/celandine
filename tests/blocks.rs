//! Partition coverage, remainder stability, and mixed iterator operations.
use celandine::transforms::{InvalidBlockSize, blocks, windows};
use proptest::{prelude::*, test_runner::RngSeed};
use std::collections::VecDeque;

fn decode(s: &str) -> Vec<u8> {
    if s == "-" {
        return vec![];
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn independent_python_partitions() {
    let mut cases = 0;
    for line in include_str!("data/blocks.tsv")
        .lines()
        .filter(|s| !s.starts_with('#'))
    {
        let fields: Vec<_> = line.split('\t').collect();
        let data = decode(fields[0]);
        let size = if fields[1] == "max" {
            usize::MAX
        } else {
            fields[1].parse().unwrap()
        };
        let result = blocks(&data, size);
        if fields[2] == "error" {
            assert_eq!(result.unwrap_err(), InvalidBlockSize);
        } else {
            let expected: Vec<_> = if fields[2] == "-" {
                vec![]
            } else {
                fields[2].split(',').map(decode).collect()
            };
            let iter = result.unwrap();
            assert_eq!(iter.len(), expected.len());
            assert_eq!(iter.remainder(), decode(fields[3]));
            let mut offset = 0;
            for (actual, wanted) in iter.clone().zip(&expected) {
                assert_eq!(actual, wanted);
                assert_eq!(actual.as_ptr(), data[offset..].as_ptr());
                offset += actual.len();
            }
            assert_eq!(iter.remainder().as_ptr(), data[offset..].as_ptr());
            assert!(
                iter.clone()
                    .rev()
                    .eq(expected.iter().rev().map(Vec::as_slice))
            );
            let mut rebuilt: Vec<_> = iter.clone().flatten().copied().collect();
            rebuilt.extend_from_slice(iter.remainder());
            assert_eq!(rebuilt, data);
        }
        cases += 1;
    }
    assert_eq!(cases, 325);
}

#[test]
fn tail_is_original_suffix_not_unconsumed_blocks() {
    let mut iter = blocks(b"ABCDEFGH", 3).unwrap();
    assert_eq!(iter.remainder(), b"GH");
    assert_eq!(iter.next_back(), Some(b"DEF".as_slice()));
    assert_eq!(iter.remainder(), b"GH");
    assert_eq!(iter.next(), Some(b"ABC".as_slice()));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
    assert_eq!(iter.remainder(), b"GH");
    assert_eq!(
        blocks(b"ABABA", 2).unwrap().collect::<Vec<_>>(),
        [b"AB", b"AB"]
    );
}

#[test]
fn errors_empty_extreme_sizes_skips_and_traits() {
    fn traits<
        T: ExactSizeIterator
            + DoubleEndedIterator
            + std::iter::FusedIterator
            + Clone
            + std::fmt::Debug,
    >(
        _: T,
    ) {
    }
    traits(blocks(b"ABC", 2).unwrap());
    assert!(InvalidBlockSize.to_string().contains("size"));
    for data in [b"".as_slice(), b"A", b"ABCDEFGH"] {
        assert_eq!(blocks(data, 0).unwrap_err(), InvalidBlockSize);
        for size in [1, 2, 3, data.len().saturating_add(1), usize::MAX] {
            let iter = blocks(data, size).unwrap();
            assert_eq!(iter.len(), data.len() / size);
            assert_eq!(iter.clone().count(), iter.len());
            assert_eq!(iter.clone().last(), iter.clone().next_back());
            for reverse in [false, true] {
                let mut skipped = iter.clone();
                assert_eq!(
                    if reverse {
                        skipped.nth_back(usize::MAX)
                    } else {
                        skipped.nth(usize::MAX)
                    },
                    None
                );
                assert_eq!(skipped.len(), 0);
                assert_eq!(skipped.next(), None);
                assert_eq!(skipped.next_back(), None);
                assert_eq!(skipped.remainder(), iter.remainder());
            }
        }
        assert_eq!(blocks(data, usize::MAX).unwrap().remainder(), data);
    }
}

proptest! {
    #![proptest_config(ProptestConfig {cases:256, rng_seed:RngSeed::Fixed(20261025), ..ProptestConfig::default()})]
    #[test]
    fn partition_and_state_match_independent_positions(
        data in prop::collection::vec(any::<u8>(),0..256),
        width in prop_oneof![1usize..300,Just(usize::MAX)],
        actions in prop::collection::vec((0u8..4,prop_oneof![0usize..300,Just(usize::MAX)]),0..64)
    ) {
        let selected:Vec<_>=(0..data.len()).filter(|&i| i%width==0 && width<=data.len()-i).map(|i| &data[i..i+width]).collect();
        let covered=selected.iter().map(|s| s.len()).sum::<usize>();
        let tail=&data[covered..];
        let mut expected:VecDeque<_>=selected.into();
        let mut actual=blocks(&data,width).unwrap();
        prop_assert!(actual.clone().eq(windows(&data,width,width).unwrap()));
        prop_assert_eq!(actual.remainder(),tail);
        for (op,n) in actions {
            let wanted=match op {
                0=>expected.pop_front(),1=>expected.pop_back(),
                2=>{ if n>=expected.len(){expected.clear();None} else {expected.drain(..n);expected.pop_front()} },
                _=>{ if n>=expected.len(){expected.clear();None} else {expected.truncate(expected.len()-n);expected.pop_back()} },
            };
            let got=match op {0=>actual.next(),1=>actual.next_back(),2=>actual.nth(n),_=>actual.nth_back(n)};
            prop_assert_eq!(got,wanted);
            if let (Some(got),Some(wanted))=(got,wanted){prop_assert_eq!(got.as_ptr(),wanted.as_ptr());}
            prop_assert_eq!(actual.len(),expected.len());
            prop_assert_eq!(actual.size_hint(),(expected.len(),Some(expected.len())));
            prop_assert_eq!(actual.remainder(),tail);
            prop_assert_eq!(actual.remainder().as_ptr(),tail.as_ptr());
            prop_assert!(actual.clone().eq(expected.iter().copied()));
        }
        prop_assert!(actual.eq(expected));
    }
}
