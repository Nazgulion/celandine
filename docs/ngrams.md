# Overlapping byte n-grams

An **n-gram** is a contiguous block of `n` bytes. Sliding one byte at a time
collects every complete block, including overlapping occurrences. This preserves
local order that a single-byte histogram discards: `AABB` and `ABAB` have the
same byte counts but different two-byte blocks.

## Background and purpose

Contiguous blocks are standard building blocks for sequence statistics.
Manning, Raghavan, and Schütze's *Introduction to Information Retrieval* (2008),
section 3.2.2, illustrates overlapping character k-grams for indexing. Celandine
uses the same contiguous-block idea for arbitrary bytes, with **no boundary
markers**. This is reusable counting infrastructure, not a newly named metric.
See the [authors' textbook section](https://nlp.stanford.edu/IR-book/html/htmledition/k-gram-indexes-for-wildcard-queries-1.html).

N-gram counts describe which local patterns occur and how often. Relative
frequencies make samples with different numbers of complete blocks comparable.
These primitives support future sequence measures; they do not themselves
calculate block entropy, conditional entropy, or an entropy rate.

## Definition and formula explained

Let `x` be a byte sequence of length `L` bytes, and `n > 0` the requested block
length in bytes. There are `m = L - n + 1` complete occurrences when `n <= L`,
and `m = 0` otherwise. At each starting position `i = 0, ..., m-1`, extract
`g_i = x[i..i+n]`: the bytes starting at `i`, excluding position `i+n`.

For a block `g` of length `n`, its count is
`c(g) = sum_{i=0}^{m-1} 1[g_i = g]`. The indicator `1[condition]` is one if
the condition is true and zero otherwise; the sum counts matching positions.
The support size `k` is the number of different blocks with positive counts.
Counts and `m` have units of block occurrences; they use exact `usize` integers.

For `m > 0`, the empirical probability is `p(g) = c(g)/m`, a dimensionless
fraction. It is the chance of finding `g` when choosing uniformly among the
observed complete starting positions. Probabilities sum to one mathematically.
This defines a distribution over length-`n` blocks, not a conditional next-byte
distribution. It makes no independence or source-model assumption.

## API and worked example

```rust
use celandine::transforms::ngrams;
use celandine::distribution::{ngram_counts, ngram_probabilities, NgramDistribution};

let data = b"ABABA"; // L = 5 bytes, n = 2 bytes, m = 4 occurrences
let blocks: Vec<_> = ngrams(data, 2)?.collect();
assert_eq!(blocks, vec![b"AB", b"BA", b"AB", b"BA"]);

let counts = ngram_counts(data, 2)?;
assert_eq!(counts.total(), 4);
assert_eq!(counts.support_size(), 2);
assert_eq!(counts.count(b"AB"), 2);
assert_eq!(counts.count(b"BA"), 2);

// Reuse counts; no new count table or probability array is allocated.
let d = NgramDistribution::from_counts(counts);
assert_eq!(d.probability(b"AB"), 0.5); // 2 / 4
assert_eq!(d.probability(b"BA"), 0.5);
assert_eq!(d.probability(b"AA"), 0.0);
assert_eq!(d, ngram_probabilities(data, 2)?);
# Ok::<(), celandine::transforms::InvalidNgramLength>(())
```

`transforms::ngrams` returns an exact-size, double-ended, fused iterator of
borrowed `&[u8]` slices in positional order. `distribution::ngram_counts` returns
`NgramCounts`, a reusable tally; `distribution::ngram_probabilities` returns
`NgramDistribution`, which owns that tally and computes probabilities lazily.
Both count and probability iterators visit **only observed blocks**, sorted
lexicographically by their byte contents. They do not enumerate `256^n`
possible blocks. Their `ngram_len()` retains the requested length even when empty.

For `AAAA` at length two, all three overlapping occurrences are `AA`, so the
count is three and the probability is one. Counting non-overlapping blocks would
give two occurrences: that is a different transformation.

## Domain, edge cases, and limits

| Input or query | Behavior |
| --- | --- |
| `n = 0`, including empty input | All constructors return `InvalidNgramLength`; no panic. |
| Empty input with positive `n` | Empty iterator, zero counts, empty empirical state. |
| `n = 1` | One block per byte; frequencies agree with `ByteHistogram`. |
| `n = L > 0` | One occurrence of the entire input, probability one. |
| `n > L`, including `usize::MAX` | No complete blocks, with no allocation or length arithmetic overflow. |
| Absent or wrong-length query | Count zero and probability positive `0.0`. |
| Binary or UTF-8 bytes | Bytes are used unchanged, even across UTF-8 character boundaries. |

There is no padding, wrapping, case conversion, tokenization, or smoothing.
`InvalidNgramLength` makes zero length an explicit error rather than choosing
an empty-word counting convention. A zero-observation state has no empirical
probability law: probability iteration is empty and individual queries return
zero by convention. Check `is_empty()` or `sample_size()` to identify that state.

Extraction and counts are exact; `f64` probabilities are rounded evaluations of
exact rational frequencies. Inferring source probabilities from them is an
estimation problem: overlapping observations may be dependent, and unobserved
blocks are not proof of impossible source events. Histograms discard positions
after counting; they do not reconstruct the original sequence in general.

The existing entropy APIs accept single-byte `Distribution`, not
`NgramDistribution`. No new entropy metrics are introduced here.

## Ownership, complexity, and numerical behavior

The iterator borrows the source: construction and each step take `O(1)` time
and storage, without allocation or copying bytes. Traversing all `m` blocks
takes `O(m)` iterator work, plus whatever the caller does with each slice.

Counts use a `BTreeMap` whose keys borrow source slices. The map compares bytes
by value, so repeated contents share one entry. Building counts takes
`O(m * n * log(k+1))` worst-case time: a tree lookup makes logarithmically many
comparisons, each of which can inspect `n` bytes. Short differing prefixes often
reduce actual comparison work. It uses `O(k)` additional tree storage, with no
copied block contents. Counts and distributions must not outlive the input, and
their borrows prevent source mutation while in use. Cloning a table allocates
new tree storage while retaining borrowed keys.

Count/probability queries take `O(n * log(k+1))` worst-case time and no allocation;
wrong-length queries return in constant time. Metadata queries take constant
time. Iterating a table takes `O(k)` total time and no allocation. Moving counts
into a distribution takes constant time without allocation or recounting.
Empty tables allocate no tree nodes. Nonempty count construction allocates;
memory exhaustion follows Rust's standard allocator behavior.

Every count is bounded by `m <= L`, so counts cannot overflow for a valid byte
slice. No alphabet size `256^n` is computed. Integer counts remain exact; the
conversion to `f64` can round integers above `2^53`. Probability sums can differ
slightly from one and are never silently normalized. Tests use `1e-12` absolute
tolerance on their bounded fixtures; this is not a universal error bound for
arbitrarily large support. Sorted iteration gives deterministic output.

## Verification and references

Tests include hand-calculated cases, exhaustive short binary sequences,
unigram agreement, reversal and bijective byte-relabeling properties, independent
Python fixtures using exact `Fraction` probabilities, and allocation checks.
Run the public-API demonstration with `cargo run --locked --example ngrams`.
Performance methodology and retained results are in `docs/benchmarks/ngrams.md`.

- Christopher D. Manning, Prabhakar Raghavan, and Hinrich Schütze (2008),
  *Introduction to Information Retrieval*, Cambridge University Press, section
  3.2.2. [Author-hosted text](https://nlp.stanford.edu/IR-book/html/htmledition/k-gram-indexes-for-wildcard-queries-1.html).
- Rust standard library: [slice windows](https://doc.rust-lang.org/std/primitive.slice.html#method.windows)
  and [BTreeMap](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html).
  These establish iterator and storage behavior; zero-length rejection and
  empty empirical states are Celandine conventions.
