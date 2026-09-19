# Development and validation workflow

This is the project's standing workflow until explicitly changed. The
[project plan](finite_sequence_information_complexity_project_plan.md) remains
the authoritative specification. Work within the currently authorized milestone;
the roadmap does not authorize implementing later metrics automatically.

## Explore results

Run the small entropy examples:

```sh
cargo run --locked --example shannon
cargo run --locked --example hartley
cargo run --locked --example renyi
cargo run --locked --example collision
cargo run --locked --example min_entropy
```

Edit the byte samples in `examples/shannon.rs`, `examples/hartley.rs`,
`examples/renyi.rs`, `examples/collision.rs`, or `examples/min_entropy.rs` and rerun
to inspect other inputs. The comparison examples reuse one histogram per sample;
Rényi also lets you change the orders.
Output appears immediately in the terminal. There is no file watcher or live
interface; each run computes the current samples once. Display formatting is
only for readability: the entropy calculation operates on the original bytes.

Maintain small runnable examples as functionality is added. Examples should use
the public library API, identify units, and make representative results easy to
inspect. Keep them separate from core algorithms. Manual output inspection
complements automated verification; plausible-looking numbers are not evidence
of mathematical correctness.

## Check each implementation iteration

From the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
python3 scripts/reference_shannon.py --check
python3 scripts/reference_hartley.py --check
python3 scripts/reference_renyi.py --check
python3 scripts/reference_collision.py --check
python3 scripts/reference_min_entropy.py --check
```

`cargo test` checks canonical results, mathematical properties, independent
reference fixtures, documentation examples, and the isolated allocation test.
Clippy also checks runnable examples and benchmarks. Run any added or modified
example explicitly, since compiling an example does not check its output:

```sh
cargo run --locked --example shannon
```

For documentation changes, check relevant examples and build the API docs:

```sh
RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps
```

Scope checks to the change: prose-only edits do not require new mathematical
tests or a fresh performance baseline. Report what was checked and any failures
or checks that could not run.

## Continuous integration

[CI](.github/workflows/ci.yml) runs the validation workflow automatically on
pushes and pull requests. It also supports a manual run from the repository's
Actions tab after the workflow reaches the default branch. This makes failures
visible without relying on contributors to remember each local command.

The initial job uses Ubuntu 24.04, Rust 1.90.0 with rustfmt and Clippy, and Python
3.12. Rust is fixed to the toolchain already used for this milestone; this does
not establish a minimum supported Rust version or cross-platform support policy.
The checkout and Python setup actions are pinned to release commit hashes.

It checks formatting and all-target lints, builds the library, runs debug and
release tests (including doctests, properties, and isolated allocation checks),
verifies independent fixtures, builds documentation with warnings treated as
errors, runs examples, and compiles the benchmark harnesses. For example, a
change that makes `H_2(AAAB)` disagree with its reference fails the test step and
marks the CI run as failed. Future `scripts/reference_*.py` scripts and top-level
`examples/*.rs` examples are picked up automatically. Keep these conventions when
adding a metric; new benchmark targets must also be declared in `Cargo.toml`.

Benchmark timing remains a separate measured workflow: shared runner load makes
CI timings unsuitable as stable performance baselines. Continue running the
relevant benchmarks locally when algorithms change. A passing CI run also does
not replace the mathematical review required by the project plan.

The workflow needs no project secrets and has read-only repository permissions.
It reports check results; making them mandatory for merging requires a separate
repository ruleset or branch-protection setting. That setting is not changed by
adding the workflow. Inspect failed steps in the Actions tab and reproduce them
with the commands above; use `cargo bench --locked --no-run` to reproduce the
benchmark compilation check.

## Requirements for new functionality

For each new mathematical function, include:

- A short explanation following the educational documentation standard below.
- A precise definition, authoritative reference, units, input domain, output
  range, assumptions, and distinction between exact quantity and estimator.
- Explicit edge cases, validation behavior, numerical tolerance, and complexity.
- Hand-verifiable examples with independently derived expected answers.
- Property tests for applicable mathematical invariants, using reproducible
  seeds and retaining regression cases.
- Independent reference calculations; extend the fixture-generation/check
  workflow and document the commands for new metrics.
- A runnable public-API example with representative inputs and explained results.
- Benchmarks, and allocation checks where relevant to the function's contract.
- API documentation and a changelog entry.

When manual experimentation reveals a defect, add a regression test that fails
before the fix, then verify it passes afterward. Expected values must come from
the definition or an independent calculation, not be copied from the function
being tested. Do not add tests that merely restate implementation details.

## Educational documentation standard

Every implemented and future public concept must have a concise explanation
of what it does and why a reader would use it. Keep this beside its API/metric
documentation, and link it from the README's implemented-feature overview.
Follow the project plan's full documentation structure, adding these elements:

- **Definition:** one or two plain-language sentences describing the quantity
  or component before introducing specialist terminology.
- **Short history:** for named measures or algorithms, two or three sentences
  with the relevant author, date, original problem or contribution, and a
  primary/authoritative citation. For basic infrastructure, explain its
  statistical background or design rationale; do not invent an origin story.
- **Why it matters:** the concrete distinction it reveals or capability it
  enables, with the assumptions needed for that claim.
- **Formula explained:** define every symbol, index, operation, parameter, and
  unit needed to read the formula; explain in words what the calculation does.
  If there is no meaningful formula, describe the operation and invariants.
- **Worked example:** show a tiny input, intermediate counts/probabilities or
  other relevant steps, the result, and its interpretation. Use independently
  verified values consistent with the tests.
- **Limits:** distinguish what is measured from what cannot be inferred, and
  retain the existing edge-case, numerical, complexity, and reference sections.

Use the Shannon, Hartley, Rényi, collision, min-entropy, and distribution pages
as examples. Reuse the metric text in rustdoc where practical to avoid divergent
explanations. Add a brief
plain-language introduction and formula legend to runnable examples so terminal
output is understandable on its own. Keep full history and references in the
documentation. Update explanations when behavior changes; this requirement
applies until the user explicitly changes it.

## Measure algorithm and performance changes

For the current entropy implementations:

```sh
cargo bench --locked --bench shannon
cargo bench --locked --bench hartley
cargo bench --locked --bench renyi
cargo bench --locked --bench collision
cargo bench --locked --bench min_entropy
cargo test --locked --release --test allocations
```

Run relevant benchmarks when adding a metric or changing an algorithm, its data
representation, or performance-sensitive code. Establish a baseline before
optimizing, record machine/toolchain details and measurement settings, and
compare the same workloads. Preserve baseline results; record new measurements
separately when evaluating changes. Follow the
[Shannon benchmark methodology](docs/benchmarks.md),
[Hartley baseline](docs/benchmarks/hartley.md),
[Rényi baseline](docs/benchmarks/renyi.md),
[collision baseline](docs/benchmarks/collision.md), and
[min-entropy baseline](docs/benchmarks/min_entropy.md).

Performance measurements do not establish mathematical correctness. Continue
to run correctness tests and check allocation promises. Example and prose-only
changes do not require repeating unchanged benchmarks.
