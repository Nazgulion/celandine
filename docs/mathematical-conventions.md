# Mathematical conventions

The library implements empirical, single-symbol Shannon, Hartley, Rényi, collision,
min-entropy, and Tsallis entropy for finite byte sequences. The project plan is
the architectural specification; later metrics and sequence transformations
remain future work.

## Units and arithmetic

Shannon, Hartley, Rényi, collision, and min-entropy use base-2 logarithms and
are reported in **bits per symbol**. Tsallis uses the fixed Shannon-bit scaling
defined below; its values are not generally average code lengths. All probability
and entropy arithmetic uses `f64`. There is no base parameter or normalized-entropy
API yet. Counts and sample sizes use `usize`.

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
observations exist. All implemented entropies return positive `0.0` for this state
by API convention, after validating any order parameter. This does not define a
probability law on an empty sample or assert that `log2(0)` is zero. Callers needing to distinguish
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

Tests compare entropy with absolute tolerance `1e-12` in the documented units
and probability sums with absolute tolerance `1e-12`. These are test acceptance thresholds for the
256-symbol `f64` path, not changes to the mathematical definition or rigorous
error bounds for every platform. No runtime tolerance is used to alter output.
See [numerical behavior](numerical-behavior.md) for precision limitations.

Counting and summation order are deterministic. Repeated calls in the same
binary/runtime produce identical results. Bitwise equality across platforms,
compiler versions, or math-library implementations is not promised.

## Meaning of the result

These mathematical quantities are exact for the empirical law; their numerical
values are evaluated approximately in floating point. Used to infer an unknown
source's Shannon entropy, the empirical Shannon result is the plug-in estimator,
with finite-sample bias. Hartley uses only observed support and cannot recover
unseen source symbols. None measures temporal dependence or an unknown
source's entropy rate. Permuting input preserves their computed results.

## Rényi orders

`renyi_entropy(data, alpha)` and `renyi_entropy_distribution(distribution, alpha)`
accept every finite nonnegative `f64` order and positive infinity. Negative zero
is order zero. Negative orders (including negative infinity) and NaN return
`InvalidRenyiOrder`, even for empty or constant data. Validation precedes counting
in the byte entry point. Valid calls return `Ok(f64)` in bits per symbol.

For finite positive `alpha != 1`, use
`H_alpha = log2(sum(p_i^alpha)) / (1 - alpha)` over positive empirical counts.
Order zero uses exact observed support (Hartley); order one delegates to Shannon;
positive infinity uses `-log2(max(p_i))`. Zero-count symbols never contribute,
including at order zero; no `0^0` is evaluated. Empty and singleton support return
positive zero for every valid order, under the existing empty-state convention.

Mathematically, entropy is nonincreasing in order and lies between zero and
`log2(support) <= 8`. Uniform empirical distributions have that upper-bound value
for every order. No near-one interval is replaced with Shannon: only exact order
one delegates. The numerical formulas are detailed in `numerical-behavior.md`.
Negative-order entropy and arbitrary probability-vector input remain out of scope.

## Collision entropy

`collision_entropy(data)` and `collision_entropy_distribution(distribution)`
return `f64` in bits per symbol. They evaluate Rényi entropy at the fixed valid
order two: `H2 = -log2(sum_i p_i^2)`. No parameter validation or `Result` is
needed at these entry points. Empty input returns positive zero by convention;
a nonempty constant sequence returns positive zero mathematically.

For a nonempty empirical law, `sum_i p_i^2` is the probability that two independent
draws with replacement have equal symbols. Equivalently, among all `n*n` ordered
pairs of sample positions, including pairing a position with itself, exactly
`sum_i c_i*c_i` have equal symbols. This is not adjacent-pair counting and not
sampling without replacement. The API returns entropy, not the matching probability.
The draws are a mathematical interpretation of the empirical law, not an
assumption that the original observations were independent.

Collision entropy is exact for the empirical law up to floating-point rounding;
it is a plug-in estimate when used to infer a source quantity. No correction for
finite-sample bias or unseen symbols is applied. For nonempty input,
`0 <= H2 <= H_Shannon <= H0 <= 8`, with equality throughout for uniform frequencies.
The implementation shares Rényi's order-two numerical behavior and does not
recount an existing distribution or allocate memory.

## Min-entropy

`min_entropy(data)` and `min_entropy_distribution(distribution)` return `f64`
in bits per symbol: `H_inf = -log2(p_max)`, where `p_max = max_i c_i / n` for a
nonempty sample. They use the positive-infinite-order Rényi limit with no user
parameter. Empty input returns positive zero by convention, without defining a
maximum empirical probability; constant input returns positive zero mathematically.

`p_max` is the greatest probability of a correct fixed-symbol guess for one draw
from the empirical law. Its negative logarithm is the minimum surprise among
observed symbols. Tied maxima give the same result; no symbol label is selected
or returned. Other frequencies matter only through the total and largest count.
The result ignores order and does not infer conditional uncertainty, entropy
rate, or an unknown source's probabilities. No independence assumption is made
about the observations.

For nonempty input, `0 <= H_inf <= H2 <= H_Shannon <= H0 <= 8`. Uniform empirical
frequencies give `log2(support_size)` for all these measures. The function reuses
the stable Rényi infinity calculation and retains its floating-point contract.

## Tsallis orders and scale

`tsallis_entropy(data, q)` and `tsallis_entropy_distribution(distribution, q)`
accept finite nonnegative `f64` orders. Negative zero is order zero. Negative
orders, NaN, and both infinities return `InvalidTsallisOrder` before empty or
constant handling, and before counting in the byte API. Valid calls return
`Result<f64, InvalidTsallisOrder>`.

For nonempty input and `q != 1`, define
`S_q = (1 - sum_i p_i^q) / (q - 1) / ln(2)` over positive empirical counts.
The constant `1/ln(2)` is fixed across orders: the `q -> 1` limit is base-2
Shannon entropy. We call this **Shannon-bit scaling**. The unscaled formula in
the plan has a natural-log Shannon limit; its allowed convention choice is made
explicit here. This is not division by the maximum entropy or a repair of the
probability distribution. Other generalized-logarithm scalings are not used.

At exact `q=1`, delegate to Shannon, including its numerical limitations. At
`q=0`, return `(k-1)/ln(2)` for observed support `k>0`; unobserved symbols are
excluded and `0^0` is never evaluated. Empty and constant inputs return positive
zero for every valid order. Empty input has no empirical law and is handled
before the formula. Infinite-order Tsallis limits are outside this API.

For support `k>=1`, the maximum is the uniform-law value
`U_q(k) = (1-k^(1-q))/(q-1)/ln(2)` when `q!=1`, and `U_1(k)=log2(k)`.
Thus `0 <= S_q <= U_q(k) <= 255/ln(2)` for byte distributions with `q>=0`.
This is not an 8-bit bound. Entropy is nonincreasing in `q`, and uniform laws
usually have different values at different orders. Rounding can slightly violate
bounds; results are not clamped. At `q=2`, the value is the probability of two
independent draws differing, divided by `ln(2)`, not collision entropy itself.

For independent product laws `P` and `Q`, this scaling gives
`S_q(P*Q) = S_q(P) + S_q(Q) + (1-q)*ln(2)*S_q(P)*S_q(Q)`.
This composition rule illustrates why Tsallis is generally not an additive
average code length. The implemented quantity concerns the empirical law and
ignores sequence order; it assumes no source model or independence of input bytes.
