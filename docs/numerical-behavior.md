# Numerical behavior

Counts are exact `usize` integers. Probability calculation converts each count
and total to `f64`, then divides. Integers through `2^53` are exactly representable
in `f64`; larger integers can round. Checked count-array construction allows
testing large totals without allocating corresponding sequences. On narrower
targets the count range is limited by `usize`.

Shannon sums `-p * p.log2()` for positive probabilities, in ascending byte order,
starting at positive zero. At most 256 terms are added with ordinary sequential
summation. There is no subtraction of large nearly equal entropy expressions.
The baseline intentionally uses the direct formula; compensated summation or
other changes need evidence from accuracy/performance measurements.

At extreme imbalance, a majority probability can round to one. The very small
minority probabilities remain positive, but relative accuracy of tiny entropies
is limited. Absolute `1e-12`-bit agreement is the test criterion; this is not a
relative-error guarantee. Counts remain available for future higher-precision
work. Probabilities derived from representable `usize` counts do not underflow
to zero for positive counts on supported 32/64-bit targets.

For nonempty data the mathematical bound is `0 <= H <= log2(support) <= 8`.
Floating-point summation can slightly exceed the theoretical upper bound;
results are not clamped. Empty and constant inputs produce exact positive zero.

Rust's `f64::log2` can vary in precision across platforms and versions. Fixed
iteration order avoids randomized accumulation, but reproducibility across
different toolchains is numerical (within tested tolerances), not bitwise.

Independent verification uses Python's standard-library `Counter`, exact
`Fraction` probabilities, and 80-digit `Decimal` arithmetic with natural
logarithms and a change of base. Committed fixtures keep `cargo test` independent
of Python; `python3 scripts/reference_shannon.py --check` checks regeneration.

## Hartley entropy

Hartley scans exact counts for presence and evaluates `log2(k)` once, where
`1 <= k <= 256`. This integer-to-`f64` conversion is exact. Counts above `2^53`
do not affect its numerical accuracy because counts are never converted to
probabilities. Empty and singleton support return positive zero explicitly,
without evaluating `log2(0)`. No tolerance determines presence or absence.

The `1e-12`-bit test tolerance and platform `log2` caveat also apply to Hartley.
Its independent Python reference uses sets and 80-digit `Decimal` logarithms
for every support size from 0 through 256. Rust tests check those values using
different symbol labels and frequencies and both API entry points. Run
`python3 scripts/reference_hartley.py --check` to verify the fixtures.
