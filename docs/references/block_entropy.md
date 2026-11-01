# Block entropy: definition and verification

C. E. Shannon (1948), *A Mathematical Theory of Communication*, Bell System
Technical Journal 27, 379–423 and 623–656,
[original paper](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf).
Section 7, Theorem 5 defines G_N using probabilities of N-symbol sequences,
scaled by 1/N. Our H_n is the unscaled Shannon entropy, with probabilities
replaced by overlapping finite-sample relative frequencies. The zero-observation
policy is a library convention, not Shannon's source theorem.

See [the metric page](../entropy/block.md) for formula, units, edge cases,
complexity and numerical choices. Overlapping observations, non-overlapping
partitions and periodic wrapping are distinct sampling conventions; this API
uses overlapping complete blocks only. Empty-word subword complexity p(0)=1
does not change the positive block-length domain here.

`python3 scripts/reference_block_entropy.py --check` regenerates 1,254 cases.
The independent implementation sorts materialized blocks and groups equal runs,
then evaluates `log2(m) - sum(c*log2(c))/m` in 100-digit Decimal arithmetic.
It uses neither the Rust n-gram tree nor the library's compensated probability
sum. Cases include all binary strings through length six, zero/oversized/maximum
orders, unequal and equal frequencies, all 256 bytes, rare transitions and
seeded larger samples. Both APIs are checked to `1e-12` absolute bits.

Hand calculations verify ABABA at n=2 (one bit) and AABBA at n=2 (two bits),
which share a byte histogram. Seeded properties check support bounds, reversal,
bijective relabeling and n=1 agreement with Shannon. Repetition or arbitrary
byte permutation is deliberately not asserted to preserve block entropy:
new joins and local arrangements change the count law. A rare-transition test
and support above 256 complement the small fixtures. No new production defect
was found during the initial review; these are correctness tests, not claimed
regressions from an observed failure.
