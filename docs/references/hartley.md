# Hartley reference notes

## Original definition

Hartley (1928), *Transmission of Information*, pages 538–540, develops a
logarithmic measure of the number of distinguishable possibilities. The project
applies this counting measure to the observed single-symbol support, in base 2.
Full bibliographic entries are in the [bibliography](../references.md).

## Modern reference and formula

Rényi (1961), *On Measures of Entropy and Information*, gives the entropy family
`H_alpha = log2(sum(p_i^alpha)) / (1 - alpha)`. For a finite probability law,
taking `alpha -> 0+` over its positive probabilities yields `H0 = log2(k)`.
Each positive term tends to one; zero terms remain zero. This derivation avoids
an ambiguous evaluation of `0^0`. General Rényi entropy is not implemented here.
Cover and Thomas, second edition, chapter 2, treats Shannon's log-support bound.

## Implementation convention

Support is counted from positive integer counts. Zero counts are absent; there
is no probability threshold. Empty input returns zero by project convention,
outside the nonempty-law formula. Units are bits per symbol.

## Known variants

Counting the theoretical alphabet instead of observed support gives a different
quantity. Total information over all possible length-n strings also differs
from this single-symbol measure. Logarithm bases change units; the name Hartley
does not select the base-10 information unit sometimes called a hartley.

## Numerical issues

Support is at most 256 and exactly representable in `f64`. Only the logarithm
introduces nontrivial rounding. See [numerical behavior](../numerical-behavior.md).

## Test sources

- Manual support counts for canonical sequences and non-text byte values.
- An independent Python set-based calculation with 80-digit decimal logarithms
  covering support sizes 0 through 256, with Rust testing different frequencies
  and symbol labels against the committed results.
- Properties: Shannon's upper bound, equality for uniform frequencies,
  invariance under permutation/relabeling/repetition and changed positive counts,
  and nondecreasing entropy when observations are appended.
- Count tables with totals through `usize::MAX` to check rare-symbol inclusion.

## Open questions

Alternate bases, probability-vector support semantics, and generic alphabets
remain separate future design decisions. There are no unsettled Hartley-specific
conventions for this byte-count API.
