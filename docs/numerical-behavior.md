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

## Collision entropy

The named collision functions delegate to the validated Rényi distribution API
at the fixed order two. Results therefore match that path bit for bit on the
same binary/runtime, with the same empty-state extension and rounding contract.
No second implementation of the entropy formula or new approximation is added.
The fixed order is always valid; it cannot produce `InvalidRenyiOrder`.

Directly squaring `usize` counts can overflow, and summing rounded probability
squares can yield exactly one for a nonconstant but extremely imbalanced law.
Reusing Rényi's maximum scaling and count-complement logarithms preserves tiny
positive entropy in those cases. This baseline does not introduce an optimized
order-two power-sum path.

The independent Python reference computes the matching probability as an exact
`Fraction(sum(c_i*c_i), n*n)` using unbounded integers, then takes its negative
base-2 logarithm with 120-digit Decimal arithmetic. The 45 fixtures include
uneven and uniform laws, counts above `2^53`, totals through `2^64-1`, and
reproducible mixtures of very large and small counts. Cases beyond `usize::MAX`
are skipped explicitly on narrower targets.

A separate property test enumerates all ordered sample-position pairs, including
self-pairs. Another computes the distinct-pair numerator `n*n-sum(c_i*c_i)`
exactly in `u128` for 32/64-bit count tables, then evaluates `-log1p(-q)/ln(2)`
where `q` is the distinct-pair probability. This checks randomized extreme
counts with both absolute and relative tolerances, without sharing the production
Rényi calculation. Run `python3 scripts/reference_collision.py --check`.

## Min-entropy

The dedicated API delegates to Rényi at positive infinity, matching that path
bit for bit within the same binary/runtime. For a nonconstant law with largest
count `m` and total `n`, it evaluates `-ln(m/n)/ln(2)`. When `m > n/2`, it first
computes the integer complement `n-m`, then uses
`-ln_1p(-(n-m)/n)/ln(2)`. Otherwise, direct logarithms of the ratio are safe.
This avoids cancellation from subtracting two large logarithms and prevents
rounding `m/n` to one from erasing small positive entropy. Empty and constant
states are handled explicitly and return positive zero.

The independent Python reference uses an exact rational maximum probability
and 120-digit Decimal logarithms. The 48 retained fixtures include ties, extreme
counts, and identical maximum probabilities with different remaining counts.
Tests use `1e-12` bits absolute tolerance and `1e-8` relative tolerance on positive
reference values, plus a first-order check near `usize::MAX`. These are validation
tolerances, not universal error guarantees. Wider count fixtures are explicitly
skipped on narrower targets. Run `python3 scripts/reference_min_entropy.py --check`.

## Tsallis entropy

For accepted finite nonnegative orders, validate first. Exact order one delegates
to Shannon; empty and singleton support return positive zero; order zero returns
`(support-1)/ln(2)` from integer support. Other orders use `t=q-1` and

```text
S_q = -sum_i p_i * expm1(t * ln(p_i)) / t / ln(2)
```

This follows from `sum(p_i)=1` for the exact empirical law. Evaluating a direct
`1-sum(p_i^q)` would lose precision near one, magnifying rounded mass error by
`1/t`. Here `expm1` retains each small correction; no probability normalization
or substitution of nearby orders by one occurs. Positive counts are visited in
byte order and summed with Kahan compensation. All numerator terms have the
same sign, avoiding cancellation between different symbols.

For `0<q<0.5` and counts at most half the total, use `p-exp(q*ln(p))`
for the numerator term instead. When `q` is tiny, evaluating the near-one
identity as `p*expm1((q-1)*ln(p))` unnecessarily reconstructs a reciprocal of
a small probability; rounding in the logarithm, exponent, and `q-1` can then
accumulate across many rare symbols. Direct powers keep the exponent near zero.
Here `p<=1/2` and `q<1/2` imply `(p^q-p)/p^q >= 1-1/sqrt(2)`, so the
subtraction is safely separated from zero. Dominant symbols still use `expm1`
and the exact count-complement logarithm. The threshold selects an evaluation
formula without changing the requested order. Sixty retained high-precision
regressions cover this review finding and the transition at 0.5.

When a count exceeds half the total, its logarithm uses
`ln_1p(-(total-count)/total)`, subtracting exact integers before conversion.
Elsewhere it uses `ln(count/total)`. This retains a dominant symbol's small
contribution even when its displayed probability rounds to one. Below order one,
the exponential argument is bounded by `ln(total)` for accepted orders and
positive integer counts. Above one, a very negative argument can become negative
infinity in `f64`; `expm1(-infinity)=-1` is its limiting value. Accumulation occurs
before division so small per-symbol contributions are not individually lost to
large-order scaling. At the largest finite order the final entropy of a
nonconstant law is positive and subnormal on the tested target.

The 532 independent fixtures use direct Decimal powers at 120-digit precision,
with a documented bound for the two orders at least `1e100`. Scientific output
preserves tiny values below `1e-300`. General absolute tolerance is `1e-12` in
Shannon-bit scaling, with `1e-8` relative checks on positive references except
exact order one. This exception inherits Shannon's existing contract. These
thresholds are tests, not universal error bounds. Fixtures beyond `usize::MAX`
are skipped explicitly. Run `python3 scripts/reference_tsallis.py --check`; see
[reference notes](references/tsallis.md) for the asymptotic bound.
