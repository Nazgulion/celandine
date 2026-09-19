# Changelog

## Unreleased — empirical entropy foundation

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
