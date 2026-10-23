# Jensen–Shannon divergence

Jensen–Shannon divergence compares two probability laws through their equally
weighted mixture. It measures the extra average ideal code length from using
that mixture rather than knowing which law produced a draw. Here it compares
observed byte composition; it does not model how the input sequences arose.

## Definition, units, and interpretation

For byte probabilities `p_i` and `q_i`, set `m_i = (p_i+q_i)/2`. Then
`J(P,Q) = (1/2) sum_i [p_i log2(p_i/m_i) + q_i log2(q_i/m_i)]`.
The index `i` ranges over all 256 byte labels. A term with zero numerator
contributes zero; if both probabilities are zero, the whole symbol contributes
zero. The mixture is positive wherever either law is positive.

Equivalently, `J(P,Q) = H(M) - (H(P)+H(Q))/2`, with `H` the base-2 Shannon
entropy and `M=(P+Q)/2`. The result is in **bits**, mathematically in `[0,1]`.
Zero means identical laws, and one means disjoint supports. It is symmetric but
is not itself a metric: the square root is the separately exposed
`jensen_shannon_distance`.

Each law has weight one half regardless of sample length. This is not entropy
of pooled observations, whose weights would depend on lengths. There is no
weight or logarithm-base parameter in this increment.

## Historical context

Jianhua Lin introduced the named Jensen–Shannon divergence in 1991 as part of
an entropy-based family allowing weighted comparisons. The present API selects
the two-law, equal-weight, base-2 convention from that family.
[Lin's paper](https://www.cise.ufl.edu/~anand/sp06/jensen-shannon.pdf).

## Worked example

`A` gives `P=(1,0)` and `AB` gives `Q=(1/2,1/2)`. Their equal mixture is
`M=(3/4,1/4)`, with `H(M)=0.8112781244591328` bits. Since `H(P)=0` and
`H(Q)=1`, `J=0.8112781244591328-(0+1)/2=0.31127812445913283` bits.
Repeating either sample does not change its law or its mixture weight.

```rust
use celandine::divergence::{jensen_shannon_divergence, EmptyDistribution};
let j = jensen_shannon_divergence(b"A", b"AB").unwrap();
assert!((j - 0.31127812445913283).abs() < 1e-14);
assert_eq!(jensen_shannon_divergence(b"AABB", b"ABAB"), Ok(0.0));
assert_eq!(jensen_shannon_divergence(b"AAA", b"B"), Ok(1.0));
assert_eq!(jensen_shannon_divergence(b"", b"AB"), Err(EmptyDistribution));
```

## Domain and edge cases

Both samples must be nonempty, with arbitrary possibly unequal lengths. Either
empty input, including both, returns `EmptyDistribution` before counting: an
empty state is not a probability law. The reusable distribution API applies
the same rule. Counts come from validated `ByteHistogram` instances; overflow
is rejected during import. Byte labels share the same alphabet in both inputs.

No sequence alignment, smoothing, probability-vector normalization, text
decoding, or input-order dependence is introduced. Zero counts are valid and
cause no infinities. Identical or proportional count tables return positive
zero; disjoint supports return exactly one. Constant samples can give zero or
one depending on whether their byte labels agree.

## Numerical behavior and cost

The quantity is exact for the empirical laws in mathematical terms; `f64`
introduces rounding. It is not an unbiased source-divergence estimator, a
significance test, or a measure of sequential dependence.

The implementation sums nonnegative per-symbol contributions without subtracting
rounded entropy values. For counts `a_i,b_i`, sample sizes `N,M`, and products
`A=a_i*M`, `B=b_i*N`, form `d=|A-B|/(A+B)`. The difference is exact `u128` on
32/64-bit count targets. The product sum is computed after floating conversion
because an integer `A+B` could overflow. A symbol contributes
`(p_i+q_i)/2 * g(d)`, where
`g(d)=((1+d)ln(1+d)+(1-d)ln(1-d))/(2 ln(2))`.
For `d<=1/2`, a positive power series avoids cancellation; elsewhere `ln_1p`
and explicit endpoint handling are used. Compensated summation combines symbols.

Bounds can be exceeded slightly by rounding; outputs are not clamped. A value
near one may round to one. Tests use `1e-12` relative tolerance for nonzero
high-precision reference values, including tiny differences, and exact checks
for zero/one. This is evidence on the supported platform, not a universal error
bound or cross-platform bitwise guarantee. Sample repetition and shared
relabeling preserve the mathematical answer, subject to rounding.

The byte API takes `O(len(x)+len(y)+256)` time and two fixed histograms with
`O(256)` auxiliary storage. Reusing distributions takes `O(256)` time and
constant auxiliary storage. All paths allocate no heap memory. The series uses
at most 32 terms per symbol. Public KL divergence, arbitrary weights, generic
alphabets, and joint distributions remain outside this increment.
