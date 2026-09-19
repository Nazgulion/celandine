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
in bits per symbol. Zero terms are omitted, as justified by continuity. Empty
input returns zero solely by documented project convention.

## Known variants

Different bases change units. Dividing by an alphabet-dependent maximum gives
a normalized score and requires an explicit choice of alphabet. Bias-corrected
source estimators and entropy rates are distinct from the empirical marginal
entropy implemented here. None of these variants is implemented yet.

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

The probability-vector constructor and its explicit tolerance policy, alternate
log bases, and a cross-platform precision policy need separate design review.
They are not prerequisites for this empirical byte API.
