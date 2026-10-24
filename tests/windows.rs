//! Positional, borrowing, validation, and iterator-state contracts.
use celandine::transforms::{InvalidWindowParameters as Invalid, ngrams, windows};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use std::collections::VecDeque;

fn reference(data: &[u8], width: usize, stride: usize) -> Vec<&[u8]> {
    (0..data.len())
        .filter(|&i| i % stride == 0 && width <= data.len() - i)
        .map(|i| &data[i..i + width])
        .collect()
}

#[test]
fn canonical_overlap_gaps_tail_and_reverse_alignment() {
    assert_eq!(
        windows(b"ABCDEFGH", 3, 2).unwrap().collect::<Vec<_>>(),
        vec![b"ABC", b"CDE", b"EFG"]
    );
    assert_eq!(
        windows(b"ABCDEFGH", 3, 2)
            .unwrap()
            .rev()
            .collect::<Vec<_>>(),
        vec![b"EFG", b"CDE", b"ABC"]
    );
    assert_eq!(
        windows(b"ABCDEFGH", 2, 3).unwrap().collect::<Vec<_>>(),
        vec![b"AB", b"DE", b"GH"]
    );
    assert_eq!(
        windows(b"ABCDEFGH", 3, 3).unwrap().collect::<Vec<_>>(),
        vec![b"ABC", b"DEF"]
    );
    assert_eq!(windows(b"AAAA", 2, 1).unwrap().len(), 3);
    let text = "éé".as_bytes();
    assert_eq!(
        windows(text, 1, 2).unwrap().collect::<Vec<_>>(),
        vec![&text[0..1], &text[2..3]]
    );
}

#[test]
fn empty_invalid_and_extreme_parameters() {
    for data in [b"".as_slice(), b"A", b"ABCD"] {
        for step in [0, 1, usize::MAX] {
            assert_eq!(windows(data, 0, step).unwrap_err(), Invalid::ZeroSize);
        }
        for width in [1, usize::MAX] {
            assert_eq!(windows(data, width, 0).unwrap_err(), Invalid::ZeroStep);
        }
        assert_eq!(windows(data, usize::MAX, usize::MAX).unwrap().len(), 0);
        let mut iter = windows(data, 1, usize::MAX).unwrap();
        assert_eq!(iter.next(), data.get(..1));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }
    assert!(Invalid::ZeroSize.to_string().contains("size"));
    assert!(Invalid::ZeroStep.to_string().contains("step"));
}

#[test]
fn independent_python_positions_and_borrowed_pointers() {
    let mut count = 0;
    for line in include_str!("data/windows.tsv")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let f: Vec<_> = line.split('\t').collect();
        let data = if f[0] == "-" {
            vec![]
        } else {
            (0..f[0].len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&f[0][i..i + 2], 16).unwrap())
                .collect()
        };
        let parameter = |s: &str| {
            if s == "max" {
                usize::MAX
            } else {
                s.parse().unwrap()
            }
        };
        let (size, step) = (parameter(f[1]), parameter(f[2]));
        let result = windows(&data, size, step);
        match f[3] {
            "size_error" => assert_eq!(result.unwrap_err(), Invalid::ZeroSize),
            "step_error" => assert_eq!(result.unwrap_err(), Invalid::ZeroStep),
            positions => {
                let starts: Vec<usize> = if positions == "-" {
                    vec![]
                } else {
                    positions.split(',').map(|s| s.parse().unwrap()).collect()
                };
                let iter = result.unwrap();
                assert_eq!(iter.len(), starts.len());
                assert_eq!(iter.size_hint(), (starts.len(), Some(starts.len())));
                for (block, start) in iter.zip(starts.iter().copied()) {
                    assert_eq!(block, &data[start..start + size]);
                    assert_eq!(block.as_ptr(), data[start..].as_ptr());
                }
                assert!(
                    windows(&data, size, step)
                        .unwrap()
                        .rev()
                        .eq(starts.iter().rev().map(|&i| &data[i..i + size]))
                );
            }
        }
        count += 1;
    }
    assert_eq!(count, 1728);
}

#[test]
fn regression_iterator_traits_and_extreme_skips() {
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
    traits(windows(b"ABCDE", 2, 2).unwrap());
    for width in [1, 2, 4, usize::MAX] {
        for step in [1, 2, 3, usize::MAX] {
            let iter = windows(b"ABCDEFGH", width, step).unwrap();
            assert_eq!(iter.clone().nth(usize::MAX), None);
            assert_eq!(iter.clone().nth_back(usize::MAX), None);
            assert_eq!(iter.clone().last(), iter.clone().next_back());
            assert_eq!(iter.clone().count(), iter.len());
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig {cases:256,rng_seed:RngSeed::Fixed(20261024),..ProptestConfig::default()})]
    #[test]
    fn mixed_front_back_and_skip_operations_match_positions(
        data in prop::collection::vec(any::<u8>(),0..256),
        width in prop_oneof![1usize..300,Just(usize::MAX)],
        stride in prop_oneof![1usize..300,Just(usize::MAX)],
        actions in prop::collection::vec((0u8..4,prop_oneof![0usize..300,Just(usize::MAX)]),0..64)
    ) {
        let mut expected:VecDeque<_>=reference(&data,width,stride).into();
        let mut actual=windows(&data,width,stride).unwrap();
        prop_assert!(actual.clone().eq(expected.iter().copied()));
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
            prop_assert!(actual.clone().eq(expected.iter().copied()));
        }
        prop_assert!(actual.eq(expected));
    }
    #[test]
    fn step_one_equals_ngrams(data in prop::collection::vec(any::<u8>(),0..256),width in 1usize..300) {
        prop_assert!(windows(&data,width,1).unwrap().eq(ngrams(&data,width).unwrap()));
    }
}
