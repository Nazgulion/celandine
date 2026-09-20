# Shannon entropy with an explicit logarithm base

## Definition and purpose

Shannon entropy averages symbol surprise using observed byte frequencies. Its
logarithm base selects the unit: base 2 gives bits, base `e` gives nats, and base
10 gives decimal information units, all per symbol. Changing units lets callers
compare results with calculations expressed in another convention; it does not
change the underlying distribution or normalize entropy to a fixed range.

## Historical context

Shannon's 1948 *A Mathematical Theory of Communication* explains the choice of
logarithm base as a choice of information unit in its introduction. Section 6
then derives the probability-weighted entropy formula. These APIs expose that
unit choice for the same empirical quantity as [`crate::entropy::shannon`].
See [Shannon's original paper, introduction and Part I, section 6](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf).

## Formula and units explained

```text
p_i = c_i / N
H_b = -sum over i with c_i > 0 of p_i * log_b(p_i)
    = H_2 / log2(b).
```

Here `i` identifies a byte value, `c_i` counts its occurrences, `N > 0` is the
total number of bytes, and `p_i` is its dimensionless observed probability.
`b` is the dimensionless logarithm base, required to be finite and greater than
one. `log_b(p_i)` means `ln(p_i)/ln(b)`, where `ln` is the natural logarithm.
Each `-log_b(p_i)` is surprise in the selected units, weighted by its probability
before summation. `H_2` is the existing base-2 entropy in bits per symbol.

| Base | Unit | Uniform `ABCD` |
| --- | --- | ---: |
| `2.0` | bits per symbol | `2.0` |
| `std::f64::consts::E` | nats per symbol | approximately `1.3862943611198906` |
| `10.0` | decimal information units per symbol | approximately `0.6020599913279624` |
| `4.0` | base-4 information units per symbol | `1.0` |

`E` is the nearest available `f64` approximation of the mathematical constant
`e`. Like any floating-point parameter, its supplied value determines the base.

## Worked example and API

`ABCD` has four counts of one, sample size `N=4`, and probabilities `1/4`.
In bits each term is `-(1/4)*log2(1/4) = 0.5`, totaling two bits per symbol.
In base 10 each term is about `0.150515`, totaling `2/log2(10) ≈ 0.602060`.
The uncertainty is the same; only the unit changes.

```rust
use celandine::entropy::{shannon_with_base, shannon_distribution_with_base};
use celandine::distribution::Distribution;

assert_eq!(shannon_with_base(b"ABCD", 2.0), Ok(2.0));
assert!((shannon_with_base(b"ABCD", std::f64::consts::E)? - 4.0_f64.ln()).abs() < 1e-12);
let d = Distribution::from_bytes(b"ABCD");
assert!((shannon_distribution_with_base(&d, 10.0)? - 0.6020599913279624).abs() < 1e-12);
assert!(shannon_with_base(b"", 1.0).is_err());
# Ok::<(), celandine::entropy::InvalidLogBase>(())
```

`shannon_with_base(data, base)` counts bytes. `shannon_distribution_with_base(d,
base)` reuses an existing single-byte distribution; it does not recount input.
Both return `Result<f64, InvalidLogBase>`. The existing `shannon` and
`shannon_distribution` APIs continue returning `f64` in bits.

## Domain, range, and errors

Input is a finite byte sequence or its count-backed `Distribution`. For
nonempty support of size `k`, the mathematical range is
`0 <= H_b <= log_b(k) = log2(k)/log2(b)`, with `k <= 256`. The result is not
necessarily bounded by eight in the selected unit. Equal frequencies maximize
entropy at fixed support. Comparing entropies requires matching units.

Bases must be finite and strictly greater than one. `InvalidLogBase` rejects
NaN, both infinities, zero (either sign), one, negative values, and positive
values below one. Although logarithms with bases between zero and one exist,
they are decreasing and would reverse the entropy sign; this API excludes them.
No special infinite-base limit or automatic base substitution is used.

Validation happens before counting or evaluating a distribution, even for
empty and constant input. Valid bases return positive zero for empty input by
the existing project convention, and for constant samples mathematically.
Zero-probability terms contribute zero without evaluating `log(0)`.

## Exactness and limits

The target is exact entropy of the empirical byte law, evaluated approximately
as `f64`. It describes symbol frequencies, not ordering, source entropy rate,
or the compressed length of a finite sequence. Changing bases does not correct
finite-sample bias or provide a source model. There is no smoothing,
renormalization, clamping, or support normalization.

## Complexity and numerical behavior

The byte API takes `O(N + 256)` time and `O(256)` fixed storage; the distribution
API takes `O(256)` time and `O(1)` extra storage. Both allocate no heap memory.
Invalid bases return in constant time before input counting.

Evaluation divides the existing Shannon result by `base.log2()` once. Base 2
therefore agrees bit-for-bit with the existing API. Every accepted `f64` base,
including the next representable value above one and `f64::MAX`, has a finite,
positive divisor. For this byte API the output remains finite. Absolute rounding
error is magnified by `1/log2(base)` near one; a universal `1e-12` tolerance in
every selected unit would be inappropriate. Parameter rounding near one is also
significant: if a caller's expression rounds to exactly one, it is rejected.

The underlying probability conversion and summation are unchanged, including
limited relative accuracy at extreme count imbalance above `2^53`. Conversion
does not restore information lost there. Independent fixtures use a `1e-12`
absolute tolerance after conversion back to bit scale; fixtures with total counts at most `2^53` also
require `1e-12` relative accuracy. Neither is a universal platform error bound.
Counts remain available for callers needing another numerical method.

## References and related APIs

Shannon (1948), *A Mathematical Theory of Communication*, introduction and Part
I, section 6; Cover and Thomas (2006), *Elements of Information Theory*, second
edition, section 2.1. The [original paper](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf)
is the definition and unit-conversion reference. [`crate::entropy::shannon`]
documents the shared base-2 calculation and its interpretation. Other entropy
families keep their existing unit conventions.
