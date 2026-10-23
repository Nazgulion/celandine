# Total variation reference notes

## Definition and historical context

David A. Levin and Yuval Peres, with contributions by Elizabeth L. Wilmer (2017),
*Markov Chains and Mixing Times*, second edition, American Mathematical Society,
Chapter 4, Proposition 4.2 and Remark 4.3.
[Author-hosted text](https://pages.uoregon.edu/dlevin/MARKOV/mcmt2e.pdf).
Their exposition uses this classical distance to study approach to equilibrium,
proves the half-L1 identity, and gives the positive-part identity used here.
It is a mathematical reference, not a claim that these authors invented TV.

For a finite alphabet, `TV(P,Q) = max_A |P(A)-Q(A)| = sum_i |p_i-q_i|/2`.
`A` ranges over subsets of the alphabet. Celandine uses the probability-distance
convention with range `[0,1]`; the unhalved L1 norm has range `[0,2]`.

## Implementation convention and derivation

Byte labels identify the same events in both inputs. Nonempty samples define
laws independently, even if sample lengths differ. Empty states have no law,
so either empty input gives an error, including two empties. There is no
smoothing or alignment of sample positions.

Since both laws sum to one, positive and negative probability differences have
equal total magnitude. Write each difference as `(a_i*M-b_i*N)/(N*M)`. Sum only
positive integer numerators, then divide by the common denominator. Each term
is at most `a_i*M`; the whole positive sum is at most `N*M`. For at most 64-bit
sample totals these fit in `u128`, whereas the sum of absolute numerators could
overflow. This is why the implementation does not accumulate the full L1 sum.
The exact subtraction also avoids cancellation after floating conversion.

For `AAAB` against `AB`, the positive count numerator is `3*2-1*4=2`;
`N*M=8`, giving `2/8=1/4`. This agrees with the independently evaluated half-L1
worked example on the [metric page](../divergence/total_variation.md).

## Verification and numerical limits

`scripts/reference_total_variation.py` uses arbitrary-precision Python integers
and `Fraction` subtraction/absolute values for the half-L1 formula, independently
of the positive-sum implementation. It retains reduced rational numerators and
denominators in 148 TSV rows. Small alphabets are also checked by enumerating
all events. Rust properties enumerate events directly from observations and
check identity, symmetry, triangle inequality, sample repetition, permutation,
shared relabeling, and byte/distribution equivalence. Wide-count properties and
fuzzing use the independent overlap-complement identity `1-sum_i min(p_i,q_i)`.

Zeros are exact; nonzero rational fixtures use `1e-15` relative tolerance.
Final integer conversion/division may round, including values near one rounding
to one. See [numerical behavior](../numerical-behavior.md#total-variation-distance).
Source-distribution estimation and arbitrary probability-vector inputs remain
outside this increment; no total-variation convention is left implicit.
