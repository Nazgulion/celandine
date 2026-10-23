//! Contracts independent of the optional backend.
use celandine::compression::{
    CompressionRatioError, Compressor, compressed_length, compression_ratio,
};
use std::{cell::Cell, error::Error, io};

struct Literal;
impl Compressor for Literal {
    type Error = std::num::TryFromIntError;
    fn compressed_len(&self, input: &[u8]) -> Result<usize, Self::Error> {
        usize::try_from(input.len() as u128 + 1)
    }
}

#[test]
fn lossless_literal_format_and_trait_object() {
    let codec: &dyn Compressor<Error = std::num::TryFromIntError> = &Literal;
    for input in [b"".as_slice(), b"A", b"AB", b"ABAB"] {
        // A leading literal-format byte, then all input bytes.
        let stream: Vec<_> = std::iter::once(0).chain(input.iter().copied()).collect();
        assert_eq!(compressed_length(input, codec).unwrap(), stream.len());
        if !input.is_empty() {
            assert_eq!(
                compression_ratio(input, codec).unwrap(),
                stream.len() as f64 / input.len() as f64
            );
        }
    }
    assert_eq!(compression_ratio(b"AB", codec).unwrap(), 1.5);
    assert_eq!(
        compression_ratio(b"", codec),
        Err(CompressionRatioError::EmptyInput)
    );
}

#[test]
fn errors_preserve_source_and_empty_ratio_skips_backend() {
    struct Failing(Cell<usize>);
    impl Compressor for Failing {
        type Error = io::Error;
        fn compressed_len(&self, _: &[u8]) -> Result<usize, Self::Error> {
            self.0.set(self.0.get() + 1);
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "test failure",
            ))
        }
    }
    let codec = Failing(Cell::new(0));
    let error = compression_ratio(b"", &codec).unwrap_err();
    assert!(matches!(error, CompressionRatioError::EmptyInput));
    assert!(error.source().is_none());
    assert_eq!(codec.0.get(), 0);
    assert_eq!(
        compressed_length(b"", &codec).unwrap_err().kind(),
        io::ErrorKind::PermissionDenied
    );
    let error = compression_ratio(b"A", &codec).unwrap_err();
    assert_eq!(error.source().unwrap().to_string(), "test failure");
    assert!(error.to_string().contains("test failure"));
    assert_eq!(codec.0.get(), 2);
}

#[test]
fn large_byte_counts_round_without_integer_division() {
    struct Fixed;
    impl Compressor for Fixed {
        type Error = std::convert::Infallible;
        fn compressed_len(&self, _: &[u8]) -> Result<usize, Self::Error> {
            Ok(usize::MAX)
        }
    }
    let ratio = compression_ratio(b"ABC", &Fixed).unwrap();
    assert!(ratio.is_finite());
    assert_eq!(ratio, usize::MAX as f64 / 3.0);
}

#[cfg(feature = "compression-deflate")]
mod zlib {
    use super::*;
    use celandine::compression::Zlib;
    use proptest::prelude::*;
    use proptest::test_runner::RngSeed;

    #[test]
    fn hand_parsed_streams_and_short_input_overhead() {
        // RFC 1950: 2-byte header + fixed-Huffman DEFLATE + 4-byte Adler-32.
        // Empty DEFLATE: 03 00. Single A: 73 04 00.
        assert_eq!(compressed_length(b"", &Zlib).unwrap(), 2 + 2 + 4);
        assert_eq!(compressed_length(b"A", &Zlib).unwrap(), 2 + 3 + 4);
        assert_eq!(compression_ratio(b"A", &Zlib).unwrap(), 9.0);
        assert!(matches!(
            compression_ratio(b"", &Zlib),
            Err(CompressionRatioError::EmptyInput)
        ));
    }

    #[test]
    fn repetitive_input_compresses_and_calls_are_independent() {
        let repeated = vec![b'A'; 100_000];
        let before = compressed_length(&repeated, &Zlib).unwrap();
        assert!(before < repeated.len() / 100);
        compressed_length(b"unrelated prior input", &Zlib).unwrap();
        assert_eq!(before, compressed_length(&repeated, &Zlib).unwrap());
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 128, rng_seed: RngSeed::Fixed(20261023), ..ProptestConfig::default() })]
        #[test]
        fn lengths_match_materialized_streams(data in prop::collection::vec(any::<u8>(), 0..100_000)) {
            let stream = miniz_oxide::deflate::compress_to_vec_zlib(&data, 6);
            prop_assert_eq!(compressed_length(&data, &Zlib).unwrap(), stream.len());
            prop_assert_eq!(miniz_oxide::inflate::decompress_to_vec_zlib(&stream).unwrap(), data.clone());
            if !data.is_empty() {
                prop_assert_eq!(compression_ratio(&data, &Zlib).unwrap(), stream.len() as f64 / data.len() as f64);
            }
        }
    }
}
