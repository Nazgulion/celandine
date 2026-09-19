# celandine

<p align="center">
  <img src="pictures/celandine_001.png" alt="Yellow celandine flower with green leaves" width="240">
</p>

Entropy, information, complexity, and distance for finite sequences.

The library implements **empirical Shannon, Hartley, and Rényi entropy over
bytes** and their distribution foundation. It is one Rust library crate, with no runtime
dependencies, no heap allocations on the byte path, and no unsafe library code.
The broader project plan describes future work, not currently available APIs.

## What is implemented and why

| Component | What it does | Why it matters |
| --- | --- | --- |
| [Byte histogram](docs/distribution.md#byte-histogram-what-it-is-and-why-it-matters) | Counts occurrences of each byte exactly. | Gives metrics a reusable foundation with fixed storage. |
| [Empirical distribution](docs/distribution.md#empirical-distribution-what-it-is-and-why-it-matters) | Divides counts by sample size to obtain observed probabilities. | Makes frequency balance comparable and lets metrics share one histogram. |
| [Shannon entropy](docs/entropy/shannon.md) | Averages symbol surprise, weighted by frequency. | Distinguishes balanced from uneven frequencies on the same support. |
| [Hartley entropy](docs/entropy/hartley.md) | Takes the logarithm of the observed support size. | Measures the number of possibilities and bounds Shannon entropy from above. |
| [Rényi entropy](docs/entropy/renyi.md) | Varies the emphasis on rare versus frequent symbols using an order parameter. | Shows how diversity changes across orders, unifying Hartley and Shannon. |
| [Validation workflow](CONTRIBUTING.md) | Checks known answers, properties, independent references, and allocations. | Detects mathematical mistakes and regressions. |
| [Continuous integration](.github/workflows/ci.yml) | Runs validation on pushes and pull requests. | Makes regressions visible automatically as the project grows. |
| [Benchmarks](docs/benchmarks.md) | Measure latency and throughput on fixed workloads. | Establish evidence for performance changes. |

Start with [counts and probabilities](docs/distribution.md), then read the
Shannon, Hartley, and Rényi pages. Each metric page includes a short history,
explains the formula's symbols, and works through a small example before the references.

## Public API

```rust
use celandine::distribution::{ByteHistogram, Distribution};
use celandine::entropy::{
    hartley_entropy, hartley_entropy_distribution, renyi_entropy,
    renyi_entropy_distribution, shannon, shannon_distribution,
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
```

Use `Distribution::from_bytes(data)` when a separate histogram step is not
needed. `ByteHistogram::try_from_counts([usize; 256])` accepts existing counts
and rejects total overflow. Probability iteration returns all 256 entries in
byte order. Arbitrary floating-point probability vectors are not yet supported.

Shannon uses `f64` and base-2 logarithms. Empty input is an explicit empty
empirical state with entropy zero by convention. The result measures symbol
frequencies and ignores order. It is exact for the empirical law in mathematical
terms, with floating-point rounding; inference about an unknown source remains
an estimation problem.

Hartley uses the same units and empty-input convention, but measures only the
number of distinct observed bytes: `log2(support_size)`. Changing frequencies
while keeping support fixed leaves Hartley unchanged. It upper-bounds empirical
Shannon entropy and does not infer the full support of an unknown source.

Rényi uses an order parameter: order 0 is Hartley, order 1 is Shannon, and
positive infinity measures the surprise of the most frequent symbol. Entropy is
nonincreasing in order. Both APIs return `Result<f64, InvalidRenyiOrder>`:
negative or NaN orders are rejected even for empty input. Near-one and very
large orders use stable evaluation formulas without heap allocations.

## Try it on sample data

```sh
cargo run --locked --example shannon
cargo run --locked --example hartley
cargo run --locked --example renyi
```

Edit the samples in [examples/shannon.rs](examples/shannon.rs) and rerun to see
the results. For example, `AAAA` gives `0.000000`, `ABAB` gives `1.000000`, and
`ABCD` gives `2.000000` bits per symbol. This is a runnable library example;
the automated tests verify correctness independently of manual inspection.

The [Hartley comparison example](examples/hartley.rs) counts each sample once
and displays both entropies. For `AAAB` it prints Shannon `0.811278` and Hartley
`1.000000`; for `AB` both are `1.000000` bits per symbol. Edit its samples to
explore how frequencies affect the two measures.

The [Rényi comparison example](examples/renyi.rs) reuses one histogram to show
orders 0, 0.5, 1, 2, and infinity. For `AAAB`, results decrease from 1.000000 at
zero to 0.678072 at two and 0.415037 at infinity. Edit its samples and orders to
explore the effect of frequency balance.

## Documentation

- [Mathematical conventions](docs/mathematical-conventions.md)
- [Byte counts and empirical probabilities explained](docs/distribution.md)
- [Shannon definition and interpretation](docs/entropy/shannon.md)
- [Hartley definition and interpretation](docs/entropy/hartley.md)
- [Rényi definition and interpretation](docs/entropy/renyi.md)
- [Numerical behavior](docs/numerical-behavior.md)
- [Bibliography](docs/references.md), [Shannon reference notes](docs/references/shannon.md),
  [Hartley reference notes](docs/references/hartley.md),
  and [Rényi reference notes](docs/references/renyi.md)
- [Shannon benchmark baseline](docs/benchmarks.md), [Hartley baseline](docs/benchmarks/hartley.md),
  and [Rényi baseline](docs/benchmarks/renyi.md)
- [Authoritative project plan](finite_sequence_information_complexity_project_plan.md)

## Development

Follow the [development and validation workflow](CONTRIBUTING.md) for every
iteration. It defines the standing requirements for new functionality, manual
examples, regression tests, independent references, and performance checks.

[Continuous integration](CONTRIBUTING.md#continuous-integration) runs the checks
on Ubuntu with Rust 1.90.0 and Python 3.12, including debug/release tests,
independent references, examples, and benchmark compilation. Timing baselines
remain separate local measurements.

Validated with Rust 1.90.0, edition 2024. A minimum supported Rust version has
not yet been established. Python 3.10+ is used only to regenerate/check reference
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
python3 scripts/reference_hartley.py --check
python3 scripts/reference_renyi.py --check
cargo bench --locked --bench shannon
cargo bench --locked --bench hartley
cargo bench --locked --bench renyi
```

Property tests use a fixed seed and 256 cases per property for reproducible
baseline validation. Regression cases produced by proptest should be retained.
The independent Python calculation uses exact rational counts and 80-digit
decimal logarithms for Shannon. Hartley independently counts sets and checks
all 257 possible byte-support sizes with 80-digit logarithms. Rust tests compare
with the committed fixtures to `1e-12` bits absolute tolerance. Rényi adds 345
fixtures from 120-digit direct calculations (with bounded limiting values for
the largest orders), including adjacent orders around one and extreme counts.

This is a foundation milestone, not the complete v0.1 roadmap. Publishing is
disabled until licensing, the supported toolchain policy, and the public API
have been reviewed. Dedicated collision/min-entropy APIs, other entropy families,
a CLI, and bindings remain future work.
