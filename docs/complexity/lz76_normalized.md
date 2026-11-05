Asymptotically normalized exhaustive LZ76 complexity with an explicit alphabet.

Two named scales make the normalization choice visible:

- `lz76_phrase_density(x) = c/N`, the fraction of byte positions beginning a
  phrase. This simple library rescaling is bounded by one, but is not an
  entropy-rate estimator.
- `lz76_complexity_asymptotic(x, K) = c*log_K(N)/N`, dividing the count by the
  asymptotic reference `N/log_K(N)`. It can exceed one on finite inputs.

Here `x` is a finite byte sequence, `N` its length in byte symbols, `c` its exact
exhaustive LZ76 phrase count, and `K` the declared number of possible byte states,
including unobserved ones. `log_K(N) = log2(N)/log2(K)`. Both results are
**dimensionless**, not compressed bytes or bits. The asymptotic scale expresses
an entropy-rate-like quantity relative to `log2(K)` bits per symbol, under the
source assumptions of the limiting theory; a finite result is not an exact
source entropy. A byte is a symbol, not a Unicode character.

### Why these scales matter

Raw counts grow with length. Naming the scale allows reproducible comparisons
with stated lengths and alphabets. Neither rescaling removes all finite-size
bias. `Lz76Summary::from_bytes` retains length, count and observed support so both
scales, or several declared alphabets, can be queried without parsing again.

### History

Lempel and Ziv introduced exhaustive production complexity in 1976, including
its asymptotic growth relative to sequence length and alphabet size
([original paper](https://doi.org/10.1109/TIT.1976.1055501)).
Estevez-Rams and colleagues discuss `c/(N/log_K(N))` and its finite-length
overshoot in their 2013 preprint, especially equations (4)–(6). Their work also
examines alternative normalizations using finite maximum-complexity sequences
([primary research](https://arxiv.org/abs/1311.0546)). This API implements the
explicit asymptotic formula, not finite-maximum or shuffled-surrogate scaling.
The `c/N` name denotes a library convenience, not a separately attributed metric.

### Worked example

`ABAB` has exhaustive history `A | B | AB`, including the final reproducible
suffix: `N=4`, `c=3`, observed support two. Density is `3/4=0.75`.
With declared `K=2`, the asymptotic value is `3*2/4=1.5`; with `K=4`, it is
`3*1/4=0.75`. The alphabet choice is part of the measurement.

```
use celandine::complexity::{lz76_complexity_asymptotic, Lz76NormalizationError};
assert_eq!(lz76_complexity_asymptotic(b"ABAB", 2), Ok(1.5));
assert_eq!(lz76_complexity_asymptotic(b"ABAB", 4), Ok(0.75));
assert_eq!(lz76_complexity_asymptotic(b"ABC", 2), Err(Lz76NormalizationError::AlphabetTooSmall));
assert_eq!(lz76_complexity_asymptotic(b"", 1), Err(Lz76NormalizationError::InvalidAlphabetSize));
```

### Domain, errors and edge cases

Validation checks `2 <= K <= 256` first, then observed support at most `K`, before
parsing. No alphabet is guessed, enlarged or required to have byte values below
`K`: `A` and `B` are valid labels for `K=2`. `K=1` is invalid because logarithm
base one is undefined. Errors implement `Display` and `std::error::Error`.

For valid `K`, empty input returns positive zero by convention; a singleton also
returns zero because `log_K(1)=0`. Density is respectively zero and one. Constant
inputs of length `N>=2` have `c=2`, hence density `2/N` and asymptotic value
`2*log_K(N)/N`; they are not forced to zero. All 256 distinct byte symbols give
`c=N=K=256`, so both scales equal one.

### Numerics and cost

The integer count is exact. Conversion, division and platform logarithms use
`f64`; results are approximate and are neither rounded nor clamped to `[0,1]`.
Evaluation uses `(c/N)*(log2(N)/log2(K))` to avoid integer multiplication. Tests
use independent 100-digit Decimal logarithms with `1e-12` absolute tolerance,
plus exact canonical results and bitwise agreement for cached queries.

Density inherits O(N²) worst-case time and O(1) auxiliary space. Asymptotic
normalization and summary construction add a fixed 256-bin histogram pass:
O(N² + 256) worst-case time and fixed auxiliary storage, no heap allocations.
Invalid alphabet range exits in O(1); invalid support exits after O(N+256) work.
Summary queries take O(1) and allocate nothing. Large diverse strings remain
expensive; normalization does not accelerate the raw parser.

### Interpretation limits

This uses exhaustive LZ76 with overlapping copies and a counted final suffix,
not LZ78 dictionary phrases or a compressor's encoded length. Order matters;
reversal invariance is not promised. The asymptotic value is nonnegative, with
the loose bound `log_K(N)` for `N>=2`, but need not be at most one. High values
are not a randomness certificate, Kolmogorov complexity, or an exact entropy
rate. Source-limit interpretations require appropriate stationary/ergodic
assumptions and do not guarantee accuracy at a finite sample length.
