//! Explicit conventions checked against independent high-precision fixtures.
use celandine::complexity::{
    Lz76NormalizationError as Error, Lz76Summary, lz76_complexity_asymptotic as asymptotic,
    lz76_phrase_density as density,
};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;

#[test]
fn exact_examples_and_domain() {
    for (data, d, a) in [
        (b"".as_slice(), 0.0_f64, 0.0_f64),
        (b"A", 1.0, 0.0),
        (b"AA", 1.0, 1.0),
        (b"AAAA", 0.5, 1.0),
        (b"ABAB", 0.75, 1.5),
    ] {
        assert_eq!(density(data).to_bits(), d.to_bits());
        assert_eq!(asymptotic(data, 2).unwrap().to_bits(), a.to_bits());
    }
    assert_eq!(asymptotic(b"ABAB", 4), Ok(0.75));
    assert_eq!(asymptotic(&[255, 128, 255, 128], 2), Ok(1.5));
    assert_eq!(asymptotic(b"ABC", 2), Err(Error::AlphabetTooSmall));
    for k in [0, 1, 257, usize::MAX] {
        for data in [b"".as_slice(), b"A", b"ABC"] {
            assert_eq!(asymptotic(data, k), Err(Error::InvalidAlphabetSize));
            assert_eq!(
                Lz76Summary::from_bytes(data).asymptotic(k),
                Err(Error::InvalidAlphabetSize)
            );
        }
    }
    let all: Vec<u8> = (0..=255).collect();
    assert_eq!(density(&all), 1.0);
    assert_eq!(asymptotic(&all, 256), Ok(1.0));
    assert_eq!(asymptotic(&all, 255), Err(Error::AlphabetTooSmall));
    let saved = {
        let temporary = b"ABAB".to_vec();
        Lz76Summary::from_bytes(&temporary)
    };
    assert_eq!(saved.asymptotic(2), Ok(1.5));
    assert_ne!(density(b"AAB"), density(b"BAA"));
    fn is_error(_: &dyn std::error::Error) {}
    for error in [Error::InvalidAlphabetSize, Error::AlphabetTooSmall] {
        is_error(&error);
        assert!(!error.to_string().is_empty());
    }
}

#[test]
fn independent_decimal_fixtures() {
    let mut rows = 0;
    for line in include_str!("data/lz76_normalized.tsv")
        .lines()
        .filter(|s| !s.starts_with('#'))
    {
        let f: Vec<_> = line.split('\t').collect();
        let data: Vec<_> = if f[0] == "-" {
            vec![]
        } else {
            (0..f[0].len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&f[0][i..i + 2], 16).unwrap())
                .collect()
        };
        let k = f[1].parse().unwrap();
        let s = Lz76Summary::from_bytes(&data);
        assert_eq!(s.len(), data.len());
        assert_eq!(s.is_empty(), data.is_empty());
        assert_eq!(s.phrase_count(), f[2].parse::<usize>().unwrap());
        assert_eq!(s.support_size(), f[3].parse::<usize>().unwrap());
        assert!((density(&data) - f[4].parse::<f64>().unwrap()).abs() <= 1e-12);
        assert_eq!(density(&data).to_bits(), s.phrase_density().to_bits());
        let actual = asymptotic(&data, k);
        match f[5] {
            "InvalidAlphabetSize" => assert_eq!(actual, Err(Error::InvalidAlphabetSize)),
            "AlphabetTooSmall" => assert_eq!(actual, Err(Error::AlphabetTooSmall)),
            value => assert!(
                (actual.unwrap() - value.parse::<f64>().unwrap()).abs() <= 1e-12,
                "{line}"
            ),
        }
        assert_eq!(actual.map(f64::to_bits), s.asymptotic(k).map(f64::to_bits));
        rows += 1;
    }
    assert_eq!(rows, 7224);
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, rng_seed: RngSeed::Fixed(0x4c5a_4e4f), ..ProptestConfig::default() })]
    #[test]
    fn bounds_alphabet_monotonicity_and_label_invariance(data in prop::collection::vec(any::<u8>(), 0..129), mask in any::<u8>()) {
        let s = Lz76Summary::from_bytes(&data);
        let k = s.support_size().max(2);
        let a = s.asymptotic(k).unwrap();
        prop_assert!((0.0..=1.0).contains(&s.phrase_density()));
        prop_assert!(a.is_finite() && a >= 0.0);
        prop_assert!(a <= (data.len().max(1) as f64).log2() / (k as f64).log2() + 1e-12);
        prop_assert!(s.asymptotic(256).unwrap() <= a);
        let renamed: Vec<_> = data.iter().map(|b| b ^ mask).collect();
        prop_assert_eq!(s, Lz76Summary::from_bytes(&renamed));
        prop_assert_eq!(asymptotic(&data, k).unwrap().to_bits(), a.to_bits());
    }
    #[test]
    fn constant_string_formula(n in 2usize..16385, byte in any::<u8>()) {
        let data = vec![byte; n];
        prop_assert_eq!(density(&data), 2.0/n as f64);
        let expected = 2.0*(n as f64).ln()/(n as f64*2.0_f64.ln());
        prop_assert!((asymptotic(&data, 2).unwrap()-expected).abs() <= 1e-12);
    }
}
