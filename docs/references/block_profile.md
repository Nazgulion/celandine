# Block-entropy profiles: references and verification

C. E. Shannon (1948), *A Mathematical Theory of Communication*, Bell System
Technical Journal 27, 379–423 and 623–656,
[original paper](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf).
Section 7, Theorem 5 relates normalized entropies of source blocks to source
entropy. The API exposes unnormalized empirical H_n, without asserting that
the source theorem's assumptions hold for a finite input. Its capped range and
lazy iterator are library choices, not additional entropy definitions.

See [the profile definition](../entropy/block_profile.md) and
[scalar block entropy](../entropy/block.md). A full finite profile ends at H_L=0;
its changing boundary samples need not yield a monotonic or concave sequence.
The library's conditional-entropy API uses matched marginals instead of taking
adjacent differences of independently sampled profile entries.

`python3 scripts/reference_block_profile.py --check` verifies 1,831 fixtures.
Python counts materialized overlapping slices independently with Counter and
computes `log2(m) - sum(c*log2(c))/m` using 100-digit Decimal arithmetic.
Profiles cover all binary strings through length seven, worked examples, all
256 byte values, and seeded mixed samples. Limits include zero, short prefixes,
full lengths, oversized values and a portable maximum marker. Both the number
of entries and their values are checked, with absolute tolerance 1e-12 bits.

Seeded stateful tests compare mixed next/next_back/nth/nth_back/clone/count/last
operations against a deque, including maximum skips. Separate checks enforce
bitwise scalar agreement. The isolated allocation executable checks that count,
last and skips do not evaluate discarded orders, from empty inputs through
10 MiB. This preserves the lesson from the existing subword-profile regression:
a generic Map adapter can evaluate skipped entries. The new profile directly
advances its range before calling the scalar metric.

No new production defect was found during initial review. The benchmark's
construction-only case initially needed an explicit discard of its lazy iterator
to satisfy the must-use lint; that development-harness issue was corrected before
measurement. Fuzzing and archived performance evidence supplement the normal
checks without proving exhaustive correctness.
