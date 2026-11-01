# Normalized Jensen–Shannon statistical complexity

This quantity combines diversity with departure from a uniform distribution.
It is zero for a single certain state and for a distribution uniform over the
whole declared alphabet, and positive between those extremes. Here it measures
empirical **byte frequencies**, not ordering, temporal organization, randomness,
or Kolmogorov complexity. Rearranging a sequence preserves its value.

## Definition, alphabet and units

Call `normalized_jensen_shannon_complexity(data, alphabet_size)`. Declare K, the
number of possible byte states, explicitly in `2..=256`; K must be at least the
number of observed distinct bytes. The alphabet contains every observed label
and K minus support-size unobserved states of probability zero. Labels need not
be numeric values below K: `A` and `B` can form a two-state alphabet. Identities
of unused labels do not affect the result. Use K=256 for the full byte alphabet.
The function does not infer K, smooth probabilities or validate an external list
of allowed labels. Comparisons require the same chosen alphabet convention.

For n observations, p_i = count_i/n is the probability of state i. Let U_K be
the uniform law with u_i = 1/K for each of the K states. With base-two logs:

- `H(P) = -sum_i p_i log2(p_i)`, with zero terms omitted, in bits per symbol.
- `J(P,U_K) = H((P+U_K)/2) - (H(P)+log2(K))/2`, in bits; the mixture gives
  each law equal weight, independently of n.
- `J_max(K) = 1 + log2(K)/2 - ((K+1)/(2K))*log2(K+1)`, in bits. This is the
  maximum of J against U_K, attained by a point mass.
- `C(P;K) = (H(P)/log2(K)) * (J(P,U_K)/J_max(K))`, dimensionless.

Each factor lies mathematically in `[0,1]`, hence so does C. This normalization
scales the two factors separately; it does **not** rescale the largest attainable
product to one. The two factors reach their maxima at different laws. Logarithm
base cancels when used consistently. This is exact for the empirical law in
mathematical terms, computed with floating-point rounding; source inference is
an estimation problem. It is not a distance between sequences.

## Worked examples

For `AAAB` and K=2, P=(3/4,1/4), U=(1/2,1/2), and their mixture is (5/8,3/8).
H(P)=0.8112781244591328 bits, J=0.0487949406953985 bits and
J_max(2)=0.3112781244591328 bits. Their normalized product is about 0.127173305349489.

```
use celandine::complexity::normalized_jensen_shannon_complexity as complexity;
assert_eq!(complexity(b"AAAA", 2), Ok(0.0));
assert_eq!(complexity(b"AABB", 2), Ok(0.0));
assert!((complexity(b"AAAB", 2).unwrap() - 0.127173305349489).abs() < 1e-6);
assert!(complexity(b"AABB", 4).unwrap() > 0.0);
```

`AABB` is uniform for K=2 but occupies only half of K=4. For K=4 its normalized
entropy is 1/2, J is 0.3112781244591328 bits, and J_max is about 0.548795,
giving C about 0.283601488804453. Choosing the alphabet changes the question. Constant
samples remain zero for all valid K because their entropy vanishes. Uniform
frequency alone says nothing about temporal randomness: `AABB` and `ABAB` agree.

## Errors and numerical behavior

# Errors

`StatisticalComplexityError` checks, in order: K outside `2..=256`
(`InvalidAlphabetSize`), no observations (`EmptyDistribution`), and support
larger than K (`AlphabetTooSmall`). K=1 is rejected because both normalizers
vanish; there is no silent singleton convention. Empty input has no law, even
though the standalone Shannon API uses a zero convention. No ordinary invalid
input panics. The distribution API accepts validated integer counts, including
large counts whose total fits `usize`.

J reuses the count-based divergence implementation: exact integer differences
preserve deviations even when rounded probabilities appear identical. No result
is clamped. Entropy uses compensated summation and the complementary count
with `ln_1p` for a majority state, retaining its contribution near probability
one. It can differ in low-order bits from standalone `shannon_distribution`.
Tolerances in the independent fixtures are validation criteria, not
a universal floating-point error bound.

## Cost, history and limits

The byte API counts once in O(n+256) time. The distribution API takes O(256)
time. Both use fixed storage and no heap allocation. The uniform reference is
represented by K unit counts, with zero counts outside that alphabet; it is not
an empirical sample combined with the input. The computation does not depend on
input order and does not construct ordinal patterns or block distributions.

López-Ruiz, Mancini and Calbet (1995) proposed combining entropy with departure
from equilibrium. Lamberti, Martín, Plastino and Rosso (2004) replaced the
Euclidean disequilibrium with Jensen–Shannon divergence. The present convention
uses the normalized Shannon and normalized Jensen–Shannon product, with an
explicit finite state space. See [reference notes](../references/statistical_complexity.md)
for primary citations, the normalization derivation, and independent checks.
