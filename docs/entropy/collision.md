# Collision entropy

## Definition

Collision entropy measures how surprising it is for two independent draws from
the same distribution to give the same symbol. Here that distribution is formed
from the byte frequencies of the input sequence.

## Short history

Collision entropy is the order-two member of Alfréd Rényi's entropy family,
introduced in his 1961 paper *On Measures of Entropy and Information*
([original paper record](https://digicoll.lib.berkeley.edu/record/112906)).
Substituting order two in his formula gives the negative logarithm of the sum
of squared probabilities; that sum has the matching-draw interpretation below.

## Why it matters

Squaring probabilities makes common symbols contribute more strongly than rare
ones. This reveals concentration that support counting alone cannot distinguish.
The dedicated API names this interpretation without requiring an order parameter.

## Formula

```text
H2 = -log2(C)
C = sum_i p_i^2 = sum_i c_i^2 / n^2
p_i = c_i / n
```

`i` ranges over observed symbols, `c_i` is a symbol's count, `n` is sample size,
and `p_i` is its empirical probability. Independent draws both give symbol `i`
with probability `p_i * p_i`; adding these disjoint outcomes gives the matching
probability `C`. The negative base-2 logarithm converts that probability to
**bits per symbol**, consistent with the other single-symbol entropy APIs.
`C` is dimensionless. The function returns `H2`, not `C`.

## Interpretation

Larger values mean matches are less likely and frequencies are more spread out.
Draws are with replacement: among all `n^2` ordered pairs of sample positions,
including a position paired with itself, `sum(c_i^2)` match. This does not count
adjacent equal bytes, and does not use the without-replacement expression
`sum(c_i*(c_i-1))/(n*(n-1))`. It ignores sequence order and temporal dependence.
Independence describes the two hypothetical draws, not the input's generating
process. The measure does not infer a source's entropy rate or unseen symbols.

## Input domain

Any byte slice or validated count-backed `Distribution`. The order is fixed at
two, so the API returns `f64` directly. There is no probability-vector input,
normalization, smoothing, or parameter to reject.

## Output range

For nonempty input, `0 <= H2 <= H_Shannon <= log2(k) <= 8`, where `k` is observed
support size. Uniform frequencies give `H2 = log2(k)`. Floating-point results may
slightly exceed a mathematical bound and are not clamped.

## Exact vs estimator

The mathematical quantity is exact for the empirical law and approximately
evaluated in `f64`. As an estimate of an unknown source's collision entropy it
has finite-sample effects. It is not a bias-corrected estimate of source matching
probability or entropy.

## Edge cases

Empty input returns exact positive zero by project convention; it has no
normalized empirical law and no matching probability. Constant and one-byte
inputs return exact positive zero because the matching probability is one.
Unobserved symbols contribute zero. All 256 bytes, including zero, are valid.

## Computational complexity

The byte function takes `O(n + 256)` time and fixed `O(256)` histogram storage.
The distribution function takes `O(256)` time and constant additional storage,
without recounting the input. Neither allocates on the heap.

## Numerical notes

The implementation reuses [`crate::entropy::renyi_entropy_distribution`] at
order two, including maximum scaling, compensated summation, and integer
complements for logarithms near probability one. It avoids squaring integer
counts in `usize`, which could overflow, or directly rounding a matching
probability close to one before taking its logarithm. Agreement tests use an
absolute `1e-12`-bit tolerance, with extra relative checks for tiny positive
results. These checks are not a universal error bound. Evaluation is deterministic
within a binary/runtime; cross-platform bitwise identity is not promised.

## Example

`AAAB` has probabilities `3/4` and `1/4`. Its matching probability is
`C = 9/16 + 1/16 = 10/16`, so `H2 = log2(8/5)`, approximately
0.678072 bits per symbol. For `AB`, `C = 1/4 + 1/4 = 1/2` and `H2 = 1` bit.
Repeated positions are allowed: `AB` has four ordered pairs, two of which match.

```rust
use celandine::entropy::collision_entropy;
assert_eq!(collision_entropy(b"AB"), 1.0);
assert_eq!(collision_entropy(b"ABCD"), 2.0);
assert!((collision_entropy(b"AAAB") - 0.6780719051126377).abs() < 1e-12);
```

## References

A. Rényi (1961), *On Measures of Entropy and Information*, Proceedings of the
Fourth Berkeley Symposium on Mathematical Statistics and Probability, volume 1,
547–561 ([original paper record](https://digicoll.lib.berkeley.edu/record/112906)).
The matching-pair interpretation follows from independence and disjoint outcomes:
`P(X=Y) = sum_i P(X=i)P(Y=i)` for draws with the same law.

## Related measures

[`crate::entropy::renyi_entropy`] generalizes this measure; order two is identical
to this API. Shannon averages individual symbol surprise, while Hartley counts
observed possibilities. Min-entropy is the infinite-order Rényi limit; a dedicated
min-entropy API remains future work.
