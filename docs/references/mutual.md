# Mutual information: references and verification

C. E. Shannon (1948), *A Mathematical Theory of Communication*, Bell System
Technical Journal 27, 379–423 and 623–656,
[original paper](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf).
Section 12 expresses transmission rate as H(X)+H(Y)-H(X,Y) and as entropy
minus equivocation. The library applies the shared-information quantity to a
finite empirical joint law in bits per pair; it does not infer a channel rate
or source law from a single sample. See [the metric](../information/mutual.md).

## Stable evaluation

Write P for the empirical joint law and Q=P_X*P_Y. Since both sum to one,
adding `-P+Q` cellwise to `P*ln(P/Q)` preserves their divergence. Each resulting
term is nonnegative. For counts c, marginal counts r,s, and total N, evaluate
`[a*ln(a/b)-a+b]/N^2` with a=c*N, b=r*s. Integer u128 arithmetic forms these
products and their differences exactly on supported targets. For absent cells,
a=0 and the term is b/N^2; the sum of their b values is `N^2-sum_observed b`.

When d=(a-b)/b is small, use
`(1+d)*ln(1+d)-d = d^2/2 - d^3/6 + d^4/12 - ...`.
The implementation uses this series for |d|<=1/4, stopping when a term is at
most machine epsilon times the partial sum, never going beyond order 32.
For this interval the remaining geometric tail is bounded and negligible at
f64 precision. Elsewhere the direct expression is separated from its cancellation
point. Compensated summation combines cell terms and absent-cell mass.
This derivation is an implementation choice; no public generalized-KL API is added.

## Independent verification and review

`python3 scripts/reference_mutual.py --check` verifies 1,389 fixtures via
100-digit Decimal H(X)+H(Y)-H(X,Y), independently of this decomposition.
They include every binary pair sequence through length five, seeded samples,
near-independent symmetric 2x2 laws, points around the series boundary, a
nonuniform independent law, rare diagonal events and missing joint cells.
Tolerance is 1e-12 absolute bits; tiny positive fixtures additionally require
relative error at most 1e-12. Exact independence is decided by integer counts
in the oracle and must return positive zero in the API.

Seeded properties verify symmetry, diagonal identity, marginal bounds, joint
permutation, relabeling, scaling and the entropy identity. Full-alphabet tests
cover 8-bit diagonal information and zero for all 65,536 pairs. A private-kernel
test uses products near 2^120 separated by one: the leading quadratic term has
a mathematically negligible relative remainder, while subtracting f64 products
would erase the difference. Allocation checks cover empty/errors and reuse.

Review found and corrected a measurement-workflow defect: adding mutual input
controls accidentally changed the older joint workload list. The new regression
failed before the fix and now protects that list. No new production metric
defect was found. Fuzzing and measured evidence supplement these checks, without
proving correctness for all inputs or eliminating finite-sample bias.
