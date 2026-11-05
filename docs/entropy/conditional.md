# Conditional entropy of aligned byte pairs

Conditional entropy measures how much uncertainty remains about X after Y is
observed. It accounts for the frequencies of the different Y values: a rare
ambiguous group contributes less than an equally ambiguous common group.
This reveals empirical predictability between aligned observations.

## History

Claude Shannon defined conditional entropy as a probability-weighted average
in his 1948 *A Mathematical Theory of Communication*, section 6, property 5.
Its chain rule relates joint and marginal uncertainty; section 12 uses conditional
entropy to describe information missing after transmission (equivocation).
See the [original paper](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf).

## Formula, direction and units

`conditional_entropy(x, y)` computes **H(X|Y)**: the first input is the target,
the second the conditioning variable. For equal-length byte slices with N
aligned pairs, let c(a,b) count `(x[i]=a, y[i]=b)` and let
`c_y(b) = sum_a c(a,b)` be the second-coordinate marginal count. Then

```text
H(X|Y) = -sum_(a,b) [c(a,b)/N] * log2[c(a,b)/c_y(b)]
       = H(X,Y) - H(Y)
       = H(X) - I(X;Y)
```

The sum visits observed pairs, so both counts inside each logarithm are positive.
`c(a,b)/N` weights each pair by its frequency; `c(a,b)/c_y(b)` is the probability
of a within its observed b group. Units are **bits per X observation** (one
first-coordinate byte per aligned pair), not total bits or bits per two input
bytes. Base two selects bits. A `JointDistribution` retains this coordinate order
for `conditional_entropy_distribution`.

The implementation sums conditional contributions directly; it does not subtract
entropies or construct a new public conditional distribution. This is distinct
from `conditional_entropy_order(data,k)`, which conditions the next byte in one
sequence on its preceding k bytes.

## Worked examples

For x=`ABCD`, y=`XXYY`, the X group contains A/B with probabilities 1/2 each;
the Y group contains C/D with probabilities 1/2 each. Each group occurs half
the time: `H(X|Y) = (1/2)*1 + (1/2)*1 = 1` bit per X observation.
Reversing the inputs gives zero because each of A, B, C, D determines its partner.

For x=`AAAABC`, y=`XXXXYY`, the first group is certain and the second has two
equally likely outcomes. Weights are 4/6 and 2/6, so the result is
`(4/6)*0 + (2/6)*1 = 1/3` bit, not the unweighted average 1/2.
Independent empirical coordinates `AABB`/`XYXY` give 1 bit; identical coordinates
give zero, regardless of their marginal entropy.

## Domain, bounds and boundaries

Only equal-length byte slices are accepted. `LengthMismatch` is returned before
counting for unequal lengths, including one empty slice. There is no truncation,
padding, realignment or smoothing. Two empty inputs return positive zero by
convention; this does not define an empty probability law. Singletons, a constant
first input, and all deterministic first-given-second laws return positive zero.
A constant second input gives H(X), up to numerical rounding.

Mathematically `0 <= H(X|Y) <= H(X) <= 8` bits per first-coordinate byte.
The chain rules above use marginals from exactly the same paired observations.
In general H(X|Y) differs from H(Y|X). Jointly permuting positions, repeating the
whole paired sample, or bijectively renaming either coordinate preserves the
quantity; independently permuting only one input need not preserve it.

## Numerical behavior

Counts and marginal sums are exact integers. Floating-point conversion, division
and logarithms can round. Each nonnegative contribution is evaluated directly,
with compensated summation. For a conditional probability above 1/2, the
logarithm uses `log1p(-(c_y-c)/c_y)` with the integer difference formed before
conversion. This retains a nearly certain outcome's small contribution even
when c/c_y would round to one. Deterministic groups contribute exactly zero.
No rounding-to-zero threshold or range clamp is applied.

Independent 100-digit references compute H(X,Y)-H(Y), using 1e-12 absolute bit
tolerance and additional 1e-12 relative checks on small positive fixtures. These
are validation thresholds, not universal error guarantees. Bitwise equality is
checked between byte and reuse APIs on one runtime; relabeling can change the
floating summation order and hence low-order bits.

## Costs and interpretation limits

For N pairs and s observed pairs (s<=65,536), the byte API takes
O(N log(s+1)+s+256) time and O(s) heap storage for the sparse joint table.
Empty inputs and length errors allocate nothing. Reuse takes O(s+256) time,
fixed 256-count scratch storage and no new heap allocation.

This is an empirical Shannon quantity evaluated with f64. As an estimate of
unknown source conditional entropy, it can be biased by missing combinations
and small samples. A deterministic sample does not prove a deterministic source.
The value is not a causal effect, entropy rate, or conditional mutual information.
