# Shannon reference notes

## Original definition

Shannon (1948), Part I, section 6, defines uncertainty of a discrete probability
law through the weighted logarithmic sum. With base 2 and unit scale this is
`H = -sum(p_i log2(p_i))`.

## Modern references

Cover and Thomas, second edition, section 2.1; MacKay, chapter 2. Full
bibliographic entries and the original paper link are in the
[bibliography](../references.md).

## Implementation convention

For a nonempty sample, probabilities are relative byte frequencies. Entropy is
in bits per symbol by default. The explicit-base APIs compute `H_b = H_2/log2(b)`
for finite `b > 1`. Zero terms are omitted, as justified by continuity. Empty
input returns zero solely by documented project convention, after base validation
when applicable.

## Known variants

Different bases change units; the explicit-base API implements this convention
from Shannon's introduction (pages 1–2 of the linked reprint). Dividing by an alphabet-dependent maximum gives
a normalized score and requires an explicit choice of alphabet. Bias-corrected
source estimators and entropy rates are distinct from the empirical marginal
entropy implemented here. Normalized scores, bias correction, and entropy rates
are not implemented yet.

## Numerical issues

`f64` count conversion, division, logarithm accuracy, and finite summation cause
rounding; see [numerical behavior](../numerical-behavior.md).

## Test sources

- Direct algebra: a point mass gives zero; `k` equal probabilities give `log2(k)`.
- `AAAB`: `-(3/4) log2(3/4) - (1/4) log2(1/4)`.
- Independent Python `Counter`/`Fraction`/`Decimal` evaluation at 80-digit
  precision generates `tests/data/shannon.tsv`.
- Property tests compare a `BTreeMap` count implementation using natural logs,
  and check bounds, count preservation, repetition, and symbol relabeling.

## Open questions

The probability-vector constructor and its explicit tolerance policy, and a
cross-platform precision policy, need separate design review.
They are not prerequisites for this empirical byte API.

## Explicit-base verification

`scripts/reference_shannon_base.py` computes `-sum(p_i * ln(p_i)/ln(base))`
directly with 120-digit Decimal arithmetic and exact rational counts. Bases
are converted from the actual binary `f64` values, not their shortened decimal
display. The 144 fixtures cover common units, bases adjacent to one and two,
very large bases, and counts through `u64::MAX`. Rust skips count totals that
cannot fit the target's `usize`. Unit conversion is also checked through
uniform-law logarithmic values, base monotonicity, and base-2 compatibility.
The accuracy thresholds and inherited extreme-count caveat are documented in
[numerical behavior](../numerical-behavior.md#shannon-with-an-explicit-base).
