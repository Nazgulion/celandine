# Empirical block entropy

Block entropy measures the uncertainty of an overlapping byte block selected
uniformly from the complete blocks observed in a finite sequence. It reveals
local patterns that a single-byte histogram discards. It is Shannon entropy
applied to blocks, with each distinct block treated as one outcome.

## History and interpretation

Claude Shannon's 1948 *A Mathematical Theory of Communication* introduced
entropy in communication theory. Its section 7, Theorem 5 considers probabilities
of whole symbol sequences and their entropy per symbol; this API returns the
unscaled entropy of a block. See the
[original paper](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf).

## Formula and units

For input length `L` bytes and positive block length `n` bytes, use every start
position once, with stride one and no padding or wrapping. There are
`m = L - n + 1` observations when `n <= L`, otherwise none. If block `g` occurs
`c_g` times, its empirical probability is `p_g = c_g / m`. Then

```text
H_n = -sum_g p_g * log2(p_g)
```

The sum covers distinct observed blocks. Each term is the probability times
the information of that outcome, averaged over the empirical law. The result is
in **bits per n-byte block**, without division by `n`. For nonempty observations,
`0 <= H_n <= log2(k) <= log2(m)`, where `k` is the distinct-block count;
also `H_n <= 8*n` mathematically. Bounds allow floating-point rounding.

## Worked example

`ABABA` at `n=2` gives `AB, BA, AB, BA`: two outcomes with counts 2 and 2,
probabilities 1/2 and 1/2, and entropy `-2*(1/2)*log2(1/2) = 1` bit per block.
`AABBA` has the same byte counts, but its four pairs `AA, AB, BB, BA` are
distinct, giving 2 bits per block. This distinguishes local arrangements.

## Domain and edge cases

- Inputs are arbitrary byte slices, without text decoding or source assumptions.
- `n=0` returns `InvalidNgramLength`, even on empty input; the empty word is
  not an observed block in this API.
- Empty input or `n > L` returns positive zero by the library's empty-entropy
  convention. This does not claim that an unknown source is deterministic.
- `n=L>0` and any constant input return positive zero (one distinct outcome).
- `n=1` agrees with byte Shannon entropy; the byte entry point reuses `shannon`.
- `usize::MAX` is safe as a block length and yields zero for realizable slices.

## Exact quantity and estimation limits

The quantity is exact for the empirical law, evaluated in `f64`. As an estimate
of unknown source block entropy it has finite-sample limitations; increasing
`n` rapidly sparsifies observations, which overlap and need not be independent.
No smoothing, bias correction, source entropy rate, or normalization is inferred.
Finite-sample `H_n` need not increase with `n`: `ABABA` has `H_2=1` but `H_5=0`.
Subtracting entropies at different lengths does not automatically yield a valid
conditional entropy, because their observation sets and boundary marginals differ.
Use `conditional_entropy_order` for a calculation with matched context/target
observations.

This differs from measuring byte entropy *inside* each sliding window, and from
partitioning a sequence into non-overlapping fixed blocks.

## Complexity and numerical behavior

The byte API counts with the existing borrowed-key n-gram tree: worst-case
`O(m*n*log(k+1) + k)` time and `O(k)` tree storage, without copying block bytes.
Long common prefixes make comparisons expensive, and diverse long blocks can
use substantial memory. The `n=1` path takes `O(L+256)` time and no heap;
`n>=L` takes constant time and no heap after validation.

The distribution API takes `O(k)` time, `O(1)` additional storage and no new
heap allocation. Counts stay exact in `usize`; conversion above `2^53` can round.
It uses compensated summation and `ln_1p` on the majority's complement to reduce
rounding near probability one. The byte `n=1` path retains existing Shannon
rounding, so distribution reuse may differ in the last few bits. Independent
100-digit fixtures use absolute tolerance `1e-12` bits; this is a test tolerance,
not a universal error bound for arbitrary hardware or inputs. No result is clamped.
