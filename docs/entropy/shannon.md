# Shannon entropy

## Definition

The average surprise of a symbol drawn according to the observed byte
frequencies. Common symbols contribute less surprise than rare ones. The
mathematical name for a symbol's surprise is its self-information.

## Short history

Claude E. Shannon introduced this probability-weighted measure of uncertainty
in his 1948 paper *A Mathematical Theory of Communication*. It became a central
quantity in information theory, connecting symbol probabilities with the limits
of lossless coding. This library applies the definition to frequencies measured
in a finite sample.
([Original paper, Part I, section 6](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf).)

## Why it matters

Shannon entropy distinguishes distributions with the same number of distinct
symbols but different frequency balances. For example, `ABAB` has more average
symbol uncertainty than `AAAB`. This makes it useful for comparing empirical
distributions; a result alone does not describe sequence order or predict the
compressed size of a particular finite string.

## Formula

For counts `c[b]` and sample size `n > 0`, set `p[b] = c[b] / n`:

```text
H = -sum over b with p[b] > 0 of p[b] * log2(p[b]).
```

Read the formula as a weighted average:

- `b` is a byte value, `c[b]` its occurrence count, and `n` the sample length.
- `p[b] = c[b] / n` is its observed fraction of the sample.
- `-log2(p[b])` is its surprise in bits: probability `1/2` gives 1 bit;
  probability `1/4` gives 2 bits.
- Multiplying by `p[b]` weights that surprise by how often the symbol occurs.
- Summing those contributions gives the average, in bits per symbol. Base 2
  selects bits as the unit; absent symbols contribute zero.

## Interpretation

A constant sample has zero symbol uncertainty. Equal frequencies maximize
entropy at fixed support size. This describes single-symbol frequencies;
it ignores ordering and does not establish randomness or independence.

## Input domain

Any finite `&[u8]`, including zero and non-text bytes. Each byte is one symbol.
`shannon_distribution` accepts the same empirical law as a reusable
`Distribution`. There are no tuning parameters.

## Output range

For nonempty samples with `k` observed symbols, `0 <= H <= log2(k) <= 8`
bits per symbol, subject to floating-point rounding. This is neither total
sample information nor a normalized score.

## Exact vs estimator

The target quantity is exact for the empirical law, evaluated approximately
with `f64`. As an estimate of unknown source entropy it is a plug-in estimator;
finite samples introduce bias. It is not an entropy-rate estimator.

## Edge cases

Empty input returns positive `0.0` by project convention: an empty sample has
no empirical probability law. A singleton or constant sample returns exactly
positive `0.0`. Absent symbols contribute zero (`0 log2(0) = 0`). All byte values
are supported. No input slicing or character decoding is performed.

## Computational complexity

`shannon` counts in one input pass, then scans 256 probability entries:
`O(n + 256)` time and `O(256)` fixed auxiliary storage. It allocates no heap
memory. `shannon_distribution` takes `O(256)` time with `O(1)` additional storage.

## Numerical notes

Uses `f64`, direct probability division, and sequential summation in byte order.
Counts above `2^53` may round on conversion to `f64`. No smoothing,
renormalization, or range clamping occurs. Tests use `1e-12` bits absolute
tolerance, not a universal error guarantee. Repeated calls are deterministic
within a binary/runtime; different platform `log2` implementations can differ.

## Example

`ABCD` gives four probabilities of `1/4`, so entropy is `4 * (1/4) * 2 = 2` bits.

For `AAAB`, the probabilities are `3/4` and `1/4`. Their contributions are
`-(3/4) log2(3/4) ≈ 0.311278` and `-(1/4) log2(1/4) = 0.5`, giving
`H ≈ 0.811278` bits per symbol. `ABAB` instead gives two equal probabilities
and `H = 1` bit per symbol.

```rust
use celandine::entropy::shannon;

assert_eq!(shannon(b"ABCD"), 2.0);
assert_eq!(shannon(b"ABAB"), 1.0);
assert_eq!(shannon(b"AAAA"), 0.0);
assert_eq!(shannon(b""), 0.0);
```

## References

Shannon (1948), *A Mathematical Theory of Communication*, Part I, section 6
([paper](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf));
Cover and Thomas (2006), *Elements of Information Theory*, second edition,
section 2.1; MacKay (2003), *Information Theory, Inference, and Learning
Algorithms*, chapter 2.

## Related measures

[`crate::entropy::hartley_entropy`] measures observed support and upper-bounds
empirical Shannon entropy. [`crate::entropy::renyi_entropy`] generalizes Shannon
through an order parameter, with exact order one delegating to this function.
Block entropy and entropy-rate estimators remain future work and address sequence
dependence.
