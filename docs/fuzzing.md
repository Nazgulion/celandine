# Dedicated fuzz testing

Fuzzing repeatedly mutates inputs and checks whether the program crashes or
violates an assertion. Coverage-guided fuzzing retains inputs that reach new
instrumented execution paths, helping explore combinations that fixed examples
miss. Celandine uses LLVM's [libFuzzer](https://llvm.org/docs/LibFuzzer.html),
through the Rust Fuzz project's [cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz/tutorial.html).
It complements the independent high-precision fixtures and property tests.

The three test-only targets live in a separate `fuzz/` Cargo workspace. Their
dependencies and nightly compiler requirements do not affect the public library
or its Rust 1.90 minimum. No mathematical APIs or runtime dependencies are added.

## Targets and checks

| Target | Input layout | Checks |
| --- | --- | --- |
| `byte_entropy` | Two little-endian 64-bit floating-point bit patterns (order, base), then up to 4096 bytes | Counts against sorted runs; all public byte entropy APIs against distribution APIs; numerical bounds, direct-formula references, parameter errors, empty/constant positive zero |
| `count_entropy` | The same 16-byte header, then up to 256 little-endian 64-bit counts | Count-total overflow against a `u128` sum; accepted distributions through every distribution entropy API, including arbitrary order/base values |
| `ngrams` | One little-endian 64-bit length, then up to 512 bytes | All three constructors, errors, overlapping positions, forward/reverse iteration, borrowed pointers, counts and probabilities against independently sorted explicit slices, arbitrary queries |

Short headers/count words are zero-padded. Trailing input beyond each target's
limit is ignored. On the supported Linux x86-64 fuzz target, counts and lengths
map exactly to `usize`; casts truncate on narrower machines, which are outside
this campaign's runtime evidence. Limits keep allocations and oracle costs
bounded; a maximum integer n-gram length is still exercised without allocating
an input of that size. The generated seed corpus includes zeros, infinities,
NaN, subnormals, adjacent values around one, maximum finite orders, exact maximum
count totals, overflow, all byte values, and oversized block lengths.

For example, encode order `2.0`, base `2.0`, and bytes `AAAB`. Sorting independently
gives counts 3 and 1, hence probabilities `3/4` and `1/4`. Collision entropy is
`-log2((3/4)^2 + (1/4)^2) = 0.6780719051126377` bits per symbol; the target checks
that value, Rényi order two's equivalence, and the bounds
`H_min <= H_collision <= H_Shannon <= H_Hartley`. Here each `H` is the named entropy
in bits per symbol. For `ABABA` with block length two, explicit positions produce
`AB, BA, AB, BA`, so the n-gram oracle expects two counts each and probabilities
`2/4 = 0.5`, measured as dimensionless fractions of block occurrences.

The general floating comparison uses
`abs(actual - reference) <= 1e-11 * max(1, abs(reference))` in the compared units.
Shannon base checks convert to nats before comparison, avoiding amplified error
near base one. Direct power formulas for Rényi/Tsallis are used only away from
order one and at moderate orders; otherwise domain, finite-result, range, and
special-order identities still apply. These `f64` oracles catch gross numerical
regressions but do not establish relative accuracy for tiny values. Existing
high-precision fixtures remain the accuracy authority.

## Setup and reproducible campaigns

Linux x86-64 setup needs Rust, Python 3.10+, and a C++ compiler. Install the
pinned development tools without changing the default compiler:

```sh
rustup toolchain install nightly-2025-10-25 --profile minimal --component rust-src
cargo install cargo-fuzz --version 0.13.1 --locked
cargo fetch --manifest-path fuzz/Cargo.toml --locked
```

The [Rust Fuzz setup guide](https://rust-fuzz.github.io/book/cargo-fuzz/setup.html)
explains the nightly/sanitizer requirement. The committed fuzz lockfile pins
`libfuzzer-sys` and transitive dependencies separately from the library lockfile.

Replay the 55 committed starting seeds with the supported stable compiler:

```sh
python3 scripts/fuzz_seeds.py --check
cargo fmt --manifest-path fuzz/Cargo.toml -- --check
cargo clippy --manifest-path fuzz/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path fuzz/Cargo.toml --locked --lib
```

Run a campaign, or select one target:

```sh
python3 scripts/fuzz_campaign.py --runs 100000
python3 scripts/fuzz_campaign.py --target count_entropy --runs 1000000 --seed 42
```

The runner checks lockfile resolution, then builds/runs offline with the pinned
nightly and cargo-fuzz, AddressSanitizer, debug assertions, and overflow checks.
Each invocation starts from a fresh copy of committed seeds in `fuzz/runs/`.
Targets run sequentially with one worker, a fixed mutation seed, a ten-second
per-input timeout, and a 2048 MiB process RSS limit. The run count includes corpus
initialization, so it is not a count of distinct new inputs. Empty seed files
are replayed by the stable test even if libFuzzer skips them during corpus loading.

Each campaign retains logs, working corpora, any crash artifacts, source/corpus
SHA-256 hashes, commands, compiler versions, parent commit, and exit status in its
output directory. These generated files are ignored by Git. Retain the same
sources, seeds, compiler, dependencies, and flags to reproduce the setup; a fixed
random seed alone does not promise identical mutation traces across machines.
The [campaign record](fuzzing/initial-campaign.md) records the initial results.

## Failures and continuous integration

On a failure, the runner exits nonzero and reports the log/artifact directory.
Replay the saved file, then minimize it:

```sh
cargo +nightly-2025-10-25 fuzz run count_entropy /path/to/crash
cargo +nightly-2025-10-25 fuzz tmin count_entropy /path/to/crash
```

First distinguish a library defect from an invalid oracle assumption. Preserve
the original artifact until that is resolved. For a library defect, retain the
minimized input in `fuzz/seeds/<target>/regression-<description>.bin` and add a
normal regression test with independently justified expected behavior. Verify
that the test fails before the fix and passes afterward. Extra regression seeds
are allowed by the generator check and automatically included in stable replay
and future campaigns. Do not commit the whole mutation corpus after each run.

CI replays seeds and lints the harness on Rust 1.90.0. A separate job runs 10,000
executions per target with the pinned nightly and uploads logs, corpus, and
failure artifacts for 14 days, even after a failed campaign. Longer local runs
remain useful; passing a bounded campaign is not proof of exhaustive input or
code coverage. Fuzzer RSS includes instrumentation and oracle allocations and
does not replace the planned library peak-memory benchmarks.
