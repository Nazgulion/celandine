# Rényi reference notes

## Original definition and history

Alfréd Rényi (1961), *On Measures of Entropy and Information*, Proceedings of the
Fourth Berkeley Symposium on Mathematical Statistics and Probability, volume 1,
547–561 ([Berkeley record](https://digicoll.lib.berkeley.edu/record/112906),
[original paper](https://static.renyi.hu/renyi_cikkek/1961_on_measures_of_entropy_and_information.pdf)).
The paper develops generalized means of information and the entropy family
`H_alpha = log2(sum(p_i^alpha))/(1-alpha)`, recovering Shannon at order one.
This library applies that definition to empirical byte frequencies.

## Special orders and bounds

At zero, each positive probability raised to the order tends to one, giving
logarithmic observed support. At one, differentiating the logarithm of the power
sum yields Shannon's weighted surprise. At infinity, factoring out the largest
probability leaves a bounded correction, giving `-log2(p_max)`. At two the power
sum is the probability that two independent draws from the empirical law agree.

For `alpha > 1`, put `S = sum((p_i/p_max)^alpha)`. Since
`sum((p_i/p_max)^alpha) <= sum(p_i/p_max) = 1/p_max`,
`0 <= H_alpha-H_inf = (H_inf-log2(S))/(alpha-1) <= log2(k)/(alpha-1)`.
This bounds the reference error for very large orders. Uniform laws have entropy
`log2(k)` at every order; increasing order cannot increase Rényi entropy.

## Implementation conventions and variants

Orders are nonnegative, including zero, one, and positive infinity. Negative
orders and NaN are errors. Empty input has no empirical law and returns zero
only by project convention after validation. Zero-count symbols are excluded.
Base two gives bits per symbol; other bases and negative-order extensions are
not part of this API. The dedicated [collision API](../entropy/collision.md) exposes order two.
The dedicated `min_entropy` function exposes the infinite-order limit; see
[its reference notes](min_entropy.md).

## Numerical issues

See [numerical behavior](../numerical-behavior.md#rényi-entropy) for the algebra,
near-one cancellation treatment, maximum scaling, and count-complement handling.
For the scaling method, see Nicholas J. Higham (2021),
[*What Is the Log-Sum-Exp Function?*](https://nhigham.com/2021/01/05/what-is-the-log-sum-exp-function/).

## Test sources

The standard-library Python reference uses the direct definition at 120-digit
precision, independent of the Rust evaluation branches. Its 345 fixtures cover
15 count distributions and 23 orders. The two extremely large finite orders use
the explicit bound above; the infinity reference uses the largest probability.
All 256 nonempty uniform supports are also checked at a range of orders.

Canonical checks use hand-derived power sums. Seeded properties check order
monotonicity, bounds, special-order identities, continuity at adjacent orders
around one, and invariance under permutation, relabeling, and sample repetition.
Imported counts test totals through `usize::MAX` without allocating huge inputs.
No third-party entropy library defines the expected answers.

## Open questions

No Rényi-specific conventions remain open for this byte-count API. Alternate
bases, arbitrary probability vectors, and generic alphabets remain future design
decisions. Cross-platform bitwise reproducibility is not promised.
