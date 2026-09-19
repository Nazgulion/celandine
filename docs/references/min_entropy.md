# Min-entropy reference notes

## Definition and history

A. Rényi (1961), *On Measures of Entropy and Information*, Proceedings of the
Fourth Berkeley Symposium on Mathematical Statistics and Probability, volume 1,
547–561 ([original paper record](https://digicoll.lib.berkeley.edu/record/112906)).
Min-entropy is the infinite-order limit of this family. This identifies its
mathematical origin without attributing a separate invention date to the name.

## Derivation and interpretation

Let `p_max` be the largest probability of a nonempty finite law. Factoring
`p_max^alpha` out of the power sum gives
`H_alpha = alpha/(1-alpha)*log2(p_max) + log2(S)/(1-alpha)`, where
`S = sum_i (p_i/p_max)^alpha`. For positive orders, `1 <= S <= k`, with `k`
the support size. As the order tends to infinity, the second term vanishes and
the first tends to `-log2(p_max)`. Ties do not change this limit.

A fixed guess of symbol `i` succeeds with probability `p_i` under this law.
Maximizing that probability gives `p_max = 2^(-H_inf)`. This is an empirical
single-symbol statement, not inference about an unknown source or conditional
predictions. In `AAAB`, `p_max=3/4` and `H_inf=log2(4/3)`.

## Conventions and numerical behavior

The API returns bits per symbol, reusing the stable Rényi infinity calculation.
Empty input has no empirical maximum probability and returns zero by project
convention. Constant input has maximum probability one and entropy zero.
See the [metric page](../entropy/min_entropy.md) and
[numerical notes](../numerical-behavior.md#min-entropy).

## Verification

The independent reference evaluates the exact rational `max(counts)/sum(counts)`
with 120-digit Decimal logarithms. Its 48 fixtures include canonical samples,
tied maxima, laws with the same maximum and different remaining probabilities,
and generated extreme count mixtures. Property tests independently tally symbols
and minimize individual surprises, recover the best fixed guess probability,
and check invariance and bounds through totals of `usize::MAX`. Tiny positive
results receive additional relative-error checks.

## Open questions

No min-entropy-specific convention remains undecided for this count-backed API.
Arbitrary probability vectors, conditional measures, source confidence bounds,
and generic alphabets remain outside this milestone.
