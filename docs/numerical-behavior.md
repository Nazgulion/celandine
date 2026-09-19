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

## Rényi entropy

Exact orders zero and one delegate to Hartley and Shannon, including their
rounding behavior. In particular, Shannon's existing relative-accuracy limitation
at extreme imbalance also applies at exact order one. Empty and singleton support
return positive zero. No tolerance changes the selected order or clamps output.

For `t = alpha - 1` with `0 < |t| <= 0.25`, use the identity

```text
u = sum_i p_i * expm1(t * ln(p_i))
H_alpha = -log1p(u) / t / ln(2)
```

It follows from `sum(p_i) = 1` for the exact empirical law. `expm1(x)` evaluates
`exp(x)-1` accurately near zero; `log1p(x)` evaluates `ln(1+x)` similarly. This
avoids cancellation in the direct formula near order one and avoids amplifying
floating-point mass error by `1/t`. It does not renormalize supplied probabilities:
input consists of exact integer counts, and no probability-vector API exists.
The threshold chooses an evaluation formula, not a different mathematical order.

Outside this interval, let `m` be the largest count and `p_max = m/n`. Factor out
`p_max^alpha` and calculate `L = ln(sum_i (c_i/m)^alpha)` by omitting one maximum
term (exactly one), summing the others, and using `log1p`. At orders above one,
evaluate `H_inf + (H_inf - L/ln(2))/(alpha-1)`, with
`H_inf = -ln(p_max)/ln(2)`. This avoids multiplying the largest finite order by
`ln(p_max)`. Below one use `(alpha*ln(p_max)+L)/(1-alpha)/ln(2)`, where the
denominator stays away from zero. Positive infinity returns `H_inf` directly.
Tiny scaled powers may underflow to zero; their contribution is negligible at
the tested absolute tolerance. All sums use Kahan compensation in byte order.

For an integer ratio `a/b` near one, compute `ln(a/b)` as
`log1p(-(b-a)/b)`, subtracting integers before conversion to `f64`. This preserves
rare-symbol contributions to the logarithm even when a majority probability
rounds to one. Elsewhere use the natural logarithm of the converted ratio.
Arithmetic and transcendental functions still round; this is not arbitrary
precision. Fixed order guarantees repeatability within a binary/runtime, not
bitwise agreement between platforms.

The [reference calculation](../scripts/reference_renyi.py) evaluates direct
powers with 120-digit Decimal arithmetic and exact binary64 order values, then
retains 60 decimal places. It tests adjacent representable orders around one,
both formula transitions, uneven distributions, tied maxima, and imported counts
through `2^64-1`. For finite orders at least `1e100`, it uses the independent bound
`0 <= H_alpha-H_inf <= log2(k)/(alpha-1) < 9e-100` bits to retain the same 60-place
reference; those cases are explicitly limiting-value checks. Fixtures needing
64-bit counts are skipped on narrower targets. General checks use `1e-12` bits
absolute tolerance, with additional relative checks on tiny reference values
except at exact order one. Run `python3 scripts/reference_renyi.py --check`.

Maximum scaling and omission of one largest term follow the numerical approach
explained by [Higham (2021)](https://nhigham.com/2021/01/05/what-is-the-log-sum-exp-function/).
The near-one identity and count-complement formulas above follow directly by
algebra from the empirical definition.
