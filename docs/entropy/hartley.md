# Hartley entropy

## Definition

The base-2 logarithm of the number of observed distinct byte symbols. This is
Hartley entropy of the empirical law, also called Rényi entropy of order zero.

## Short history

Ralph V. L. Hartley's 1928 paper *Transmission of Information* developed a
logarithmic measure of distinguishable possibilities, before Shannon's 1948
probability-weighted entropy. The function here uses that counting principle
on the symbols actually observed in a finite sample, with base 2 for bits.
([Original paper](https://doi.org/10.1002/j.1538-7305.1928.tb01236.x).)

## Why it matters

Hartley isolates the effect of support size from frequency balance. It gives
the largest Shannon entropy possible on the observed support. Comparing the
two helps distinguish a small set of possibilities from a larger set whose
frequencies are uneven, without assuming unseen symbols are present.

## Formula

For nonempty input, let `k = |{b : count[b] > 0}|`:

```text
H0 = log2(k).
```

Here `b` is a byte value, `count[b]` its occurrence count, and the braces collect
the byte values with positive counts. The surrounding bars mean the number of
elements in that set: `k` is how many distinct symbols were observed, not how
long the sample is. `H0` names the entropy of order zero. The base-2 logarithm
expresses the count in bits: 2 possibilities give 1 bit, 4 give 2 bits, and
doubling the number of possibilities adds 1 bit.

Only strictly positive counts contribute to support. There is no probability
cutoff and no assumption that all 256 possible bytes were observed.

## Interpretation

Hartley measures the size of the observed set of possibilities. It ignores
their relative frequencies: both `AB` and `AAAB` have `H0 = 1` bit per symbol.
It equals the Shannon entropy of a uniform law on that set and upper-bounds the
empirical Shannon entropy. It ignores order and does not establish randomness.

## Input domain

Any finite `&[u8]`, including empty input and non-text bytes. Each byte is one
symbol. `hartley_entropy_distribution` accepts a reusable, validated empirical
`Distribution`. There are no parameters or externally supplied probability
vectors; existing count validation applies.

## Output range

For nonempty byte samples, `0 <= H0 <= 8` bits per symbol. There are only 256
possible nonempty support sizes. This is not total sample information, an
integer codeword length, or a normalized score.

## Exact vs estimator

Support is counted exactly and its logarithm is evaluated in `f64`. The target
quantity is exact for the empirical law, not an estimate guaranteed to recover
an unknown source's support entropy. A finite sample may omit possible symbols.
It is not an entropy-rate estimator.

## Edge cases

Empty input returns positive `0.0` by project convention, matching Shannon.
This is an explicit extension, not an evaluation of `log2(0)`; there is no
empirical probability law for an empty sample. A singleton or constant sequence
has support one and returns positive `0.0`. Zero counts are ignored. Even a
single occurrence adds a symbol to support, regardless of the other counts.
All 256 bytes present gives 8 bits.

## Computational complexity

`hartley_entropy` builds the existing histogram in one input pass, then scans
its 256 counts: `O(n + 256)` time, `O(256)` fixed storage, no heap allocation.
`hartley_entropy_distribution` scans existing counts in `O(256)` time with
`O(1)` additional storage and no recounting or allocation.

## Numerical notes

Only the support size (at most 256) is converted to `f64`, exactly. No count
ratios, sums of logarithms, smoothing, or normalization are needed. Even counts
above `2^53` retain their exact presence/absence classification. The nontrivial
rounding comes from `log2`; tests use `1e-12` bits absolute tolerance. Repeated
calls are deterministic within a binary/runtime. Platform math libraries can
differ in the final bits.

## Example

`AAAB` has four observations but only two distinct symbols, `A` and `B`:
`k = 2`, so `H0 = log2(2) = 1` bit per symbol. `ABAB` has the same result despite
different frequencies. `ABC` has three possibilities and `H0 ≈ 1.584963` bits;
the result need not be an integer.

```rust
use celandine::entropy::{hartley_entropy, shannon};

assert_eq!(hartley_entropy(b""), 0.0);
assert_eq!(hartley_entropy(b"AAAA"), 0.0);
assert_eq!(hartley_entropy(b"ABCD"), 2.0);
assert_eq!(hartley_entropy(b"AAAB"), 1.0);
assert!(shannon(b"AAAB") < hartley_entropy(b"AAAB"));
```

## References

R. V. L. Hartley (1928), *Transmission of Information*, Bell System Technical
Journal 7(3), 535–563 ([original paper DOI](https://doi.org/10.1002/j.1538-7305.1928.tb01236.x)),
for logarithmic counting of possibilities. A. Rényi (1961), *On Measures of
Entropy and Information*, Proceedings of the Fourth Berkeley Symposium,
volume 1, 547–561
([paper](https://static.renyi.hu/renyi_cikkek/1961_on_measures_of_entropy_and_information.pdf)),
for the family whose order-zero limit yields log support. Shannon's bound is
also treated in Cover and Thomas (2006), *Elements of Information Theory*,
second edition, chapter 2.

## Related measures

Shannon entropy weights symbols by their observed probabilities. Hartley equals
the limit of Rényi entropy as its order approaches zero from above: on positive
probabilities each `p^alpha` tends to one, while absent symbols contribute zero.
General Rényi entropy is not implemented yet. Neither this function nor Shannon
models dependence between consecutive symbols.
