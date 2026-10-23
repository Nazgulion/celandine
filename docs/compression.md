# Compression measurements

A compressed length measures how many bytes a particular lossless encoder uses
for a complete input. It reveals repetition that a byte histogram cannot see.
A compression ratio compares that cost with the original byte length. These are
exact encoder measurements (apart from floating-point ratio rounding), and only
compressor-dependent proxies for descriptive complexity, not Kolmogorov complexity
or estimates of an unknown source's entropy.

## Definition and operation

`Compressor` supplies `compressed_len(input)` and its own error type. Each call
must measure a fresh, complete lossless stream, including framing and finalization.
Custom implementations document their format, configuration, dictionary policy,
and failures. The interface and generic measurements have no runtime dependencies.

`compressed_length(x, &codec)` returns C(x), the stream's size in bytes as `usize`.
`compression_ratio(x, &codec)` returns R(x) = C(x) / |x| as `f64`: divide compressed
bytes by the number of original bytes. R is dimensionless and can exceed one;
it is not clamped to [0, 1]. Successful lengths are nonnegative integers fitting
`usize`; successful ratios are finite and nonnegative on supported platforms.
The chosen lossless format determines tighter bounds. No estimated overhead is subtracted. A ratio call
compresses once; calling length and ratio separately compresses twice.

Empty input still has a complete encoded stream. Its compressed length is valid,
but its ratio returns `CompressionRatioError::EmptyInput` before invoking the
compressor. Backend failures propagate unchanged from length and appear in
`CompressionRatioError::Compressor` from ratio. Integer conversion can round
lengths above 2^53. Generic time/space costs depend on the supplied backend.

## Worked example: an inspectable custom format

Suppose a lossless format prefixes the unchanged bytes with `0x00` to identify
its literal encoding. For `AB`, its stream is `[0x00, 0x41, 0x42]`, so C = 3 bytes
and R = 3 / 2 = 1.5. It expands the input, illustrating why a ratio can exceed one.

```
use celandine::compression::{Compressor, compressed_length, compression_ratio,
    CompressionRatioError};
struct Literal;
impl Compressor for Literal {
    type Error = std::num::TryFromIntError;
    fn compressed_len(&self, input: &[u8]) -> Result<usize, Self::Error> {
        usize::try_from(input.len() as u128 + 1)
    }
}
assert_eq!(compressed_length(b"AB", &Literal).unwrap(), 3);
assert_eq!(compression_ratio(b"AB", &Literal).unwrap(), 1.5);
assert_eq!(compressed_length(b"", &Literal).unwrap(), 1);
assert_eq!(compression_ratio(b"", &Literal), Err(CompressionRatioError::EmptyInput));
```

## Optional zlib backend

Enable Cargo feature `compression-deflate` to use `Zlib`. It pins miniz_oxide
0.9.1 and fixes level 6, default strategy, a 32 KiB window, no preset dictionary,
and RFC 1950 framing around RFC 1951 DEFLATE. Each call initializes fresh state;
there is no shared training or prior-input context. The encoder counts and
discards output through a 32 KiB scratch buffer. At fixed window/search limits,
time is O(n), auxiliary memory O(1) in input length n, including initialization.
Backend failure or size overflow returns `std::io::Error`.

For `A`, the complete stream is `78 9c 73 04 00 00 42 00 42`: a two-byte zlib
header, three-byte DEFLATE payload, and four-byte Adler-32 checksum. C = 9 bytes
and R = 9 / 1 = 9. Empty input occupies eight bytes. These examples are checked
against an independent decoder and hand-parsed format fields. The six bytes of
zlib framing are retained; DEFLATE also has block and end-marker overhead.

DEFLATE combines LZ77 matches with Huffman coding. L. Peter Deutsch documented
its interoperable format in May 1996 in RFC 1951; Deutsch and Jean-loup Gailly
specified zlib's wrapper and checksum in RFC 1950 that month. These formats
standardize decoding, not an encoder's choice of matches or output size.

Small inputs can be dominated by framing and initialization. Finite windows
miss distant repetitions. Different codecs, levels, versions, input boundaries,
and dictionaries can give different results. Retain Cargo.lock and configuration
with measurements; a future backend upgrade may change byte counts. These
measurements are distinct from exhaustive LZ76 phrase counts and byte entropy.

## References and verification

- [Deutsch and Gailly, RFC 1950 (1996)](https://www.rfc-editor.org/rfc/rfc1950), §§2.2–2.3.
- [Deutsch, RFC 1951 (1996)](https://www.rfc-editor.org/rfc/rfc1951), §§3.2–3.3.
- [miniz_oxide compression API](https://docs.rs/miniz_oxide/0.9.1/miniz_oxide/deflate/core/index.html).
- [Python zlib decoder](https://docs.python.org/3/library/zlib.html).

Run `python3 scripts/reference_compression.py --check` to decode actual counted
streams with Python's independently implemented C-zlib, verify complete stream
termination, payloads, checksums, lengths and ratios. Cross-encoder sizes are not
expected to agree. See `docs/benchmarks/compression.md` for measured performance.
