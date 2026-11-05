# Mutual information

Mutual information measures how much two aligned byte observations tell us
about one another in their empirical joint law. It detects dependence that
separate frequency tables cannot show. Zero means the empirical joint law
factorizes; it does not prove independence of the source that produced the data.

## Definition, formula and units

For equal-length inputs x,y containing N paired observations:

```text
I(X;Y) = sum_(a,b) p(a,b) * log2(p(a,b)/(p_x(a)*p_y(b)))
p(a,b) = count(a,b)/N
p_x(a) = sum_b p(a,b); p_y(b) = sum_a p(a,b)
```

Here a,b are byte values. The sum visits positive joint probabilities; zero
joint cells contribute zero in this formula. A positive joint count guarantees
positive marginal counts. Units are **bits per pair**, without normalization,
division by two, or multiplication by N. Equivalently, I=H(X)+H(Y)-H(X,Y), with
all three entropies derived from the same observations.

Shannon's 1948 communication paper expresses transmission rate as input entropy
minus remaining uncertainty and equivalently as the sum of input/output
entropies minus their joint entropy (section 12). This shared-information
quantity is now called mutual information. See the
[original paper](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf).

## Worked example

For x=`AABB`, y=`XYXY`, pairs AX, AY, BX, BY each occur once. Each joint
probability is 1/4 and each marginal probability 1/2, so every ratio is one:
I=0 bits/pair. For x=`AABB`, y=`XXYY`, AX and BY each have probability 1/2,
but their marginal products are 1/4. Thus I=2*(1/2)*log2(2)=1 bit/pair.
The same marginal entropies conceal different dependence.

## Domain, boundaries and interpretation

Unequal input lengths return `LengthMismatch` before counting, even if one is
empty. Inputs are paired by position without padding, truncation or alignment
search. Two empty slices return positive zero by library convention; they do
not define a probability law. A singleton or any constant coordinate gives zero.

For a nonempty empirical law, mathematically:

```text
0 <= I(X;Y) <= min(H(X),H(Y)) <= 8
I(X;Y) = I(Y;X); I(X;X) = H(X)
```

Jointly permuting positions, bijectively relabeling either coordinate or scaling
all counts preserves I. Independently reordering one coordinate generally does
not. Identities are subject to floating-point rounding. The API measures the
empirical law, not causation, temporal direction, a significance test or an
unbiased estimate of source dependence. Finite samples can show dependence by
chance, especially with many rare pairs. No smoothing or bias correction is added.

## Computation and numerical behavior

The byte API constructs the existing sparse joint law in `O(N log(s+1))` time
and O(s) heap space for s observed pairs, then evaluates it. Reuse takes
`O(s+256)` time and two fixed 256-count histograms with no new heap allocation.
On supported targets usize has at most 64 bits. Products of exact counts use
u128, so equality and differences of `count*N` and `count_x*count_y` are exact.

To avoid subtracting nearly equal entropies, evaluation uses
`I*ln(2) = sum [p*ln(p/q)-p+q]`, where q=p_x*p_y. Each bracket is nonnegative.
For absent joint cells the bracket is q; their combined mass is calculated by
exact integer subtraction from N squared. Near p=q a convergent power series
for `(1+d)*ln(1+d)-d`, d=(p-q)/q, avoids cancellation. Other cells use the direct
expression. Compensated summation combines the terms. This is an algebraic
evaluation choice, not a different information measure.

Exact empirical independence returns positive zero. Small positive values are
retained without a cutoff or clamp. Byte and reuse paths agree bit-for-bit;
swapping/relabeling can change rounding. Tests use 100-digit references, with
1e-12 absolute tolerance ordinarily and relative checks near independence.
These are validation thresholds, not universal floating-error guarantees.
