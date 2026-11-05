# Aligned conditional entropy: references and verification

C. E. Shannon (1948), *A Mathematical Theory of Communication*, Bell System
Technical Journal 27, 379–423 and 623–656,
[original paper](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf).
Section 6, property 5 defines conditional entropy as a probability-weighted
average and derives the chain rule. Section 12 interprets message uncertainty
after reception as equivocation. Celandine applies the empirical quantity to
aligned bytes, with its own explicit H(first|second) API direction and empty
convention. See [the metric definition](../entropy/conditional.md).

## Independent oracle

`python3 scripts/reference_conditional_pair.py --check` verifies 1,397 fixtures
using 100-digit Decimal H(X,Y)-H(Y). Each entropy is evaluated as
`log2(N)-sum c*log2(c)/N`; this is independent of the production conditional
ratio/log1p calculation. Exact deterministic cases are recognized in integers
by one observed X per observed Y, rather than a floating threshold.

Fixtures cover every binary pair sequence through length five, deterministic
seeded samples on 2/16/256-symbol alphabets, both directions of worked examples,
unequal conditioning-group weights, rare outcomes within one group, rare
ambiguous groups among deterministic groups, and both sides of the conditional
probability one-half branch. Validation uses 1e-12 absolute bits and additional
1e-12 relative tolerance for positive references below 1e-4 bits.

## Identities and numerical checks

Hand calculations establish H(ABCD|XXYY)=1 and H(XXYY|ABCD)=0, and unequal
weights H(AAAABC|XXXXYY)=1/3. Tests cover all 65,536 byte pairs, diagonal and
many-to-one deterministic laws, length errors and positive zeros. Fixed-seed
properties verify bounds by H(X), both chain rules, directional differences,
sample permutation, bijective label renaming, repeated-sample scaling and
byte/reuse bit agreement. First-order sequence conditioning is independently
cross-checked using target=data[1..], given=data[..len-1].

A private kernel test uses a one-count gap at usize::MAX. Its majority term
has leading value 1/(N ln 2), with relative correction O(1/N); forming a floating
ratio first would erase the gap on 64-bit targets. This verifies precision
without introducing artificial public count-import constructors. All public
joint laws still come from actual aligned slices.

Allocation checks cover reuse, empty inputs and length errors. New memory-probe
workloads are isolated from the historical mutual and normalized-LZ76 cases.
Fuzzing and measured performance supplement mathematical verification; they do
not prove exact source predictability or correctness for every possible input.

## Review outcome

Review checked first-given-second direction, exact marginal construction,
frequency weighting, deterministic branches, floating stability, ownership,
error precedence and allocation contracts. No new production defect was found.
All 64 standard validation commands passed, including both feature sets and
build profiles, reference checkers, examples, warning-free docs, stable fuzz
replay and benchmark compilation. See [fuzz evidence](../fuzzing/conditional_pair.md)
and [performance](../benchmarks/conditional_pair.md) for their separate scopes.
