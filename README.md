# celandine

<p align="center">
  <img src="pictures/celandine_001.png" alt="Yellow celandine flower with green leaves" width="240">
</p>

Entropy, information, complexity, and distance for finite sequences.

The library implements **empirical Shannon, Hartley, Rényi, collision, min-entropy,
and Tsallis entropy over bytes**, plus **overlapping byte n-grams, counts, and
empirical block probabilities**, and **total variation distance between byte
distributions**, plus **Jensen–Shannon divergence and distance**.
It is one Rust library crate, with no runtime dependencies and
no unsafe library code. Single-byte entropy, implemented distribution comparisons, and n-gram
extraction allocate no heap memory; n-gram count tables allocate storage for observed blocks.
The broader project plan describes future work, not currently available APIs.

## What is implemented and why

| Component | What it does | Why it matters |
| --- | --- | --- |
| [Byte histogram](docs/distribution.md#byte-histogram-what-it-is-and-why-it-matters) | Counts occurrences of each byte exactly. | Gives metrics a reusable foundation with fixed storage. |
| [Empirical distribution](docs/distribution.md#empirical-distribution-what-it-is-and-why-it-matters) | Divides counts by sample size to obtain observed probabilities. | Makes frequency balance comparable and lets metrics share one histogram. |
| [Overlapping n-grams](docs/ngrams.md) | Extracts every complete block of a chosen byte length, counts repeated blocks, and derives their probabilities. | Reveals local patterns that single-byte frequencies discard. |
| [Shannon entropy](docs/entropy/shannon.md) | Averages symbol surprise, weighted by frequency, in bits or explicitly selected units. | Distinguishes balanced from uneven frequencies on the same support. |
| [Hartley entropy](docs/entropy/hartley.md) | Takes the logarithm of the observed support size. | Measures the number of possibilities and bounds Shannon entropy from above. |
| [Rényi entropy](docs/entropy/renyi.md) | Varies the emphasis on rare versus frequent symbols using an order parameter. | Shows how diversity changes across orders, unifying Hartley and Shannon. |
| [Collision entropy](docs/entropy/collision.md) | Takes the negative logarithm of the probability that two independent draws match. | Measures concentration and gives Rényi order two a dedicated API. |
| [Min-entropy](docs/entropy/min_entropy.md) | Takes the negative logarithm of the largest probability. | Isolates symbol dominance and gives the infinite-order limit a dedicated API. |
| [Tsallis entropy](docs/entropy/tsallis.md) | Measures diversity through probability powers with a fixed Shannon-bit scale. | Provides an order-dependent measure with a different composition rule and the Shannon limit at one. |
| [Total variation distance](docs/divergence/total_variation.md) | Compares byte relative frequencies across two nonempty samples. | Measures composition differences independently of sample length and ordering. |
| [Jensen–Shannon divergence](docs/divergence/jensen_shannon.md) | Compares two laws to their equal mixture, in bits. | Quantifies the mixture’s extra ideal coding cost, with finite results for disjoint supports. |
| [Jensen–Shannon distance](docs/divergence/jensen_shannon_distance.md) | Takes the square root of Jensen–Shannon divergence. | Supplies a metric on probability laws, including the triangle inequality. |
| [Validation workflow](CONTRIBUTING.md) | Checks known answers, properties, independent references, and allocations. | Detects mathematical mistakes and regressions. |
| [Dedicated fuzzing](docs/fuzzing.md) | Mutates byte inputs, counts, parameters, and block lengths with independent checks. | Finds unusual combinations and retains failures for regression testing. |
| [Continuous integration](.github/workflows/ci.yml) | Runs validation on pushes and pull requests. | Makes regressions visible automatically as the project grows. |
| [Benchmarks](docs/benchmarks.md) | Measure latency and throughput on fixed workloads. | Establish evidence for performance changes. |

Start with [counts and probabilities](docs/distribution.md), then read the
Shannon, Hartley, Rényi, collision, min-entropy, and Tsallis pages. Each metric page
includes a short history, explains the formula's symbols, and works through a small example
before the references.

## Public API

```rust
use celandine::distribution::{ByteHistogram, Distribution};
use celandine::entropy::{
    collision_entropy, collision_entropy_distribution,
    hartley_entropy, hartley_entropy_distribution, min_entropy,
    min_entropy_distribution, renyi_entropy,
    renyi_entropy_distribution, shannon, shannon_distribution,
    tsallis_entropy, tsallis_entropy_distribution,
};

assert_eq!(shannon(b"ABCD"), 2.0); // bits per symbol
assert_eq!(shannon(b"AAAA"), 0.0);
assert_eq!(shannon(b""), 0.0);     // explicit empty-input convention
assert_eq!(hartley_entropy(b"AAAB"), 1.0); // two observed symbols

let histogram = ByteHistogram::from_bytes(b"ABAB");
assert_eq!(histogram.count(b'A'), 2);
let distribution = Distribution::from_counts(histogram);
assert_eq!(distribution.sample_size(), 4);
assert_eq!(distribution.support_size(), 2);
assert_eq!(distribution.probability(b'A'), 0.5);
assert_eq!(shannon_distribution(&distribution), 1.0);
assert_eq!(hartley_entropy_distribution(&distribution), 1.0);
assert_eq!(renyi_entropy_distribution(&distribution, 2.0), Ok(1.0));
assert!((renyi_entropy(b"AAAB", 2.0).unwrap() - 0.6780719051126377).abs() < 1e-12);
assert!(renyi_entropy(b"AB", -1.0).is_err());
assert_eq!(collision_entropy_distribution(&distribution), 1.0);
assert!((collision_entropy(b"AAAB") - 0.6780719051126377).abs() < 1e-12);
assert_eq!(min_entropy_distribution(&distribution), 1.0);
assert!((min_entropy(b"AAAB") - 0.4150374992788438).abs() < 1e-12);
assert_eq!(tsallis_entropy_distribution(&distribution, 1.0), Ok(1.0));
assert!((tsallis_entropy(b"AAAB", 2.0).unwrap() - 0.375 / std::f64::consts::LN_2).abs() < 1e-12);
```

Use `Distribution::from_bytes(data)` when a separate histogram step is not
needed. `ByteHistogram::try_from_counts([usize; 256])` accepts existing counts
and rejects total overflow. Probability iteration returns all 256 entries in
byte order. Arbitrary floating-point probability vectors are not yet supported.

Shannon uses `f64` and defaults to base-2 logarithms. Empty input is an explicit
empty empirical state with entropy zero by convention. The result measures symbol
frequencies and ignores order. It is exact for the empirical law in mathematical
terms, with floating-point rounding; inference about an unknown source remains
an estimation problem.

The explicit-base APIs select units without recounting an existing distribution:

```rust
use celandine::distribution::Distribution;
use celandine::entropy::{shannon_with_base, shannon_distribution_with_base};

assert_eq!(shannon_with_base(b"ABCD", 2.0), Ok(2.0)); // bits per symbol
let d = Distribution::from_bytes(b"ABCD");
let nats = shannon_distribution_with_base(&d, std::f64::consts::E).unwrap();
assert!((nats - 1.3862943611198906).abs() < 1e-12);
assert!(shannon_with_base(b"", 1.0).is_err());
```

They compute `H_b = H_2 / log2(b)`, where `H_2` is entropy in bits and `b` is
finite and greater than one. Base `e` gives nats and base 10 gives decimal
information units, all per symbol. Invalid bases return `InvalidLogBase`, even
for empty input; valid empty/constant inputs return positive zero. These paths
allocate no heap memory. Values near base one can be large and amplify absolute
rounding error. See [the formula, history, worked example, and numerical
limits](docs/entropy/shannon_base.md).

Hartley uses the same units and empty-input convention as default Shannon, but
measures only the number of distinct observed bytes: `log2(support_size)`. Changing frequencies
while keeping support fixed leaves Hartley unchanged. It upper-bounds empirical
Shannon entropy and does not infer the full support of an unknown source.

Rényi uses an order parameter: order 0 is Hartley, order 1 is Shannon, and
positive infinity measures the surprise of the most frequent symbol. Entropy is
nonincreasing in order. Both APIs return `Result<f64, InvalidRenyiOrder>`:
negative or NaN orders are rejected even for empty input. Near-one and very
large orders use stable evaluation formulas without heap allocations.

Collision entropy is Rényi order two with a direct `f64` API:
`H2 = -log2(sum(p_i^2))`. The sum is the probability that two independent draws
with replacement give the same symbol. `AAAB` has matching probability `10/16`
and entropy `0.678072` bits per symbol. It measures frequency concentration and
ignores position; it does not count adjacent matches or assume the observed
sequence was generated independently.

Min-entropy is the infinite-order Rényi limit with a direct `f64` API:
`H_inf = -log2(p_max)`. It measures the surprise of the most likely symbol.
For `AAAB`, the largest probability is `3/4` and min-entropy is `0.415037` bits
per symbol. Keeping that maximum probability fixed leaves min-entropy unchanged,
even if the remaining frequencies change. It does not infer a source's behavior.

Tsallis uses finite `q>=0` and fixed **Shannon-bit scaling**:
`S_q = (1-sum(p_i^q))/(q-1)/ln(2)`. At `q=1` it returns Shannon; near one it
keeps the supplied order using stable evaluation. `AAAB` gives `0.541011` at
order two. Tsallis can exceed 8 in this scale and is not generally an average
code length. Both APIs return `Result<f64, InvalidTsallisOrder>`; negative and
nonfinite orders are rejected even for empty input.

### Comparing distributions

```rust
use celandine::distribution::Distribution;
use celandine::divergence::{total_variation_distance, total_variation_distance_distribution};

assert_eq!(total_variation_distance(b"AAAB", b"AB"), Ok(0.25));
let p = Distribution::from_bytes(b"AABB");
let q = Distribution::from_bytes(b"ABAB");
assert_eq!(total_variation_distance_distribution(&p, &q), Ok(0.0));
assert!(total_variation_distance(b"", b"AB").is_err());
```

Total variation is `TV(P,Q) = sum_i |p_i-q_i| / 2`, where `p_i` and `q_i` are
relative frequencies of byte `i`. It is dimensionless in `[0,1]`: zero for equal
laws, one for disjoint supports. `AAAB` versus `AB` gives `(1/4+1/4)/2 = 0.25`.
Sample lengths may differ. Either empty sample returns `EmptyDistribution`,
including two empty samples, because no empirical probability law exists.
Exact count arithmetic preserves small differences before the final `f64`
conversion. See the [definition, history, and limits](docs/divergence/total_variation.md).

Jensen–Shannon uses the equal mixture `M=(P+Q)/2`, with
`J(P,Q)=H(M)-(H(P)+H(Q))/2` in bits. Each law has weight one half even when sample
lengths differ. `A` versus `AB` gives `J=0.811278-0.5=0.311278`; the separately
named distance is `sqrt(J)=0.557923`. Both have mathematical range `[0,1]`, and
both reject either empty input. The distance uses square-root-bit units.

```rust
use celandine::divergence::{jensen_shannon_divergence, jensen_shannon_distance};
assert_eq!(jensen_shannon_divergence(b"A", b"B"), Ok(1.0));
assert_eq!(jensen_shannon_distance(b"AABB", b"ABAB"), Ok(0.0));
```

The corresponding `_distribution` APIs reuse count tables. See the
[divergence](docs/divergence/jensen_shannon.md) and
[distance](docs/divergence/jensen_shannon_distance.md) guides for references,
worked examples, and the stable evaluation used for nearly identical laws.

### N-gram foundation

```rust
use celandine::transforms::ngrams;
use celandine::distribution::{ngram_counts, NgramDistribution};

let blocks: Vec<_> = ngrams(b"ABABA", 2).unwrap().collect();
assert_eq!(blocks, vec![b"AB", b"BA", b"AB", b"BA"]);
let counts = ngram_counts(b"ABABA", 2).unwrap();
assert_eq!(counts.total(), 4);
assert_eq!(counts.count(b"AB"), 2);
let d = NgramDistribution::from_counts(counts);
assert_eq!(d.probability(b"AB"), 0.5); // 2 of 4 block occurrences
```

`distribution::ngram_probabilities(data, n)` constructs a distribution directly.
For input length `L` and block length `n`, the number of occurrences is `m = L-n+1`
when `1 <= n <= L`, otherwise zero for valid `n`. A block's dimensionless
probability is its count divided by `m`. Length zero returns `InvalidNgramLength`;
empty results have no empirical law and probability queries return zero by convention.
Blocks overlap, borrow their source bytes, and use no padding or text decoding.
Count and probability tables iterate over observed blocks in lexicographic byte
order. These distributions describe observed local patterns; the current entropy
APIs still accept single-byte distributions. See the [definition, rationale,
worked example, and limits](docs/ngrams.md).

## Try it on sample data

```sh
cargo run --locked --example shannon
cargo run --locked --example hartley
cargo run --locked --example renyi
cargo run --locked --example collision
cargo run --locked --example min_entropy
cargo run --locked --example tsallis
cargo run --locked --example ngrams
cargo run --locked --example total_variation
cargo run --locked --example jensen_shannon
```

Edit the samples in [examples/shannon.rs](examples/shannon.rs) and rerun to see
the results. For example, `AAAA` gives `0.000000`, `ABAB` gives `1.000000`, and
`ABCD` gives `2.000000` bits per symbol. This is a runnable library example;
the automated tests verify correctness independently of manual inspection.
The Shannon example reuses each histogram to compare bits, nats, and decimal
information units per symbol; `ABCD` also gives `1.386294` nats and `0.602060`
decimal units per symbol.

The [Hartley comparison example](examples/hartley.rs) counts each sample once
and displays both entropies. For `AAAB` it prints Shannon `0.811278` and Hartley
`1.000000`; for `AB` both are `1.000000` bits per symbol. Edit its samples to
explore how frequencies affect the two measures.

The [Rényi comparison example](examples/renyi.rs) reuses one histogram to show
orders 0, 0.5, 1, 2, and infinity. For `AAAB`, results decrease from 1.000000 at
zero to 0.678072 at two and 0.415037 at infinity. Edit its samples and orders to
explore the effect of frequency balance.

The [collision example](examples/collision.rs) prints the matching probability
and compares collision, Shannon, and Hartley entropy. Empty input has no matching
probability; the example labels that explicitly while showing the zero-entropy
convention. Edit its samples and rerun to explore frequency concentration.

The [min-entropy example](examples/min_entropy.rs) prints the largest probability
and compares min-entropy, collision, Shannon, and Hartley using one histogram.
Its final two samples share the same dominant probability but different remaining
frequencies, illustrating what min-entropy captures and what it ignores.

The [Tsallis example](examples/tsallis.rs) shows orders 0, 0.5, 1, 2, and 4,
explains the scaling, and prints additional values just below and above one.
For uniform `AB`, it prints `1.442695`, `1.000000`, and `0.721348` at orders
zero, one, and two: uniformity does not make Tsallis constant across orders.

The [n-gram example](examples/ngrams.rs) prints overlapping blocks, their counts,
and probabilities. `ABABA` at length two has four occurrences and two distinct
blocks, each with probability `0.5`. It also contrasts `AABB` with `ABAB`, handles
binary data, and explains empty results and zero-length rejection.

The [total variation example](examples/total_variation.rs) compares unequal sample
lengths, identical composition with different ordering, disjoint supports, binary
data, and empty-input errors.

The [Jensen–Shannon example](examples/jensen_shannon.rs) shows both outputs,
equal mixture weights with unequal sample lengths, binary data, and empty errors.

## Documentation

- [Mathematical conventions](docs/mathematical-conventions.md)
- [Byte counts and empirical probabilities explained](docs/distribution.md)
- [Overlapping n-grams, counts, and probabilities](docs/ngrams.md)
- [Shannon definition and interpretation](docs/entropy/shannon.md)
- [Shannon logarithm bases and units](docs/entropy/shannon_base.md)
- [Hartley definition and interpretation](docs/entropy/hartley.md)
- [Rényi definition and interpretation](docs/entropy/renyi.md)
- [Collision definition and interpretation](docs/entropy/collision.md)
- [Min-entropy definition and interpretation](docs/entropy/min_entropy.md)
- [Tsallis definition and interpretation](docs/entropy/tsallis.md)
- [Total variation definition and interpretation](docs/divergence/total_variation.md)
- [Total variation reference notes](docs/references/total_variation.md)
- [Total variation performance and memory baseline](docs/benchmarks/total_variation.md)
- [Jensen–Shannon divergence](docs/divergence/jensen_shannon.md) and [distance](docs/divergence/jensen_shannon_distance.md)
- [Jensen–Shannon references](docs/references/jensen_shannon.md) and [benchmarks](docs/benchmarks/jensen_shannon.md)
- [Numerical behavior](docs/numerical-behavior.md)
- [Toolchain, platform, and compatibility policy](docs/support.md)
- [v0.1 readiness review](docs/releases/v0.1-readiness.md)
- [Fuzz targets, replay, and campaign workflow](docs/fuzzing.md)
- [Bibliography](docs/references.md), [Shannon reference notes](docs/references/shannon.md),
  [Hartley reference notes](docs/references/hartley.md),
  [Rényi reference notes](docs/references/renyi.md),
  [collision reference notes](docs/references/collision.md),
  [min-entropy reference notes](docs/references/min_entropy.md),
  and [Tsallis reference notes](docs/references/tsallis.md)
- [Shannon benchmark baseline](docs/benchmarks.md), [Hartley baseline](docs/benchmarks/hartley.md),
  [Rényi baseline](docs/benchmarks/renyi.md),
  [collision baseline](docs/benchmarks/collision.md),
  [min-entropy baseline](docs/benchmarks/min_entropy.md),
  and [Tsallis baseline](docs/benchmarks/tsallis.md)
- [N-gram baseline](docs/benchmarks/ngrams.md)
- [Shannon explicit-base baseline](docs/benchmarks/shannon_base.md)
- [100 MiB latency and peak-memory measurements](docs/benchmarks/large-input-memory.md)
- [Authoritative project plan](finite_sequence_information_complexity_project_plan.md)

## Development

Follow the [development and validation workflow](CONTRIBUTING.md) for every
iteration. It defines the standing requirements for new functionality, manual
examples, regression tests, independent references, and performance checks.

[Continuous integration](CONTRIBUTING.md#continuous-integration) runs the checks
on Ubuntu with Rust 1.90.0 and Python 3.12, including debug/release tests,
independent references, examples, and benchmark compilation. Timing baselines
remain separate local measurements. Dedicated fuzz seeds also replay on Rust
1.90.0; a separate pinned nightly job runs bounded AddressSanitizer campaigns.

Minimum supported Rust: **1.90**, edition 2024. The tested runtime target is
Linux x86-64 (`x86_64-unknown-linux-gnu`); see the [support policy](docs/support.md).
Python 3.10+ is used only to regenerate/check reference
fixtures; ordinary Rust tests need no Python interpreter. Criterion and proptest
are development dependencies only. The lockfile is retained for reproducibility.

```sh
cargo build --locked
cargo test --locked
cargo test --locked --release --test allocations
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo doc --locked --no-deps
python3 scripts/reference_shannon.py --check
python3 scripts/reference_shannon_base.py --check
python3 scripts/reference_hartley.py --check
python3 scripts/reference_renyi.py --check
python3 scripts/reference_collision.py --check
python3 scripts/reference_min_entropy.py --check
python3 scripts/reference_tsallis.py --check
python3 scripts/reference_ngrams.py --check
python3 scripts/reference_total_variation.py --check
python3 scripts/reference_jensen_shannon.py --check
cargo bench --locked --bench shannon
cargo bench --locked --bench shannon_base
cargo bench --locked --bench hartley
cargo bench --locked --bench renyi
cargo bench --locked --bench collision
cargo bench --locked --bench min_entropy
cargo bench --locked --bench tsallis
cargo bench --locked --bench ngrams
cargo bench --locked --bench total_variation
cargo bench --locked --bench jensen_shannon
```

Property tests use a fixed seed and 256 cases per property for reproducible
baseline validation. Regression cases produced by proptest should be retained.
The independent Python calculation uses exact rational counts and 80-digit
decimal logarithms for Shannon. Hartley independently counts sets and checks
all 257 possible byte-support sizes with 80-digit logarithms. Rust tests compare
with the default-unit fixtures to `1e-12` absolute tolerance in the documented units.
Rényi adds 345 fixtures from 120-digit direct calculations (with bounded limiting values for
the largest orders), including adjacent orders around one and extreme counts.
Collision adds 45 exact-rational reference fixtures and properties based on
explicit pair enumeration and exact integer complements at extreme counts.
Min-entropy adds 48 exact-rational reference fixtures, independent minimum-surprise
checks, and properties covering dominant symbols and extreme count tables.
Tsallis adds 532 reference fixtures, including bounded large-order references,
and properties for order monotonicity, differing pairs, and product composition.
N-grams add 433 exact-rational Python fixtures, exhaustive short binary inputs,
reversal/relabeling properties, and allocation checks for extraction and reuse.
Explicit Shannon bases add 144 independent 120-digit fixtures using exact `f64`
base values, invalid-base checks, unit-conversion properties, and allocation tests.
Their absolute error is checked at equivalent bit scale because values near
base one amplify absolute rounding error; ordinary fixtures also check relative
accuracy as described in the [numerical notes](docs/entropy/shannon_base.md).

Total variation adds 148 exact-rational fixtures, event-enumeration checks,
metric properties, and a dedicated fuzz target.

Jensen–Shannon adds 171 high-precision reference pairs, a dedicated fuzz target,
and separate divergence/distance benchmarks.

The six core entropy measures, n-gram primitives, total variation, and
Jensen–Shannon comparisons are implemented, including explicit Shannon logarithm bases. The [v0.1 readiness review](docs/releases/v0.1-readiness.md)
records passing validation and remaining release-scope decisions. Dedicated fuzz
campaigns, 100 MiB entropy benchmarks, and controlled peak-memory measurements
now supplement the property tests. Publishing stays disabled until the remaining
release gates are resolved.
Development now prioritizes breadth across categories, with total variation and
Jensen–Shannon comparisons as the first distribution-comparison increments. Joint distributions and additional entropies are deferred; other
metrics, a CLI, and bindings remain future work.

## License

Licensed under the [MIT License](LICENSE).
