# Min-entropy

## Definition

Min-entropy is the surprise of the most likely symbol in a distribution. Here
probabilities are empirical frequencies of bytes in a finite sequence.

## Short history

Min-entropy is the infinite-order limit of the entropy family introduced by
Alfréd Rényi in *On Measures of Entropy and Information* (1961)
([original paper record](https://digicoll.lib.berkeley.edu/record/112906)).
As the order grows, the largest probability determines the limit. This describes
its mathematical origin, without assigning an invention date to the later name.

## Why it matters

Averages can obscure a dominant symbol. Min-entropy isolates that dominance:
it depends only on the largest probability. Two distributions can therefore
have the same min-entropy while their Shannon or collision entropies differ.

## Formula

```text
H_inf = -log2(p_max)
p_max = m / n
m = max_i c_i
```

`i` indexes byte symbols, `c_i` is their integer count, `m` is the largest count,
and `n` is the nonzero sample size. Dividing `m` by `n` gives the dimensionless
probability `p_max`; its negative base-2 logarithm gives **bits per symbol**.
Equivalently, this is `min_i(-log2(p_i))` over positive empirical probabilities:
the most likely symbol has the smallest surprise.

## Interpretation

Among fixed guesses of one symbol drawn from the empirical law, the greatest
success probability is `p_max = 2^(-H_inf)`. This statement concerns the empirical
law and assumes no model of the input source. It does not describe guesses made
with additional information. Sequence order, temporal dependence, unseen symbols,
and an unknown source's entropy rate are not captured.

## Input domain

Any byte slice or validated count-backed `Distribution`. No order parameter is
needed; the return type is `f64`. Arbitrary probability vectors, smoothing,
normalization, and source bias corrections are not part of this API.

## Output range

For nonempty input, `0 <= H_inf <= H2 <= H_Shannon <= log2(k) <= 8`, where `k`
is observed support size. Uniform frequencies give `log2(k)`; a constant sequence
gives zero. Floating-point rounding can slightly exceed a mathematical bound;
results are not clamped.

## Exact vs estimator

The definition is exact for the empirical law, with approximate `f64` evaluation.
Used to infer a source's min-entropy, it is a plug-in estimate with finite-sample
effects and no statistical confidence guarantee.

## Edge cases

Empty input returns positive zero by project convention and has no empirical
`p_max`. Constant and one-byte inputs return positive zero because `p_max=1`.
Zero-count symbols do not affect the maximum of a nonempty table. Ties between
most frequent symbols require no tie-breaking in the public result.

## Computational complexity

The byte function takes `O(n + 256)` time and fixed `O(256)` histogram storage.
The distribution function scans 256 counts and takes constant additional storage.
Neither function allocates on the heap or recounts an existing distribution.

## Numerical notes

The implementation reuses Rényi at positive infinity. For a dominant symbol,
it computes the logarithm using the integer complement `n-m` before conversion
to `f64`. This avoids returning a false zero if `m/n` rounds to one for a
nonconstant law. It does not subtract two large, almost equal logarithms.
The existing absolute `1e-12`-bit test tolerance applies, with extra relative
checks on tiny positive values. It is not a universal error bound or a promise
of bitwise identity across platforms and toolchains.

## Example

`AAAB` has counts 3 and 1. The largest probability is `3/4`, so min-entropy is
`-log2(3/4) = log2(4/3)`, approximately **0.415037 bits per symbol**. The best
fixed guess is `A`, which succeeds with probability 0.75 under the empirical law.
For `ABCD`, the largest probability is 1/4 and min-entropy is 2 bits per symbol.

```rust
use celandine::entropy::min_entropy;
assert_eq!(min_entropy(b"ABCD"), 2.0);
assert_eq!(min_entropy(b"AAAA"), 0.0);
assert!((min_entropy(b"AAAB") - 0.4150374992788438).abs() < 1e-12);
```

## References

A. Rényi (1961), *On Measures of Entropy and Information*, Proceedings of the
Fourth Berkeley Symposium on Mathematical Statistics and Probability, volume 1,
547–561 ([original paper record](https://digicoll.lib.berkeley.edu/record/112906)).
For `p_max > 0`, factor `p_max^alpha` out of the Rényi power sum. The remaining
sum lies between 1 and the support size, so its logarithm divided by `alpha-1`
vanishes as the order grows. The limit is `-log2(p_max)`.

## Related measures

[`crate::entropy::renyi_entropy`] returns the same value at positive infinity.
Collision entropy measures matching-draw concentration; Shannon averages symbol
surprise; Hartley counts observed possibilities. These are single-symbol empirical
measures and do not account for sequence dependence.
