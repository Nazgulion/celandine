# Conditional entropy by sequence order

Conditional entropy by order measures uncertainty about a byte after its
preceding `k` bytes are known. It distinguishes predictable local transitions
from a sequence's overall frequency balance. The API measures the empirical
context/next-byte law of one finite byte sequence.

## History

Claude Shannon introduced conditional entropy in his 1948 *A Mathematical Theory
of Communication*. Section 7, Theorem 6 describes uncertainty of the next symbol
conditioned on a preceding block, linking increasing context to source entropy.
See the [original paper](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf).
Our finite-sample convention uses observed counts; it does not supply the
source-law assumptions or limits of Shannon's theorem.

## Formula, observations and units

Let `L` be the input length and `k` the context order, both in bytes. For each
position `t` from `k` through `L-1`, pair the context `x[t-k..t]` with the next
byte `x[t]`. There are `m = L-k` observations if `k < L`, otherwise none.
The first `k` bytes supply context but are not prediction targets. There is no
padding, wrapping, or incomplete context. At `k=0`, every byte is a target and
all observations share the empty context.

For context `c` and following byte `a`, let `N(c,a)` count their observations
and `N(c) = sum_a N(c,a)`. The quantity is

```text
H(next | context) = -sum_(c,a) [N(c,a)/m] * log2[N(c,a)/N(c)]
```

The sum covers observed pairs only. `N(c,a)/m` weights each pair by its fraction
of all observations; `N(c,a)/N(c)` is the probability of `a` within context `c`.
Units are **bits per predicted byte**, not bits per context block. The range
is mathematically `0..=8`, bounded more tightly by the Shannon entropy of the
observed target bytes `data[k..]`. Floating-point results allow rounding.

## Worked examples

- `ABABAB`, order 0: A and B each occur three times, giving 1 bit per byte.
  Order 1: A→B occurs three times and B→A twice. Each context has exactly one
  next byte, so conditional entropy is 0 bits per predicted byte.
- `AAB`, order 1: A→A and A→B occur once each. The only observed context A
  has probabilities 1/2 and 1/2, giving 1 bit per predicted byte.
- `AABA`, order 1: A→A, A→B and B→A occur once each. Context A has one bit
  of uncertainty and weight 2/3; B has zero and weight 1/3. The result is 2/3 bit.

Only contexts with a following byte count toward `N(c)`. For `AABA`, the final
A is a target, but is not another context observation. Subtracting
`block_entropy(data,k)` from `block_entropy(data,k+1)` uses incompatible samples.
The chain rule requires the prefix marginal of the very same `(k+1)`-gram law.

## Domain and edge cases

All `usize` orders are valid. Order zero delegates to byte Shannon entropy.
Empty input or `k >= L` returns positive zero by the library's empty-entropy
convention. One observation (`k=L-1` for nonempty input) also returns zero.
These values do not establish that an unknown source is deterministic.
`usize::MAX` is handled without overflowing `k+1` or allocating.

`conditional_entropy_order_distribution` reuses an `NgramDistribution` of
length `k+1`. Its order is therefore the table length minus one; no separate
order parameter can disagree with that table. Empty and empirically deterministic
transitions return positive zero. Keys can contain any bytes, including zero
and 255, without text decoding.

## Exact quantity and estimation limits

This is Shannon conditional entropy of the empirical pair law, evaluated in
`f64`. Interpreted as an estimate of unknown-source conditional entropy, it has
finite-sample bias and overlapping, potentially dependent observations. Large
orders can leave each context with one observation and a misleading zero result.
There is no smoothing, bias correction, confidence interval or source entropy-rate
claim. Conditional entropies at different orders can increase on finite data
because the target sets change: `AAB` has order-zero entropy about 0.918296 but
order-one entropy 1. Reversal and arbitrary byte shuffling need not preserve it.

## Cost and numerical behavior

For positive `k` with more than one observation, the byte API constructs the
existing `(k+1)`-gram tree. With `s` distinct observed pairs, worst-case time is
`O(m*(k+1)*log(s+1) + s*(k+1))` and heap storage is `O(s)`. Keys borrow bytes;
long common prefixes can make comparisons expensive. Order zero takes
`O(L+256)` time and no heap; positive orders with at most one observation take
constant time and no heap.

The reuse API groups lexicographically adjacent keys by their k-byte prefix in
`O(s*(k+1))` worst-case time. It keeps at most 256 successor counts in a fixed
array (2048 bytes on 64-bit systems), plus iterator/scalar state, without new
heap allocation. Each context total is an exact subset of the sample count.

Entropy terms are evaluated directly with compensated summation, avoiding
subtraction of large entropies. A majority successor's logarithm uses its
integer complement and `ln_1p` to retain small probabilities of alternatives.
Counts above `2^53` may round on conversion to `f64`. Order-zero reuse can differ
from the byte Shannon path in final rounding bits. Results are not clamped.
Independent 100-digit fixtures use `1e-12` absolute bit tolerance; this is a
validation threshold, not a universal numerical error guarantee.
