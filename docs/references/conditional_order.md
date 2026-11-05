# Conditional entropy by order: references and verification

C. E. Shannon (1948), *A Mathematical Theory of Communication*, Bell System
Technical Journal 27, 379–423 and 623–656,
[original paper](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf).
Section 6 describes conditional entropy; section 7, Theorem 6 conditions a
following symbol on a block of preceding symbols. The finite-sample API uses
empirical joint counts and their prefix marginal on exactly the same positions.
Its empty-observation convention is a library choice, not a source theorem.

See [the metric](../entropy/conditional_order.md) for the formula, units, edge
cases and limits. The direct weighted sum avoids subtraction of large entropies.
The n-gram reuse API interprets each final byte as a target and its preceding
prefix as the context; its order is the stored positive block length minus one.

`python3 scripts/reference_conditional_order.py --check` verifies 2,419 fixtures.
Python independently counts joint observations and contexts by target position,
then calculates `H(context,next)-H(context)` using 100-digit Decimal logarithms.
This differs from Rust's sorted prefix groups and direct conditional sum.
All binary strings through length seven, canonical boundaries, all byte values,
rare transitions and seeded samples are included, with zero, full, oversized
and maximum orders. Both Rust APIs use `1e-12` absolute bit checks against the
references (order MAX has no representable `order+1` table).

Hand examples establish ABABAB order one = 0, AAB order one = 1, and AABA
order one = 2/3 bits per predicted byte. A context with all 256 successors tests
the full byte alphabet; an analytic rare-transition case checks relative error.
Seeded properties verify the matched chain rule independently, bounds by target
entropy, bijective renaming and byte/reuse agreement. Arbitrary shuffling,
reversal and monotonicity across changing target sets are not valid invariants.

The initial test run exposed an incorrect fixture-count assertion (the generator
produced 2,419, not the estimated >2,500). All numerical comparisons passed;
the assertion now checks the exact count. No production defect was found during
this initial review. Allocation tests, sanitizer fuzzing and performance evidence
supplement these mathematical checks.
