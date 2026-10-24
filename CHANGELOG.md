# Changelog

## Unreleased — entropy, distance, sequence, and compression foundations

- Add `transforms::blocks(data, size)` for complete adjacent byte slices with
  an inspectable original remainder and fallible zero-size validation. Reuse
  Rust's exact-chunk iterator without copying bytes or allocating.
- Add independent partition fixtures, stateful/reconstruction tests, a runnable
  example, fuzzing, and measured latency/heap/RSS evidence.

- Add allocation-free `transforms::windows(data, size, step)` with explicit zero
  parameter errors, omitted partial tails, exact length, fused forward/reverse
  iteration, and constant-time skips even at `usize::MAX`.
- Add positional fixtures, stateful properties, extreme-skip regression tests,
  local-entropy example, dedicated fuzzing, and latency/heap/RSS measurements.

- Add ordered `normalized_compression_distance`, inspectable compressed sizes,
  and allocation-free normalization of components. Preserve signed differences
  before float conversion; retain nonmetric results and accept empty inputs.
- Report backend, zero-denominator, and fallible concatenation errors. Add exact
  rational fixtures, independent pair-stream decoding, allocation checks, a
  runnable example, bounded fuzzing, and paired latency/memory measurements.

- Add a dependency-free `Compressor` interface, complete `compressed_length` in
  bytes, and dimensionless `compression_ratio` with an explicit empty-input error.
- Add optional `compression-deflate`: pinned miniz_oxide 0.9.1, fixed level-6
  zlib streams, fresh state, no dictionary, and counted/discarded output.
- Retain framing/checksum overhead and propagate backend errors. Add independent
  C-zlib decoding, properties, an example, feature-matrix CI, fuzz coverage,
  and measured latency and memory evidence.


- Add `lz76_complexity` for exhaustive 1976 phrase counts over bytes, including
  overlapping reproduction and a counted final reproducible suffix.
- Define empty/single-byte behavior and integer units, with constant auxiliary
  space, no heap allocation, and explicit quadratic worst-case runtime limits.
- Add worked/published parses, independent histories, a minimum-production
  oracle, properties, a runnable example, fuzzing, and measured performance.

- Add exact subword complexity for contiguous byte blocks and a lazy profile
  covering lengths zero through the input length, including the empty word.
- Reuse n-gram counts for interior lengths, with allocation-free fast paths for
  zero, one, full, and oversized block lengths; document finite-profile costs.
- Add independent exact profiles, exhaustive/property tests, allocation checks,
  a runnable example, bounded fuzzing, and measured latency/memory baselines.

- Add equal-weight, base-2 Jensen–Shannon divergence and its separately named
  square-root distance, for byte slices and reusable distributions.
- Preserve tiny count differences with exact cross products and stable per-symbol
  evaluation; reject empty laws and handle zero counts without smoothing.
- Reuse `EmptyDistribution` across comparisons, generalizing its display text
  while preserving its public path and total variation's calculation.
- Add high-precision references, metric properties, a runnable example, fuzzing,
  allocation checks, and measured latency/memory baselines for both APIs.

- Add allocation-free total variation distance for byte samples and reusable
  distributions, with explicit errors for either empty input and support for
  unequal sample sizes.
- Use exact integer count comparisons before floating conversion to preserve
  tiny differences, proportional-law zeros, and disjoint-support ones.
- Add 148 independent rational fixtures, metric/event property tests, an example,
  dedicated fuzz coverage, and measured paired-input and memory benchmarks.
- Prioritize breadth across categories at the owner's request; defer joint
  distributions and additional entropy measures.

- Extend all entropy byte benchmark harnesses to 100 MiB and add a repeatable
  10/100 MiB comparison run that preserves historical baselines.
- Add an isolated, self-checked memory probe for peak requested heap allocations
  and fresh-process RSS, including n-gram support growth and distribution reuse.
- Add a small CI check for memory tooling and document measurement units,
  input residency, allocator overhead, and the limits of bounded workloads.

- Add three dedicated libFuzzer targets with independent count/block oracles,
  entropy invariants, and arbitrary order/base bit patterns, including invalid
  parameters and overflowing imported counts.
- Add 55 reproducible seed cases, stable seed replay, a pinned ASan campaign
  runner with retained logs/artifacts, and bounded fuzz campaigns in CI.
- Keep fuzz tooling in a separate development workspace and lockfile; preserve
  the public API, runtime dependency set, and Rust 1.90 support baseline.

- Adopt the MIT license and add crate repository, README, and license metadata.
- Declare Rust 1.90 as the supported minimum; document Linux x86-64 runtime
  support, compile-only target evidence, and numerical/API compatibility policy.
- Record the v0.1 readiness review, validation evidence, and remaining scope,
  dedicated fuzzing, large-input, and peak-memory release gates.

- Add allocation-free `shannon_with_base` and `shannon_distribution_with_base`
  with finite bases greater than one and explicit `InvalidLogBase` errors.
- Preserve default bit-valued APIs and document unit conversion, validation,
  near-one error amplification, and inherited extreme-count accuracy limits.
- Add 144 independent 120-digit reference fixtures, unit-conversion properties,
  allocation coverage, an updated Shannon example, and a measured benchmark
  suite with existing-API controls.

- Add borrowed, allocation-free overlapping byte n-gram extraction, exact count
  tables, and reusable empirical block distributions with sorted iteration.
- Reject zero lengths explicitly; define empty/oversized behavior, overlap,
  byte semantics, borrowing, memory costs, and probability units.
- Add educational docs and an example, 433 independently generated exact-rational
  fixtures, exhaustive short binary checks, property tests, allocation checks,
  and a measured n-gram benchmark baseline.

- Add allocation-free Tsallis entropy for bytes and reusable distributions with
  finite nonnegative orders and explicit invalid-order errors.
- Fix the scale to `1/ln(2)` so order one agrees with base-2 Shannon; document
  support at zero, different bounds, history, interpretation, and composition.
- Fix the near-zero accuracy issue found during review with direct powers for
  non-dominant symbols below order 0.5, retaining 60 independent regression cases.
- Use compensated `expm1` evaluation and exact count complements for stability
  near one and extreme imbalance, with positive subnormal large-order results.
- Add 532 independent high-precision fixtures, canonical/property tests,
  allocation checks, an explained example, and an 81-case benchmark run, retaining
  the original 77-case baseline.

- Add dedicated allocation-free min-entropy for bytes and reusable distributions,
  sharing the stable Rényi infinite-order calculation.
- Explain dominant-symbol surprise, the infinite-order limit, history, formula,
  edge cases, and empirical interpretation in docs and a runnable example.
- Add 48 independent high-precision fixtures, canonical/property tests, extreme
  count checks, allocation coverage, and a dedicated min-entropy benchmark baseline.

- Add dedicated allocation-free collision entropy for bytes and reusable
  distributions, sharing the validated Rényi order-two calculation.
- Explain matching draws with replacement, the formula, historical context,
  edge cases, and estimation limits in docs and a runnable comparison example.
- Add exact-rational reference fixtures, pair-count and extreme-count properties,
  allocation coverage, and a dedicated collision benchmark baseline.

- Add GitHub Actions CI for formatting, linting, builds, debug/release tests,
  reference fixtures, documentation, runnable examples, and benchmark compilation.
  Document its purpose, toolchain, automatic discovery, and validation limits.

- Add allocation-free Rényi entropy for byte slices and reusable distributions,
  accepting nonnegative orders and positive infinity, with explicit errors for
  negative/NaN orders. Reuse Hartley at zero and Shannon at one.
- Stabilize calculations near order one and for very large orders; retain small
  count complements near probability one without changing existing Shannon.
- Add educational documentation, a comparison example, 345 independent Decimal
  fixtures, canonical/property tests, allocation checks, and a benchmark baseline.

- Rename the project and Rust crate to `celandine`, including imports,
  documentation, examples, benchmarks, and the project plan.
- Add concise educational explanations for counts, empirical distributions,
  Shannon, and Hartley, including metric history, formula walkthroughs, and
  example introductions. Require this documentation for future functionality.
- Add allocation-free Hartley entropy for byte slices and reusable distributions,
  using exact observed support and the existing empty-input convention.
- Add Hartley metric/reference documentation, all-support independent fixtures,
  property tests, allocation checks, a comparison example, and a benchmark suite.

- Add fixed byte histograms with checked construction from existing counts.
- Add count-backed empirical distributions with lazy probabilities.
- Add allocation-free, base-2 empirical Shannon entropy for bytes and reusable
  distributions; define empty-input behavior explicitly.
- Document mathematical conventions, numerical behavior, and authoritative
  references.
- Add canonical and property tests, independent high-precision Python fixtures,
  an isolated allocation check, and Criterion latency/throughput benchmarks.
- Add a runnable Shannon example and a standing contributor/agent workflow for
  validating each iteration and future mathematical functionality.

This does not complete the full v0.1 roadmap in the project plan.
