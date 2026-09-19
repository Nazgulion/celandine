# Collision entropy reference notes

## Definition and history

A. Rényi (1961), *On Measures of Entropy and Information*, Proceedings of the
Fourth Berkeley Symposium on Mathematical Statistics and Probability, volume 1,
547–561 ([original paper record](https://digicoll.lib.berkeley.edu/record/112906)).
Collision entropy is the order-two specialization of this family:
`H2 = -log2(sum_i p_i^2)`. We do not attribute a separate invention date to the
name "collision entropy".

## Matching-pair interpretation

For independent draws `X` and `Y` with the same law, the events `X=Y=i` are
disjoint and have probabilities `p_i*p_i`. Thus `P(X=Y)=sum_i p_i^2`.
With empirical counts, this is `sum_i c_i^2/n^2`, the fraction of all ordered
sample-position pairs that have equal symbols, including self-pairs. The core
measure describes this empirical law and assumes no model for the input source.

For `AAAB`, ten of sixteen pairs match, so `H2=log2(8/5)`. For `AB`, two of four
pairs match and `H2=1`. These calculations use replacement. Excluding self-pairs
changes the probability to `sum_i c_i*(c_i-1)/(n*(n-1))`, a different quantity.
Neither adjacent-pair counting nor that without-replacement estimator is used.

## Conventions and numerical behavior

The dedicated API returns bits per symbol and reuses the stable Rényi order-two
calculation. Empty input has no matching probability but returns zero entropy by
project convention. Constant input has matching probability one and entropy zero.
See the [metric page](../entropy/collision.md), [Rényi reference](renyi.md), and
[numerical notes](../numerical-behavior.md#collision-entropy) for details.

## Verification

The reference script uses exact rational matching probabilities and 120-digit
Decimal logarithms, not a third-party entropy library. Its 45 fixtures include
32 generated mixtures of count magnitudes as well as named canonical and extreme
cases. Property tests independently enumerate pairs and use an exact integer
complement formula for large counts. Tests cover every uniform byte support,
positive zero, invariance under permutation/relabeling/repetition, equivalence
to order two, and the Shannon/Hartley upper bounds.

## Open questions

No collision-specific convention remains undecided for the count-backed byte
API. A public matching-probability function, source bias corrections, arbitrary
probability vectors, and generic alphabets are outside this milestone.
