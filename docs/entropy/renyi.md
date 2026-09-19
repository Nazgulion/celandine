# Rényi entropy

## Definition

Rényi entropy measures the diversity of symbol probabilities using an order
parameter, `alpha`, that controls their relative influence. Here the probabilities
come from byte counts in a finite sequence.

## Short history

Alfréd Rényi introduced this family in his 1961 paper *On Measures of Entropy and
Information*, extending Shannon's entropy through generalized means. Shannon's
quantity is recovered as the order approaches one; logarithmic support counting
is recovered at zero ([original paper record](https://digicoll.lib.berkeley.edu/record/112906)).

## Why it matters

One number can hide how diversity depends on rare versus frequent symbols.
Orders below one give rare symbols greater influence; orders above one emphasize
common symbols. Comparing orders distinguishes distributions that have the same
support but different frequency balance.

## Formula

For `alpha > 0`, `alpha != 1`:

```text
H_alpha = log2(sum_i p_i^alpha) / (1 - alpha)
p_i = c_i / n
```

Here `i` runs over observed symbols, `c_i` is a positive integer count, `n` is the
sample size, `p_i` is its empirical probability, and `alpha` is dimensionless.
Raise each probability to the order, add the results, take the base-2 logarithm,
and divide by `1 - alpha`. The result is in **bits per symbol**.

The continuous extensions are `H_0 = log2(k)` (Hartley, with `k` observed symbols),
`H_1 = -sum_i p_i log2(p_i)` (Shannon), and
`H_infinity = -log2(max_i p_i)`. Order two is `-log2(sum_i p_i^2)`.

## Interpretation

Entropy is nonincreasing as order increases. It depends on frequencies, not the
sequence's arrangement. It does not measure temporal dependence, entropy rate,
or the full support of an unknown source.

## Input domain

Byte slices or count-backed empirical `Distribution` values. The order must be a
nonnegative `f64`; positive infinity is accepted. Negative zero denotes zero.
Arbitrary floating-point probability vectors are not accepted or normalized.

## Output range

For nonempty data, `0 <= H_alpha <= log2(k) <= 8` bits per symbol. A uniform law
on `k` observed symbols gives `log2(k)` for every valid order. Floating-point
rounding can slightly exceed a mathematical bound; results are not clamped.

## Exact vs estimator

This is the exact mathematical Rényi entropy of the empirical law, evaluated
approximately in `f64`. Used to infer an unknown source's entropy, it is a
plug-in estimate; unseen symbols and finite-sample effects remain unresolved.

## Edge cases

Empty input returns positive zero by project convention, although no normalized
empirical law exists there. Constant input returns positive zero. Zero-count
symbols are excluded; no `0^0` or logarithm of zero is evaluated. Exact orders
zero and one reuse the existing Hartley and Shannon implementations.

## Errors

Negative orders, negative infinity, and NaN return `InvalidRenyiOrder`, including
on empty or constant inputs. Validation occurs before counting bytes.

## Computational complexity

The byte entry point takes `O(n + 256)` time and fixed `O(256)` stack storage.
The distribution entry point scans at most 256 counts, uses constant auxiliary
storage, and never recounts the original sequence. Neither entry point allocates.

## Numerical notes

Near one, `expm1` and `log1p` avoid cancellation; only exact order one delegates
to Shannon. Elsewhere maximum-scaled powers avoid overflow at very large orders.
Compensated sums use deterministic byte order. Counts near `usize::MAX` retain
small complements when taking logarithms near probability one. Tests use an
absolute `1e-12`-bit tolerance, not a universal error or relative-error guarantee.
Results need not be bitwise equal across platforms or toolchains.

## Example

For `AAAB`, counts are 3 and 1, so probabilities are 3/4 and 1/4. At order two,
`sum(p_i^2) = 9/16 + 1/16 = 10/16`, giving `H_2 = log2(8/5)`, approximately
0.678072 bits per symbol. The same distribution has Hartley 1, Shannon 0.811278,
and order-infinity entropy 0.415037 bits per symbol.

```rust
use celandine::entropy::{renyi_entropy, InvalidRenyiOrder};
assert_eq!(renyi_entropy(b"ABCD", 2.0), Ok(2.0));
assert!((renyi_entropy(b"AAAB", 2.0).unwrap() - 0.6780719051126377).abs() < 1e-12);
assert_eq!(renyi_entropy(b"", -1.0), Err(InvalidRenyiOrder));
```

## References

A. Rényi (1961), *On Measures of Entropy and Information*, Proceedings of the
Fourth Berkeley Symposium, volume 1, 547–561
([original paper record](https://digicoll.lib.berkeley.edu/record/112906)).
N. J. Higham (2021), [*What Is the Log-Sum-Exp Function?*](https://nhigham.com/2021/01/05/what-is-the-log-sum-exp-function/),
for maximum scaling and accurate logarithms of small corrections.

## Related measures

Hartley and Shannon are the orders zero and one. Collision entropy is order two;
min-entropy is the infinite-order limit. They can be evaluated through this API;
[`crate::entropy::collision_entropy`] exposes order two directly. A dedicated
min-entropy entry point remains a later milestone.
