# Jensen–Shannon distance

Jensen–Shannon distance is the square root of the equal-weight Jensen–Shannon
divergence: `d_JS(P,Q)=sqrt(J(P,Q))`. Here `P` and `Q` are empirical byte laws,
`J` is measured using base-2 logarithms in bits, and the distance uses
square-root-bit units with mathematical range `[0,1]`.

Taking the square root gives a metric on probability laws, including the
triangle inequality. This is useful when a comparison method needs metric
behavior. Divergence and distance are distinct outputs; neither distinguishes
sequences that have the same byte composition but different order.

## Historical context

Endres and Schindelin (2003) proved metric properties for the square root of an
entropy-based expression equal to twice the equal-weight Jensen–Shannon
divergence. Multiplication by a positive constant preserves the metric property,
so their result also supports the square-root convention used here.
[Original paper](https://pdodds.w3.uvm.edu/research/papers/others/2003/endres2003a.pdf).

## Worked example and API

For `A` and `AB`, the equal mixture has probabilities `(3/4,1/4)` and the
divergence is `0.31127812445913283` bits. Its square root is approximately
`0.5579230452841438`. Equal laws have distance zero; disjoint supports have one.

```rust
use celandine::divergence::jensen_shannon_distance;
let d = jensen_shannon_distance(b"A", b"AB").unwrap();
assert!((d - 0.5579230452841438).abs() < 1e-14);
assert_eq!(jensen_shannon_distance(b"AAA", b"B"), Ok(1.0));
assert!(jensen_shannon_distance(b"", b"").is_err());
```

## Conventions and limits

Inputs must both be nonempty; their lengths may differ. Either empty input,
including both, returns `EmptyDistribution`. Each law receives weight one half,
independently of its sample length. The distance API takes the square root of
the same divergence result, preserving its conventions, rounding limits, and
allocation behavior. It does not infer the distance between unknown sources.

`jensen_shannon_distance_distribution` reuses validated counts. The byte API
uses `O(len(x)+len(y)+256)` time, two fixed histograms, and no heap allocation;
the distribution API uses `O(256)` time and constant auxiliary storage. The
square root adds one floating operation. See `jensen_shannon_divergence` for
the mixture formula, zero-count handling, numerical evaluation, and references.
