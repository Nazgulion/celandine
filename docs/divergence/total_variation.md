# Total variation distance

Total variation measures how much two probability distributions differ. It is
useful for comparing byte composition even when samples have different lengths.
It ignores position: `AABB` and `ABAB` have distance zero.

## Definition and interpretation

`TV(P,Q) = (1/2) sum_i |p_i - q_i|`, over all 256 byte values `i`.
Here `p_i = a_i/N` and `q_i = b_i/M` are relative frequencies, `a_i` and `b_i`
are counts, and `N` and `M` are the respective positive sample sizes. Absolute
differences are summed and halved. The result is dimensionless, with no logarithm
base, in `[0,1]`: equal laws give zero and disjoint supports give one.

Equivalently, TV is the largest difference in probability assigned to any set
of byte values. It obeys symmetry and the triangle inequality as a distance on
distributions. Different sequences may have identical distributions, so it is
not a metric distinguishing individual sequences.

## Historical context

Total variation is a classical probability distance, used, among other things,
to quantify convergence of Markov chains to equilibrium. Levin and Peres's
2017 *Markov Chains and Mixing Times*, with contributions by Wilmer, presents
the event-based definition and proves the half-L1 formula in Chapter 4,
Proposition 4.2. This is an authoritative exposition, not an attribution of
the measure's invention. [Author-hosted text](https://pages.uoregon.edu/dlevin/MARKOV/mcmt2e.pdf).

## Worked example

`AAAB` has probabilities `(3/4, 1/4)` on `(A,B)`; `AB` has `(1/2, 1/2)`.
Thus `TV = (|3/4-1/2| + |1/4-1/2|)/2 = (1/4+1/4)/2 = 1/4`.
The event `{A}` has probability differing by 0.25, the largest possible gap.

```rust
use celandine::divergence::{total_variation_distance, EmptyDistribution};
assert_eq!(total_variation_distance(b"AAAB", b"AB"), Ok(0.25));
assert_eq!(total_variation_distance(b"AABB", b"ABAB"), Ok(0.0));
assert_eq!(total_variation_distance(b"AAA", b"BB"), Ok(1.0));
assert_eq!(total_variation_distance(b"", b"AB"), Err(EmptyDistribution));
```

## Domain and errors

Inputs are byte slices of arbitrary, possibly unequal lengths. Byte labels are
shared across inputs; no alignment, text decoding, smoothing, or relabeling is
performed. Unobserved bytes have zero probability and require no special handling.

Returns `Result<f64, EmptyDistribution>`. Either empty input, including two empty
inputs, returns an error before counting. An empty empirical state has no
unit-mass law; the entropy-specific zero convention does not define a distance
to that state. Constant samples with the same byte give zero; different bytes
give one. Imported histograms retain the existing checked-total requirement.

## Exactness and numerical behavior

The mathematical quantity is exact for the two empirical laws; the returned
`f64` is rounded. It does not estimate uncertainty, test significance, or identify
an underlying source distribution. No independence model for observations is
assumed.

Internally, TV is `sum_i max(a_i*M - b_i*N, 0) / (N*M)`. Products, differences,
and their sum are exact `u128` integers on supported targets with at most 64-bit
`usize`. The numerator is at most `N*M`, so no accumulator can overflow.
Only the final numerator and denominator are converted to `f64` and divided.
This preserves tiny distances that subtracting rounded probabilities would lose.
Identical laws return positive zero and disjoint supports return exactly one;
results need no clamping. Values close to one may round to one. Tests use a
relative tolerance of `1e-15` against rational references; this is a validation
tolerance, not a promise of correct rounding for every input.

## Cost and related APIs

The byte API takes `O(len(x)+len(y)+256)` time and two fixed byte histograms,
`O(256)` auxiliary storage, with no heap allocation. The reusable
`total_variation_distance_distribution` API takes `O(256)` time and constant
auxiliary storage without recounting or allocation.

The existing `Distribution` and `ByteHistogram` provide exact validated counts.
Arbitrary probability vectors, joint distributions, other distances, and
sequence-order metrics are outside this increment.
