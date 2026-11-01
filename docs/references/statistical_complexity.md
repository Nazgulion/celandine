# Normalized Jensen–Shannon statistical complexity: references and verification

## Primary sources

- R. López-Ruiz, H. L. Mancini and X. Calbet (1995), *A statistical measure of
  complexity*, Physics Letters A 209, 321–326.
  [DOI](https://doi.org/10.1016/0375-9601(95)00867-5).
  The entropy-times-disequilibrium starting point; its Euclidean variant is
  not the function implemented here.
- P. W. Lamberti, M. T. Martín, A. Plastino and O. A. Rosso (2004), *Intensive
  entropic non-triviality measure*, Physica A 334, 119–131.
  [DOI](https://doi.org/10.1016/j.physa.2003.11.005).
  Introduces the Jensen–Shannon disequilibrium construction.
- O. A. Rosso et al. (2012), *Causality and the entropy–complexity plane:
  Robustness and missing ordinal patterns*, Physica A 391, 42–55, section 2.1,
  equations 2–6, page 45. [DOI](https://doi.org/10.1016/j.physa.2011.07.030),
  [author-institution PDF](https://ri.conicet.gov.ar/bitstream/handle/11336/105235/CONICET_Digital_Nro.6aedcb8c-a174-423b-9f69-c0163d3b2414_D.pdf?sequence=5).
  Explicit normalized factors and point-mass maximum. Its ordinal-pattern
  construction is not used by this byte-frequency API.

## Convention and derivation

The [metric page](../complexity/jensen_shannon.md) defines the state space,
units, worked examples and error precedence. Both factors are normalized, not
the attainable maximum of their product. Alphabet K is caller-declared, never
inferred from observed support; K=1 has undefined normalizers and is rejected.
No probability-vector importer, ordinal transform or source estimator is added.

For a point mass and U_K, the mixture has one probability `(K+1)/(2K)` and
K-1 probabilities `1/(2K)`. Substitution gives
`Jmax = 1 + log2(K)/2 - (1+1/K)*log2(K+1)/2`.
Using `log2(K+1) = log2(K) + log2(1+1/K)` gives the evaluated form
`1 - (log2(K)/K + (1+1/K)*ln1p(1/K)/ln(2))/2`.

The numerator reuses stable count-based Jensen–Shannon divergence against a
uniform K-unit count table. The identities of absent states do not affect the
sum. Entropy retains a majority state's complementary count before `ln1p`,
with compensated summation. No clamping or smoothing is applied.

## Independent checks and regression

`python3 scripts/reference_statistical_complexity.py --check` verifies 853
fixtures using 160-digit Decimal arithmetic. It directly evaluates mixture
entropies and separately evaluates the point-mass mixture for normalization,
rather than using the library's gap series or closed-form denominator.
Cases cover every K in 2..=256, point masses, full and partial uniform laws,
seeded mixed counts, parameter errors, and rare/near-uniform laws at 32, 53, 60
and 64-bit count scales. Nonzero results require relative error <=5e-13;
endpoint zeros require exact positive zero. Wider count cases are skipped on
narrower machines, without asserting runtime support there.

Initial reuse of ordinary rounded probabilities failed these checks: for
`[usize::MAX-1, 1]` at K=2 on x86-64, it returned 3.469446951953614e-18 instead
of about 3.547655606831908e-18. The near-certain-limit regression and the fixtures
failed before the complementary-count entropy fix and passed afterward. The
standalone Shannon API remains unchanged; this local evaluation has the same
mathematical definition with better relative behavior at that boundary.

Seeded tests additionally check order invariance, byte-label permutation,
repetition invariance, byte/distribution agreement, range and error precedence.
Dedicated fuzzing and allocation/performance evidence supplement these fixtures.
Tolerances are tested criteria on supported platforms, not global error bounds.
