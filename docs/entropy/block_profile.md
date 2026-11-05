# Block-entropy profiles

A block-entropy profile collects the empirical uncertainty of overlapping byte
blocks at several lengths. Comparing these values shows how observed pattern
frequencies change as a block includes more bytes. The API returns a lazy
iterator: it computes only requested values, letting callers bound the work.

## Definition, formula and units

For an input of `L` bytes and maximum block length `max_order`, let
`r = min(max_order, L)`. The profile contains

```text
H_1, H_2, ..., H_r
H_n = -sum_g p_n(g) * log2(p_n(g))
p_n(g) = count of block g / (L - n + 1)
```

Here `n` is block length in bytes, `g` ranges over distinct observed n-byte
blocks, and all complete overlapping starts are counted once. Every value is
**bits per n-byte block**, without division by n. Forward entry at index i is
`H_(i+1)`. The scalar `block_entropy` computes each entry, including its
floating-point conventions and support bounds.

Claude Shannon's 1948 *A Mathematical Theory of Communication*, section 7,
Theorem 5 considers entropy of symbol blocks divided by block length and its
relation to source entropy. This profile exposes the unscaled empirical block
entropies at finite lengths. See the
[original paper](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf).
The iterator's cap and laziness are library design choices.

## Worked example

For `AABB` and maximum length 4:

| n | Observed block counts | H_n, bits per block |
| --- | --- | ---: |
| 1 | A:2, B:2 | 1 |
| 2 | AA:1, AB:1, BB:1 | log2(3), about 1.584963 |
| 3 | AAB:1, ABB:1 | 1 |
| 4 | AABB:1 | 0 |

A limit of 2 yields only the first two entries. A limit of 100 still yields
four, because longer blocks have no observations. To keep lengths attached to
values during later traversal, use `.enumerate()` before `.rev()`, or zip the
fresh profile with lengths `1..=profile.len()`.

## Domain, boundaries and iterator contract

- Input is any byte slice, with no text interpretation, padding or wrapping.
- Every usize maximum is accepted. Zero requests an empty profile, not H_0;
  the scalar block-entropy API continues to reject block length zero.
- Empty input yields no entries. A singleton yields one positive-zero entry.
- Limits above L, including usize::MAX, are capped at L; no artificial zero
  tail is appended. Scalar block entropy remains available for individual
  oversized lengths, where it returns zero by convention.
- Iteration is exact-size, double-ended, fused and cloneable. Clones independently
  retain the remaining range and borrow the same input.
- Construction, cloning, length queries and `count()` take O(1) time and allocate
  nothing. `nth`/`nth_back` skip lengths in O(1) before evaluating only the chosen
  entry; oversized skips exhaust the iterator safely. `last()` evaluates only
  the last remaining entry. Exhausted operations calculate no entropy.
- Each yielded value is computed independently, without caching tables across
  lengths. Calling iterator adapters that consume values performs those scalar
  calculations; collecting additionally allocates the output vector.

## Interpretation and limitations

This is a profile of empirical laws, not an entropy-rate estimator. At each n,
there are only L-n+1 overlapping observations. As n approaches L, data become
sparse; H_L is always zero for nonempty input because only one block is observed.
The finite profile need not increase, be concave, or have nonnegative adjacent
differences. Subtracting its entries does not generally produce conditional
entropy: observation sets and boundary marginals differ. Use
`conditional_entropy_order` for a calculation with matched context/target samples.
There is no smoothing, bias correction, or implicit normalization by n.

## Complexity and numerical behavior

The iterator itself stores a borrowed slice and two endpoints, using O(1)
space without heap allocation. Evaluating length n has the scalar block-entropy
cost: with m=L-n+1 observations and s distinct blocks, worst-case time is
`O(m*n*log(s+1) + s)` and temporary tree storage is O(s). Length one uses the
fixed byte histogram; length L returns zero without counting. Skipping work
avoids that cost entirely, while querying a chosen value still incurs it.

Consuming r entries adds their scalar costs. A conservative bound for a full
profile is `O(L^3*log(L+1))` time and O(L) peak temporary tree storage; collecting
also stores r floats. Full profiles are intended for short inputs. Choose a
small maximum or select specific entries for large inputs; laziness does not
make a fully consumed profile cheap.

Every entry agrees bit-for-bit with the scalar byte API at that length. That
API uses exact integer counts, rounded f64 probabilities and compensated
summation for n>1, with its existing Shannon path for n=1. Independent 100-digit
profile fixtures use `1e-12` absolute bit tolerance, a test threshold rather
than a universal error guarantee. No normalization or clamping is added.
