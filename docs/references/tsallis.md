# Tsallis entropy reference notes

## Original definition and history

C. Tsallis (1988), *Possible Generalization of Boltzmann–Gibbs Statistics*,
Journal of Statistical Physics 52, 479–487.
[DOI](https://doi.org/10.1007/BF01016429),
[original paper scan](https://physicsgg.me/wp-content/uploads/2023/08/possible_generalization_of_boltzmann-gibbs_statist.pdf).
The paper proposes an entropy family using probability powers, with a positive
scaling constant, to generalize Boltzmann–Gibbs statistics. Equations (1), (1'),
(2), and (3) give the definition, order-one limit, per-symbol identity, and
uniform-law value. Our implementation uses these as mathematical distribution
functionals without a physical model.

## Scale and domain selected here

The library fixes the constant to `1/ln(2)`, so the limit at one is
`-sum(p_i*ln(p_i))/ln(2)`, equal to Shannon in bits per symbol. The unscaled
formula uses constant one and has a natural-log limit. A different choice,
`(1-sum(p_i^q))/(1-2^(1-q))`, makes the uniform binary law equal to one at every
order; it is **not** this API. No automatic probability normalization occurs.

We accept finite nonnegative orders. Zero uses observed positive support and
returns `(k-1)/ln(2)`, with empty input handled separately by convention. Negative
orders and nonfinite parameters are rejected before evaluating any input.

## Useful identities

At order two, `S_2 = (1-sum(p_i^2))/ln(2)` is the probability of differing
independent draws divided by `ln(2)`. At order one, differentiating the power
sum gives Shannon. For Rényi entropy, substituting
`sum(p_i^q)=2^((1-q)*H_q)` gives the relation on the
[metric page](../entropy/tsallis.md).

For independent product laws, power sums multiply. Substituting
`sum(p_i^q)=1+(1-q)*ln(2)*S_q(P)` for each factor yields
`S_q(P*Q)=S_q(P)+S_q(Q)+(1-q)*ln(2)*S_q(P)*S_q(Q)`.
This explains the scale-dependent coefficient in the composition property test.
The test builds product count tables only; no public joint-distribution API is
introduced in this milestone.

## Numerical verification

The independent Python script evaluates the direct power sum with 120-digit
Decimal arithmetic and exact binary64 parameters. It retains scientific-format
values so tiny positive reference results remain representable as subnormal
`f64`. There are 532 fixtures: 15 named distributions at 24 orders, plus 16
reproducibly generated extreme count tables at seven selected orders, and 60
review regression cases across four full-support extreme tables and 15 orders.
The regression cases cover tiny positive orders and both sides of the numerical
formula transition at 0.5.

For `q>=1e100`, the reference uses `1/(q-1)/ln(2)` for nonconstant laws. With
`n<=2^64-1`, every probability is at most `1-1/n`; hence the omitted sum is
bounded by `256*exp(-q/(2^64-1))`. Its effect on relative error is at most that
bound divided by one minus the bound, far below `1e-120` at these orders.
These are explicitly bounded asymptotic references, not direct-power evaluations.
Empty and constant cases are exactly zero at every accepted order.

Canonical and property tests also check uniform supports, independent product
composition, order monotonicity, the Rényi relation, and permutation/relabeling/
repetition invariance. Order-two extreme-count tests compute differing-pair
numerators exactly in `u128` on 32/64-bit targets. Fixtures requiring counts
wider than `usize` are explicitly skipped. Absolute tolerance is `1e-12`, with
extra relative checks for positive values except exact order one, which retains
Shannon's documented limitations. See [numerical behavior](../numerical-behavior.md#tsallis-entropy).

## Open questions

No Tsallis-specific convention remains undecided for this byte-count API.
Other scaling choices, negative or infinite orders, arbitrary probability
vectors, and source estimators are outside this milestone.
