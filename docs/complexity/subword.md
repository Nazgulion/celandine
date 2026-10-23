Counts distinct contiguous byte blocks of length `n` in a finite sequence.

Subword complexity, also called **factor complexity**, measures observed pattern
diversity. Unlike single-byte entropy, it can distinguish arrangements having the
same byte frequencies. Here “subword” always means contiguous, including overlaps;
it does not mean a subsequence with gaps.

## Short history

Morse and Hedlund studied block growth in symbolic dynamics in 1938, relating
the diversity of finite blocks to the structure of infinite symbolic sequences.
Their [original paper](https://doi.org/10.2307/2371264) provides the historical
setting. Cassaigne and Nicolas's [Factor complexity chapter](https://doi.org/10.1017/CBO9780511777653.005)
surveys this notion and its terminology. This API counts blocks in a supplied
finite sample; infinite-sequence theorems require additional assumptions.

## Definition, terms, and units

For a byte sequence `x` of length `L`, define

```text
p_x(n) = |{ x[i..i+n] : 0 <= i <= L-n }|,  1 <= n <= L
p_x(0) = 1
p_x(n) = 0,                              n > L
```

`n` is block length in bytes; `i` is a zero-based start position. The braces
form a set, so repeated occurrences contribute once; `|...|` is its cardinality.
The output is an exact `usize` count of distinct blocks, with no logarithm,
normalization, floating-point tolerance, or statistical estimator. No circular
wrapping, padding, text decoding, or source model is used.

The empty word is the one length-zero block, including for empty input.
This combinatorial convention is deliberately different from `transforms::ngrams`
and `distribution::ngram_counts`, which require positive block lengths.
Every `usize` length is accepted, including `usize::MAX`; oversized lengths yield zero.

For `1 <= n <= L`, `1 <= p_x(n) <= min(L-n+1, 256^n)`.
`p_x(1)` is the observed byte support, and `p_x(L)=1` for nonempty input.
Reversing a sequence or bijectively renaming its bytes preserves these counts.
Arbitrary reordering and repeating the sample need not preserve them.

## Worked example

`ABABA` has length-two occurrences `AB, BA, AB, BA`. The distinct blocks are
`{AB, BA}`, so `p(2)=2`, although there are four occurrences. Its full profile
for lengths zero through five is `[1, 2, 2, 2, 2, 1]`.
`AABB` instead has `{AA, AB, BB}` at length two, giving three, while `ABAB`
has two; both have identical single-byte counts.

```
use celandine::complexity::subword_complexity;
assert_eq!(subword_complexity(b"ABABA", 2), 2);
assert_eq!(subword_complexity(b"AABB", 2), 3);
assert_eq!(subword_complexity(b"", 0), 1);
assert_eq!(subword_complexity(b"ABABA", usize::MAX), 0);
```

## Algorithm and cost

Length zero, oversized lengths, and full-length blocks take constant time without
allocation. Length one uses the fixed byte histogram in `O(L + 256)` time with
no heap allocation. Interior lengths `1 < n < L` reuse the exact n-gram count
table, then discard it. For `m=L-n+1` occurrences and `k=p_x(n)` distinct blocks,
the worst-case cost is `O(m * n * log(k+1))` time and `O(k)` temporary tree storage.
Keys borrow input bytes; blocks are not copied. Existing `NgramCounts` callers
can read `support_size()` in constant time without recounting.

`subword_complexity_profile` computes requested lengths lazily. It is useful for
short complete profiles or bounded prefixes; consuming every length on a large
sample can be expensive. No suffix index or cross-length cache is built.

## Interpretation and limits

These counts describe observed blocks, not unseen patterns of an unknown source.
Finite profiles need not increase: `ABCD` gives `[1,4,3,2,1]` because long blocks
have fewer valid positions. Counts alone ignore block frequencies and identities.
They are neither Kolmogorov complexity nor entropy, compression size, randomness
certificates, or proof of periodicity of an infinite generating process.
