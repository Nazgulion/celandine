# Fixed byte blocks

Fixed blocks partition the complete prefix of a byte sequence into adjacent,
equal-sized, non-overlapping slices. They enable measurements on disjoint local
regions and preserve an incomplete suffix for explicit handling. This is an exact
positional transformation, not an entropy measure or a statistical estimator.

## Operation, units and rationale

`blocks(data, size)` uses a positive block size b in bytes. For L input bytes,
there are K = floor(L/b) complete blocks. Block k is `data[k*b..(k+1)*b]` for
integer k from zero through K-1. Floor means round down to a whole number of
blocks. The original remainder is `data[K*b..L]`, with R = L mod b bytes;
thus `L = K*b + R` and `0 <= R < b`. Each byte occurs exactly once across the
complete blocks and remainder. Duplicate block contents remain separate occurrences.

The iterator yields only full blocks. Its `.remainder()` returns the original
incomplete suffix, unchanged by forward/backward iteration or skips; it does
not return complete blocks that have not yet been consumed. There is no padding,
wrapping, UTF-8 decoding, or alignment change on reverse traversal. Size counts
bytes, so blocks can split a multi-byte character.

This convention follows Rust's
[`chunks_exact`](https://doc.rust-lang.org/1.90.0/std/primitive.slice.html#method.chunks_exact)
and [`ChunksExact::remainder`](https://doc.rust-lang.org/1.90.0/std/slice/struct.ChunksExact.html#method.remainder).
Celandine adds fallible zero-size validation. This is basic sequence-processing
infrastructure, without a named metric or single historical inventor attributed
here. Equal block lengths make local measurements comparable by extent; disjoint
blocks do not by themselves imply statistical independence.

## Worked examples

For `ABCDEFGH` and size 3, K = floor(8/3) = 2 and R = 2. Starts 0 and 3 yield
`ABC`, `DEF`; `.remainder()` is `GH`. Reverse iteration gives `DEF`, `ABC`,
using the same left-aligned partition. It never yields `FGH`, `CDE`.

```
use celandine::transforms::blocks;
let mut iter = blocks(b"ABCDEFGH", 3)?;
assert_eq!(iter.len(), 2);
assert_eq!(iter.remainder(), b"GH");
assert_eq!(iter.next_back(), Some(b"DEF".as_slice()));
assert_eq!(iter.next(), Some(b"ABC".as_slice()));
assert_eq!(iter.next(), None);
assert_eq!(iter.remainder(), b"GH");
# Ok::<(), celandine::transforms::InvalidBlockSize>(())
```

Compose existing metrics to compare regions:

```
use celandine::{entropy::shannon, transforms::blocks};
let iter = blocks(b"AAAABABAZ", 4)?;
assert_eq!(iter.remainder(), b"Z");
let entropy: Vec<_> = iter.map(shannon).collect();
assert_eq!(entropy, vec![0.0, 1.0]);
# Ok::<(), celandine::transforms::InvalidBlockSize>(())
```

`AAAA` has one byte value and zero bits per symbol. `BABA` has A=2 and B=2,
giving `-2*(1/2)*log2(1/2) = 1` bit per symbol. The tail `Z` is not measured.
These are single-byte entropies within individual blocks, not the entropy of a
probability distribution over block-valued symbols.

## Edge cases and errors

# Errors

Size zero returns `InvalidBlockSize`, even for empty input. Every positive
`usize`, including `usize::MAX`, is valid. Empty input yields no blocks and an
empty remainder. A size greater than L yields no blocks and the entire input
as remainder. A size dividing L leaves an empty remainder; size one yields
all individual bytes. No ordinary invalid size causes a panic.

## Cost and related transforms

The returned standard `ChunksExact<u8>` iterator is cloneable, exact-size,
double-ended and fused. Construction, next/next_back, nth/nth_back, len, count,
last and remainder take O(1) time on the supported Rust 1.90 implementation,
including maximum-integer skips. Traversal costs O(K); iterator storage is O(1).
Extraction borrows the input with no heap allocations or byte copies. Collecting
outputs or applying a metric adds caller costs. Byte Shannon over all complete
blocks takes O(K*b) time and does not automatically include the remainder.

For positive b, yielded slices match `windows(data, b, b)`. The fixed-block API
also exposes the tail. `ngrams(data, b)` advances one byte and generally overlaps;
`windows(data, b, step)` allows a separate stride. For `ABABA`, size two gives
fixed `AB`, `AB` with tail `A`, versus n-grams `AB`, `BA`, `AB`, `BA`.
