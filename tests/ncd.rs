//! Ordered NCD contracts and independent rational fixtures.
use celandine::compression::{
    CompressionDistanceComponents as Parts, CompressionDistanceError as Failure, Compressor,
    ZeroCompressedLength, compression_distance_components as components,
    normalized_compression_distance as ncd,
};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use std::{
    cell::{Cell, RefCell},
    error::Error,
    io,
};

struct Literal;
impl Compressor for Literal {
    type Error = std::num::TryFromIntError;
    fn compressed_len(&self, data: &[u8]) -> Result<usize, Self::Error> {
        usize::try_from(data.len() as u128 + 1)
    }
}

#[test]
fn literal_format_empty_inputs_and_no_identity_shortcut() {
    let codec: &dyn Compressor<Error = std::num::TryFromIntError> = &Literal;
    assert_eq!(
        components(b"AB", b"CD", codec).unwrap(),
        Parts { x: 3, y: 3, xy: 5 }
    );
    assert_eq!(ncd(b"AB", b"AB", codec).unwrap(), 2.0 / 3.0);
    assert_eq!(ncd(b"", b"", codec).unwrap(), 0.0);
    assert_eq!(ncd(b"", b"A", codec).unwrap(), 0.5);
    assert_eq!(ncd(b"A", b"", codec).unwrap(), 0.5);
}

#[test]
fn exact_order_raw_bytes_and_three_fresh_calls() {
    struct Recorder(RefCell<Vec<Vec<u8>>>);
    impl Compressor for Recorder {
        type Error = std::convert::Infallible;
        fn compressed_len(&self, data: &[u8]) -> Result<usize, Self::Error> {
            self.0.borrow_mut().push(data.to_vec());
            // A lossless format can choose a nine-byte header for A-prefixed
            // payloads and a one-byte header otherwise, followed by literal bytes.
            Ok(data.len() + if data.first() == Some(&b'A') { 9 } else { 1 })
        }
    }
    let codec = Recorder(RefCell::new(vec![]));
    assert_eq!(ncd(b"A", b"B", &codec).unwrap(), 0.9);
    assert_eq!(
        *codec.0.borrow(),
        vec![b"A".to_vec(), b"B".to_vec(), b"AB".to_vec()]
    );
    assert_eq!(ncd(b"B", b"A", &codec).unwrap(), 0.1);
    codec.0.borrow_mut().clear();
    components(b"\0\xff", b"", &codec).unwrap();
    assert_eq!(*codec.0.borrow(), vec![vec![0, 255], vec![], vec![0, 255]]);
}

#[test]
fn failures_at_each_compression_preserve_source_and_stop() {
    struct Failing {
        calls: Cell<usize>,
        fail: usize,
    }
    impl Compressor for Failing {
        type Error = io::Error;
        fn compressed_len(&self, data: &[u8]) -> Result<usize, Self::Error> {
            let call = self.calls.get();
            self.calls.set(call + 1);
            if call == self.fail {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "backend sentinel",
                ))
            } else {
                Ok(data.len() + 1)
            }
        }
    }
    for fail in 0..3 {
        let codec = Failing {
            calls: Cell::new(0),
            fail,
        };
        let error = ncd(b"A", b"B", &codec).unwrap_err();
        assert_eq!(error.source().unwrap().to_string(), "backend sentinel");
        assert!(error.to_string().contains("backend sentinel"));
        assert_eq!(codec.calls.get(), fail + 1);
        assert!(matches!(error, Failure::Compressor(_)));
    }
}

#[test]
fn zero_denominator_components_remain_inspectable() {
    struct EmptyCode;
    impl Compressor for EmptyCode {
        type Error = std::convert::Infallible;
        fn compressed_len(&self, _: &[u8]) -> Result<usize, Self::Error> {
            Ok(0)
        }
    }
    let parts = components(b"", b"", &EmptyCode).unwrap();
    assert_eq!(parts.normalized_distance(), Err(ZeroCompressedLength));
    let error = ncd(b"", b"", &EmptyCode).unwrap_err();
    assert!(matches!(error, Failure::ZeroDenominator));
    assert!(error.source().is_none());
    assert!(error.to_string().contains("nonzero"));
}

#[test]
fn rational_fixtures_include_negative_above_one_and_unit_gaps() {
    let mut checked = 0;
    for line in include_str!("data/ncd.tsv")
        .lines()
        .filter(|s| !s.starts_with('#'))
    {
        let fields: Vec<_> = line.split('\t').collect();
        let sizes: Vec<_> = fields[..3]
            .iter()
            .map(|s| s.parse::<u64>().unwrap())
            .collect();
        let Ok(x) = usize::try_from(sizes[0]) else {
            continue;
        };
        let Ok(y) = usize::try_from(sizes[1]) else {
            continue;
        };
        let Ok(xy) = usize::try_from(sizes[2]) else {
            continue;
        };
        let actual = Parts { x, y, xy }.normalized_distance();
        if fields[3] == "undefined" {
            assert_eq!(actual, Err(ZeroCompressedLength));
        } else {
            let expected = fields[3].parse::<f64>().unwrap();
            let actual = actual.unwrap();
            assert!(
                (actual - expected).abs() <= expected.abs() * 4.0 * f64::EPSILON,
                "{line}: {actual}"
            );
            assert!(actual.is_finite());
        }
        checked += 1;
    }
    assert!(checked >= 512);
    assert_eq!(
        Parts {
            x: 10,
            y: 10,
            xy: 5
        }
        .normalized_distance()
        .unwrap(),
        -0.5
    );
    assert_eq!(
        Parts {
            x: 10,
            y: 10,
            xy: 30
        }
        .normalized_distance()
        .unwrap(),
        2.0
    );
}

proptest! {
    #![proptest_config(ProptestConfig{cases:256,rng_seed:RngSeed::Fixed(20261024),..ProptestConfig::default()})]
    #[test]
    fn literal_encoder_matches_independent_simplified_formula(x in prop::collection::vec(any::<u8>(),0..1024),y in prop::collection::vec(any::<u8>(),0..1024)) {
        // C(xy)-min(C(x),C(y)) = max(|x|,|y|) for this exact literal format.
        let maximum=x.len().max(y.len()) as f64;
        prop_assert_eq!(ncd(&x,&y,&Literal).unwrap(),maximum/(maximum+1.0));
    }
}

#[cfg(feature = "compression-deflate")]
#[test]
fn zlib_matches_materialized_streams_at_window_boundaries() {
    use celandine::compression::Zlib;
    for size in [0, 1, 16, 32767, 32768, 32769, 65536] {
        let x: Vec<_> = (0..size).map(|i| i as u8).collect();
        let y: Vec<_> = (0..size + 3).map(|i| (i * 7) as u8).collect();
        let xy = [x.as_slice(), y.as_slice()].concat();
        let enc = |input: &[u8]| miniz_oxide::deflate::compress_to_vec_zlib(input, 6).len();
        let expected = Parts {
            x: enc(&x),
            y: enc(&y),
            xy: enc(&xy),
        };
        assert_eq!(components(&x, &y, &Zlib).unwrap(), expected);
        assert_eq!(
            ncd(&x, &y, &Zlib).unwrap(),
            expected.normalized_distance().unwrap()
        );
    }
    assert_eq!(ncd(b"", b"", &Zlib).unwrap(), 0.0);
    assert_eq!(ncd(b"", b"A", &Zlib).unwrap(), 1.0 / 9.0);
}
