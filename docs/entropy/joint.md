# Joint entropy

Joint entropy measures the uncertainty of a pair of byte values observed
together. It captures their combined variability while accounting for repeated
pairings; separate byte entropies cannot determine this quantity.

## Definition, formula and units

For equal-length byte slices x and y of length N, count the aligned pairs
`(x[i],y[i])` and compute

```text
H(X,Y) = -sum_(a,b) p(a,b) * log2(p(a,b))
p(a,b) = count of pair (a,b) / N
```

The sum visits observed pairs a,b; N counts paired observations, not 2N bytes.
The unit is **bits per pair**, without dividing by two or multiplying by N.
This is Shannon entropy of the empirical joint law, evaluated approximately
in floating point. It is not an entropy-rate estimator or a measure of shared
information alone.

Shannon's 1948 *A Mathematical Theory of Communication*, section 6, property 3,
defines joint-event entropy and its relation to marginal entropies as part of
his theory of communication. See the
[original paper](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf).

## Worked example

For x=`AABB`, y=`XYXY`, the pairs AX, AY, BX, BY each occur once:
`H = -4*(1/4)*log2(1/4) = 2` bits/pair. Both marginals have entropy 1 bit.
For the same x paired with `XXYY`, only AX and BY occur, each twice:
`H = -2*(1/2)*log2(1/2) = 1` bit/pair. The marginals remain unchanged,
but knowing one coordinate now determines the other in this sample.

## Domain, bounds and edge cases

Inputs are byte slices with **equal lengths**, interpreted as aligned
observations. Unequal lengths return `LengthMismatch` before counting, including
one empty input. Nothing is truncated, padded or realigned. Two empty inputs
return positive zero by library convention, without defining an empty
unit-mass law. A singleton or constant pair also returns positive zero.

For nonempty samples with s observed pairs:

```text
0 <= H(X,Y) <= log2(s) <= 16
max(H(X),H(Y)) <= H(X,Y) <= H(X)+H(Y)
```

Bounds hold mathematically, subject to rounding. The marginals must come from
the same paired sample. Identical coordinates satisfy H(X,X)=H(X);
independent empirical coordinates satisfy H(X,Y)=H(X)+H(Y). Swapping coordinates
or jointly permuting positions preserves entropy; independently permuting just
one input generally does not. Floating sums need not be bit-identical after
relabeling or swapping. No smoothing, normalization or clamping is added.

## Complexity and numerical behavior

`joint_entropy` constructs the sparse joint table, evaluates it and drops it:
`O(N log(s+1)+s)` time and O(s) heap space, s<=65,536.
`joint_entropy_distribution` reuses a `JointDistribution` in O(s) time, O(1)
extra space and no new heap allocation. Both paths return identical f64 bits
for the same table. The reusable law also exposes exact marginal counts.

Counts are exact integers. Probability conversion/division can round; entropy
uses compensated summation and an integer-complement log1p calculation for a
majority pair. Independent 100-digit fixtures use 1e-12 absolute bit tolerance;
this is a test threshold, not a universal error bound. Finite samples can miss
possible pairs and bias source-entropy estimates. Joint entropy does not
establish causation, independence of a source, or its future behavior.
