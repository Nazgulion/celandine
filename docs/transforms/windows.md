# Sliding byte windows

Sliding windows select local portions of a sequence so existing metrics can
show where its properties change. This is an exact positional transformation,
not a statistic or an estimator. It borrows the original bytes without copying.

## Operation and units

`windows(data, size, step)` starts at byte offsets 0, step, 2*step, and so on.
At each selected start s, it yields `data[s..s+size]` only when the entire window
fits. Size and step are positive counts of bytes, not characters or samples of
another type. There is no padding, wrapping, or partial tail.

For L input bytes, width w = size and stride h = step, the number of windows is
zero if w > L; otherwise it is `1 + floor((L-w)/h)`. Each slice has exactly w
bytes. The floor discards any final fractional step. Valid start offsets satisfy
`s = k*h` and `s+w <= L` for integers k beginning at zero. This formula explains
the positions; the implementation bounds window ordinals before calculating offsets, so large
parameters and skips cannot overflow index arithmetic.

When step < size, windows overlap; when step = size they are adjacent; when
step > size there are gaps. With step one, results equal `ngrams(data, size)`.
These are positional occurrences, so duplicate byte strings remain present.
Backward iteration reverses the selected forward windows; it does not realign
the grid to the input's right edge.

## Worked examples

For `ABCDEFGH`, size 3 and step 2 select starts 0, 2, 4: `ABC`, `CDE`, `EFG`.
The count is `1 + floor((8-3)/2) = 3`. Start 6 has only `GH`, so it is omitted.
Reverse iteration gives `EFG`, `CDE`, `ABC`, not windows starting at 5, 3, 1.

```
use celandine::transforms::windows;
let blocks: Vec<_> = windows(b"ABCDEFGH", 3, 2)?.collect();
assert_eq!(blocks, vec![b"ABC", b"CDE", b"EFG"]);
let backwards: Vec<_> = windows(b"ABCDEFGH", 3, 2)?.rev().collect();
assert_eq!(backwards, vec![b"EFG", b"CDE", b"ABC"]);
# Ok::<(), celandine::transforms::InvalidWindowParameters>(())
```

Compose with an existing metric to inspect local variation:

```
use celandine::{transforms::windows, entropy::shannon};
let entropy: Vec<_> = windows(b"AAAABABA", 4, 2)?.map(shannon).collect();
assert_eq!(entropy, vec![0.0, shannon(b"AABA"), 1.0]);
# Ok::<(), celandine::transforms::InvalidWindowParameters>(())
```

Here entropy is in bits per byte-symbol within each window. The middle window
has counts A=3, B=1, giving about 0.811278 bits per symbol. Window starts 0, 2, 4
reveal a change from constant to balanced bytes. Overlapping windows share data;
the transform does not make their measurements independent or select a model.

## Edge cases and errors

Empty input or size greater than input length produces an empty iterator for
valid parameters. A step greater than the input length still selects the first
window if it fits. All positive `usize` values, including `usize::MAX`, are valid
parameters. Bytes are not decoded as UTF-8; windows may split a multi-byte character.

# Errors

`InvalidWindowParameters::ZeroSize` rejects size zero; `ZeroStep` rejects step
zero. Validation occurs even for empty or undersized input. Size is checked first
when both parameters are zero. No ordinary invalid parameter causes a panic.

## Iterator and computational cost

The returned iterator is cloneable, exact-size, double-ended and fused. It yields
borrowed immutable byte slices with the input's lifetime. Construction and each
front/back step take O(1) time; `nth`, `nth_back`, `len`, `count` and `last`
also take O(1) time, including skips of `usize::MAX`; full traversal is O(K) for K windows. Iterator
storage is O(1) and extraction performs no heap allocations or byte copies.
Reading window contents or computing a metric adds that operation's cost; for
example, naively applying byte Shannon to every window takes O(K*size) time.
Collecting outputs allocates in the caller. The transform does not scan content
or maintain incremental metric state.

## Background and references

This is basic sequence-processing infrastructure with conventions matching Rust's [slice windows](https://doc.rust-lang.org/1.90.0/std/primitive.slice.html#method.windows)
and [step_by](https://doc.rust-lang.org/1.90.0/std/iter/trait.Iterator.html#method.step_by)
operations. It has no named metric or single historical inventor attributed here.
Celandine adds fallible validation and a bounded ordinal iterator, guaranteeing
constant-time skips and fused iteration on the supported Rust 1.90 compiler.
See `ngrams` for step-one extraction and the entropy APIs for composable metrics.
