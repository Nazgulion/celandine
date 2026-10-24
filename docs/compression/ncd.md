# Normalized compression distance (NCD)

NCD compares two byte sequences by measuring their separate and combined encoded
sizes. It can reveal shared repetition beyond symbol frequencies. It is a
compressor-dependent similarity proxy, not exact normalized information distance
or a general-purpose metric guarantee.

## Definition and units

NCD(x, y) = [C(xy) - min(C(x), C(y))] / max(C(x), C(y)).

Here x and y are byte slices; xy means x followed immediately by y. C measures
a complete fresh encoded stream in bytes. `min` selects the smaller individual
size and `max` the larger; byte units cancel, leaving a dimensionless `f64`.
Lower values can indicate more shared structure for the selected compressor.
`compression_distance_components` exposes the three integer sizes; its
`normalized_distance()` method evaluates the formula without recompression.

## Convention and limits

Celandine uses ordered raw concatenation, without a separator or length prefix.
It retains all framing and checksum bytes and does not subtract overhead, clamp
results, average orders, or force equal inputs to zero. Swapping arguments can
change the answer. Empty inputs are accepted and encoded normally; only two zero
individual *compressed sizes* make the denominator undefined. For the optional
zlib backend C(empty) = 8, so NCD(empty, empty) = 0 by the formula.

Results may be negative or exceed one. With nonnegative integer components and
a positive denominator, the mathematical lower bound is -1; there is no useful
universal upper bound for arbitrary compressors. Results remain finite on
supported platforms. Signed integer differences are formed before conversion,
preserving one-byte differences at large counts; subsequent `f64` conversion and
division can round. Do not assume symmetry, zero self-distance, or the triangle
inequality for an arbitrary backend. Headers dominate short samples; fixed
windows may miss shared patterns across long concatenations. In particular,
zlib's 32 KiB window limits its usefulness for large-file similarity.

## Worked example

A lossless literal format stores one header byte and the input unchanged. For
x = `AB` and y = `CD`, C(x) = C(y) = 3 bytes and C(xy) = 5 bytes, hence NCD =
(5 - 3) / 3 = 2/3. Even identical inputs have that result with this format: it
cannot exploit repetition. This illustrates why the compressor matters.

```
use celandine::compression::{Compressor, compression_distance_components,
    normalized_compression_distance};
struct Literal;
impl Compressor for Literal {
    type Error = std::num::TryFromIntError;
    fn compressed_len(&self, data: &[u8]) -> Result<usize, Self::Error> {
        usize::try_from(data.len() as u128 + 1)
    }
}
let parts = compression_distance_components(b"AB", b"CD", &Literal).unwrap();
assert_eq!((parts.x, parts.y, parts.xy), (3, 3, 5));
assert_eq!(parts.normalized_distance().unwrap(), 2.0 / 3.0);
assert_eq!(normalized_compression_distance(b"AB", b"AB", &Literal).unwrap(), 2.0 / 3.0);
```

## Cost and errors

A successful call performs three compressions, in the order x, y, xy, including
empty inputs. For lengths n and m, copying adds O(n+m) time and auxiliary storage
to the backend's costs. A temporary contiguous buffer is reserved fallibly when
both inputs are nonempty, then freed. With the fixed zlib backend, total time is
O(n+m) and peak auxiliary space O(n+m) plus its fixed workspace. When either
input is empty, concatenation borrows the other and adds no buffer allocation.
This first implementation preserves the existing compressor interface.

# Errors

`CompressionDistanceError` reports backend failure, input-length overflow, failed
concatenation reservation, or a zero denominator. Components remain inspectable
with `compression_distance_components` even for a zero denominator; the method
`normalized_distance()` then returns `ZeroCompressedLength`. Backend allocation
behavior and failures remain governed by that backend's contract.

## History and references

Rudi Cilibrasi and Paul M. B. Vitányi developed compression-based clustering in
work circulated in 2003 and published as *Clustering by Compression* in IEEE
Transactions on Information Theory 51(4), 1523–1545 (2005). Their formula replaces
uncomputable description lengths with practical compressed sizes; the metric
theory depends on properties of a “normal” compressor.

- [Author's corrected paper](https://homepages.cwi.nl/~paulv/papers/cluster.pdf), §§III and VI.
- [Original preprint](https://arxiv.org/abs/cs/0312044).

See the compression module for backend settings and the distinction between
compression measurements and exact mathematical quantities. Symmetric variants,
extra backends, and clustering applications are separate work.
