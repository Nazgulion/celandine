# Mathematical conventions

The library implements empirical, single-symbol Shannon and Hartley entropy for
finite byte sequences. The project plan is the architectural specification;
later metrics and sequence transformations remain future work.

## Units and arithmetic

Entropy uses base-2 logarithms and is reported in **bits per symbol**. All
probability and entropy arithmetic uses `f64`. There is no base parameter or
normalized-entropy API yet. Counts and sample sizes use `usize`.

## Counts and empirical probabilities

For a sequence of length `n > 0`, let `c[b]` be the number of occurrences of byte
`b`. The empirical probability is `p[b] = c[b] / n`. The alphabet has 256 possible
values; the observed support consists only of bytes with positive counts.
Sample size, support size, and alphabet cardinality are different quantities.

`ByteHistogram` owns `[usize; 256]` and its total. Fields are private and immutable
through the public API. The total equals the integer sum of the counts.
Construction from a slice cannot overflow: no count exceeds the slice length.
Construction from an existing count array checks the sum and returns
`CountOverflow` if it exceeds `usize::MAX`.

`Distribution` owns one histogram. It derives probabilities lazily, retaining
the exact counts and sample size, without storing another table or recounting
the original sequence. Its probability iterator covers **all 256 bytes in byte
order**, including zero entries. Support size counts positive integer counts.
For nonempty input the probabilities sum to one mathematically; the floating
sum can differ slightly from one.

## Empty and constant inputs

An empty histogram/distribution has sample size zero, support size zero, and
returns zero for each probability query. This is an **empty empirical state**,
not a normalized probability distribution: there is no empirical law when no
observations exist. All Shannon and Hartley entry points return positive `0.0`
for this state by API convention. This does not define a probability law on an
empty sample or assert that `log2(0)` is zero. Callers needing to distinguish
missing data must check `sample_size()` or `is_empty()` first.

Every nonempty constant sequence has one probability equal to one and entropy
exactly positive `0.0`. Zero-probability terms contribute zero, using the
continuous extension `0 log2(0) = 0`; the implementation never evaluates `log2(0)`.

## Hartley support convention

For a nonempty empirical distribution, Hartley entropy is `H0 = log2(k)`, where
`k` is the number of strictly positive integer counts. Unobserved byte values do
not contribute. Frequencies and sample size do not affect the result once the
observed support is fixed. No probability conversion or numerical threshold is
used to decide support, including for very large counts.

The result is in bits per symbol, with `0 <= H0 <= 8`. It is the Shannon entropy
of a uniform law on the observed support and the order-zero Rényi entropy of the
empirical law. For nonempty input, `H_Shannon <= H0`, with equality when the
observed frequencies are uniform (subject to floating-point tolerance).

This measures observed possibilities, not a known theoretical alphabet or an
unknown source's full support. A finite sample can miss possible source symbols.
The empty-state extension is shared with Shannon, not derived from the formula.

## Validation and normalization

Only integer counts and byte slices can construct this milestone's empirical
distribution. Arbitrary floating-point probability vectors are deliberately
not accepted. Negative, NaN, infinite, or non-unit-mass probability vectors
therefore cannot enter the API. Validation is by construction, with checked
count totals, rather than approximate floating-point acceptance.

Dividing counts by their sample size is the explicit definition of an empirical
law, not repair of an invalid probability vector. There is no automatic
renormalization, smoothing, clamping, or discarded invalid value. An eventual
probability-vector API must separately specify an explicit finite tolerance,
reject negative/nonfinite entries and invalid mass, and expose any normalization
as an explicit operation. That API is deferred.

## Tolerance and reproducibility

Tests compare entropy with absolute tolerance `1e-12` bits and probability sums
with absolute tolerance `1e-12`. These are test acceptance thresholds for the
256-symbol `f64` path, not changes to the mathematical definition or rigorous
error bounds for every platform. No runtime tolerance is used to alter output.
See [numerical behavior](numerical-behavior.md) for precision limitations.

Counting and summation order are deterministic. Repeated calls in the same
binary/runtime produce identical results. Bitwise equality across platforms,
compiler versions, or math-library implementations is not promised.

## Meaning of the result

Both mathematical quantities are exact for the empirical law; their numerical
values are evaluated approximately in floating point. Used to infer an unknown
source's Shannon entropy, the empirical Shannon result is the plug-in estimator,
with finite-sample bias. Hartley uses only observed support and cannot recover
unseen source symbols. Neither measures temporal dependence or an unknown
source's entropy rate. Permuting input preserves both computed results.
