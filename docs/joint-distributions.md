# Joint byte counts and empirical distributions

A joint table records which byte values occur together at matching positions in
two sequences. It preserves their pairing, which separate histograms discard.
It supplies reusable observations for joint entropy and later information measures.

For equal-length inputs x and y containing N bytes each:

```text
c[a,b] = number of positions i with x[i]=a and y[i]=b
p[a,b] = c[a,b]/N                      (N > 0)
c_x[a] = sum_b c[a,b]                  first marginal counts
c_y[b] = sum_a c[a,b]                  second marginal counts
```

Here a and b are byte values 0..255; counts and N are numbers of paired
observations, and probabilities are dimensionless fractions. Both marginals
have total N. Pairing is positional: no truncation, alignment search, Cartesian
product, or assumption of independence is applied.

## Worked example and use

For x=`AABB`, y=`XYXY`, observed pairs are AX, AY, BX, BY, each once.
N=4, support size=4, and every joint probability is 1/4. The first marginal
counts A:2, B:2; the second counts X:2, Y:2. Changing y to `XXYY` leaves these
marginals unchanged but gives AX:2 and BY:2. The joint table detects that change.

`joint_counts` returns `JointCounts`; `joint_probabilities` returns
`JointDistribution`. The latter takes ownership through `from_counts` when a
table already exists. `count(a,b)` and `probability(a,b)` query a pair;
`counts()` and `probabilities()` visit only observed pairs in lexicographic order.
`total()`/`sample_size()` count observations; `support_size()` counts distinct
pairs. `marginal_x()` and `marginal_y()` return the existing `ByteHistogram` or
`Distribution` types respectively, allowing single-byte metrics to reuse them.

Joint probabilities and marginalization are standard probability concepts.
Shannon's 1948 communication paper applies them to the entropy of joint events
in section 6, property 3; see the
[original paper](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf).
The sparse storage, byte specialization and empty convention are library choices.

## Domain, ownership and boundaries

Unequal lengths return `LengthMismatch { x_len, y_len }` before counting,
including when only one input is empty. Two empty slices produce zero counts,
zero support, zero probability queries and no iterator entries. This is an
explicit empty empirical state, not a probability law with total mass one.
A singleton produces one pair with probability one.

The table owns its pair keys and can outlive both input slices. Only positive
counts are stored. Private counts always sum exactly to N, bounded by either
input slice length, so construction and marginal sums cannot overflow usize.
No arbitrary counts or floating probability vectors are imported in this API.
The alphabet has 65,536 possible pairs regardless of observed support.

## Costs, numerical behavior and limits

With s distinct pairs, construction uses a deterministic ordered tree:
`O(N log(s+1))` time and `O(s)` heap space, with s<=min(N,65536).
Empty construction allocates nothing. Queries cost `O(log(s+1))`; iteration
costs O(s); total/support queries cost O(1). Cloning copies the tree in O(s)
time and space. Moving counts into a distribution does not allocate or recount.
Marginals sum integer counts in `O(s+256)` time with fixed 256-count storage
and no heap allocation. Repeated marginal queries recompute these sums.

Probabilities are derived as f64 and can round; no smoothing, normalization,
independence inference or bias correction is performed. Reordering both inputs
by the same permutation preserves the law, while reordering only one generally
does not. Counts discard observation order after pairing. The empirical law
describes this sample and does not establish the law of its generating source.
See also [joint entropy](entropy/joint.md) and [byte distributions](distribution.md).
