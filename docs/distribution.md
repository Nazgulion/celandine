# Byte counts and empirical distributions

The distribution layer turns observations into the counts and probabilities
used by the entropy functions. The byte values are symbols; no text encoding or
domain meaning is assumed. These are basic statistical summaries, not separate
entropy measures.

## Byte histogram: what it is and why it matters

`ByteHistogram` stores the exact number of occurrences of each of the 256 byte
values. Frequency tables summarize repeated observations; the fixed byte
alphabet lets this implementation use an array instead of a hash table.
One pass produces reusable counts with predictable storage and no heap allocation.

```text
c[b] = number of observations equal to byte b
n    = sum of all c[b]
k    = number of byte values whose c[b] is positive
```

`n` is the sample size and `k` is the observed support size. For `AAAB`,
`c[A] = 3`, `c[B] = 1`, `n = 4`, and `k = 2`; every other count is zero.
The alphabet size is still 256. These quantities answer different questions:
how much data was observed, which values occurred, and which values could be
represented by the input type.

`from_bytes` constructs the table; `count`, `counts`, `total`, and `support_size`
expose these summaries. `try_from_counts` imports an existing table and checks
its sum. `CountOverflow` reports a total larger than `usize::MAX`; rejecting it
prevents a wrapped total from corrupting later probability calculations.

## Empirical distribution: what it is and why it matters

`Distribution` interprets counts as relative frequencies. “Empirical” means
computed from the observations rather than supplied as a model of an unknown
source. For a nonempty sample:

```text
p[b] = c[b] / n
```

`p[b]` is the probability of seeing byte `b` when choosing one of the sample's
positions uniformly. Dividing by `n` converts a count to a fraction: in `AAAB`,
`p[A] = 3/4` and `p[B] = 1/4`. Doubling all counts preserves these fractions,
which allows comparing frequency balance across samples of different lengths.
Relative frequencies do not guarantee the probabilities of future observations.

The distribution owns one histogram and calculates probabilities when queried.
`sample_size`, `support_size`, and `histogram` retain access to exact observations;
`probability` and `probabilities` expose the derived `f64` values. This lets
Shannon use frequencies and Hartley use support without counting the input again.
Use `from_bytes` to count once, or `from_counts` to reuse an existing histogram.

## Worked example

```rust
use celandine::distribution::{ByteHistogram, Distribution};
use celandine::entropy::{hartley_entropy_distribution, shannon_distribution};

let counts = ByteHistogram::from_bytes(b"AAAB");
assert_eq!(counts.count(b'A'), 3);
assert_eq!(counts.total(), 4);
assert_eq!(counts.support_size(), 2);

let d = Distribution::from_counts(counts);
assert_eq!(d.probability(b'A'), 0.75);
assert_eq!(d.probability(b'B'), 0.25);
assert!((shannon_distribution(&d) - 0.811_278_124_459_132_8).abs() < 1e-12);
assert_eq!(hartley_entropy_distribution(&d), 1.0);
```

## Limits and conventions

Counts discard order: `AAAB` and `ABAA` produce the same histogram. This summary
is enough for the implemented single-symbol entropies, but does not represent
dependence between positions.

An empty sample has no empirical probability law. The API exposes an empty
state with zero counts, zero support, and zero probability queries; the implemented
entropies return zero by convention (Rényi requires a valid order). `is_empty` distinguishes this from a
nonempty constant sample. Arbitrary probability vectors are not accepted or
silently normalized. Counts are exact; conversion to `f64` and division can
round. Importing counts rejects integer overflow.

For the underlying probability and information concepts, see Cover and Thomas
(2006), *Elements of Information Theory*, second edition, chapter 2, and MacKay
(2003), *Information Theory, Inference, and Learning Algorithms*, chapter 2.
The storage layout, import checks, and empty-state behavior are project design
choices rather than historical definitions.
