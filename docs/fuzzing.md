# Dedicated fuzz testing

Fuzzing repeatedly mutates inputs and checks whether the program crashes or
violates an assertion. Coverage-guided fuzzing retains inputs that reach new
instrumented execution paths, helping explore combinations that fixed examples
miss. Celandine uses LLVM's [libFuzzer](https://llvm.org/docs/LibFuzzer.html),
through the Rust Fuzz project's [cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz/tutorial.html).
It complements the independent high-precision fixtures and property tests.

The eleven test-only targets live in a separate `fuzz/` Cargo workspace. Their
dependencies and nightly compiler requirements do not affect the public library
or its Rust 1.90 minimum. The fuzz workspace explicitly enables the optional
compression backend; ordinary consumers retain their own feature selection.

## Targets and checks

| Target | Input layout | Checks |
| --- | --- | --- |
| `byte_entropy` | Two little-endian 64-bit floating-point bit patterns (order, base), then up to 4096 bytes | Counts against sorted runs; all public byte entropy APIs against distribution APIs; numerical bounds, direct-formula references, parameter errors, empty/constant positive zero |
| `count_entropy` | The same 16-byte header, then up to 256 little-endian 64-bit counts | Count-total overflow against a `u128` sum; accepted distributions through every distribution entropy API, including arbitrary order/base values |
| `ngrams` | One little-endian 64-bit length, then up to 512 bytes | All three constructors, errors, overlapping positions, forward/reverse iteration, borrowed pointers, counts and probabilities against independently sorted explicit slices, arbitrary queries |
| `total_variation` | Mode byte: remainder mod 3 selects byte pairs (0), raw counts (1), or bounded counts (2); details below | Empty errors, byte/count equivalence, overflow validation, bounds, exact symmetry/identity, overlap-complement and half-L1 references |
| `jensen_shannon` | Same three-mode layout as total variation | All four APIs, empty errors, count overflow, symmetry/identity, disjoint support, square-root equivalence, TV bounds, independent pairwise entropy-mixture oracle |
| `subword` | One little-endian 64-bit length, then up to 64 bytes (72 total) | Scalar and complete profile against sorted explicit slices; empty word, oversized lengths, reversal/relabeling, forward/reverse iteration |
| `lz76` | Up to 128 raw bytes, no header | Exhaustive count against substring-novelty parsing, integer bounds, bijective renaming, one-symbol extension |
| `compression` | Up to 65,536 raw bytes, no header | Complete zlib length against materialized encoding, round-trip decoding, ratio formula, and empty error |
| `ncd` | Three u64 sizes, a u16 split, up to 1024 bytes | Signed normalization, zero denominator, materialized ordered streams |
| `windows` | Three little-endian u64 values (width, step, skip), 32 operation bytes, up to 256 bytes | Positional oracle, mixed front/back/skips, exact length, borrowed pointers, validation |
| `blocks` | Two little-endian u64 values (width, skip), 32 operation bytes, up to 256 bytes | Complete partitions, original tail stability, mixed iterator operations, borrowing, errors |

Blocks use at most 304 bytes and twelve starting seeds. See [the campaign](fuzzing/blocks.md).

Windows use 312 bytes at most; 14 seeds include gaps, overlap, reverse traversal,
zero parameters and maximum skips. See [the campaign](fuzzing/windows.md).

NCD uses three little-endian u64 compressed sizes, a little-endian u16 split
selector, and up to 1024 payload bytes (1050 total). It checks normalization
against signed i128 arithmetic, materialized zlib sizes and the ordered formula.
Ten seeds include zero denominators, negative/large values, one-byte gaps at
usize::MAX, empty inputs and diverse bytes. See [the campaign](fuzzing/ncd.md).

LZ76 has fourteen seeds covering empty/single inputs, overlapping constant and
periodic strings, distinct labels, binary bytes, published examples, late novelty,
and reversal-sensitive examples. Results require exact integer equality.

Subword profiling is capped at 64 bytes because all lengths are evaluated.
Fifteen new seeds include empty headers, zero/oversized lengths, constant,
alternating, grouped, binary, and distinct-byte samples. Integer results require
exact equality; this target has no floating-point tolerance.

For total variation mode 0, a little-endian u64 split selector follows the mode,
then at most 4096 bytes; the split is selector modulo `(payload_length+1)`.
Modes 1 and 2 read 256 interleaved pairs of little-endian u64 counts after the
mode byte, with short words zero-padded. Mode 2 divides each machine-sized count
by 512, keeping both totals valid while mutations explore wide counts. Mode 1
also tests overflowing imports. Total target length is bounded by 4105 bytes.
The byte oracle sorts runs independently. The exact overlap complement checks
tiny distances as well as zeros; a separate compensated floating half-L1 check uses absolute
`1e-14` tolerance. Fifteen new seeds cover byte pairs, empty states, wide counts,
overflow, proportional laws, disjoint supports, and unit cross-product gaps.

Jensen–Shannon uses the same decoder layout and 15 base cases, plus five
series-boundary seeds. Its pairwise entropy-mixture oracle uses absolute
`2e-14` tolerance; exact zeros, disjoint-support ones, and positivity for different
laws are checked separately. Relative accuracy for tiny divergence comes from
160-digit fixtures, not this floating oracle.

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

Replay the 164 generated seeds plus one retained oracle regression seed with the supported stable compiler:

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
The [campaign record](fuzzing/initial-campaign.md) records the initial three targets;
the [total variation campaign](fuzzing/total-variation.md) records the fourth target.
The [Jensen–Shannon campaign](fuzzing/jensen-shannon.md) records the fifth target
and a fresh TV regression campaign. The [subword campaign](fuzzing/subword.md)
records the sixth target. The [LZ76 campaign](fuzzing/lz76.md) records the seventh.

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

## Optional compression coverage

The separate fuzz workspace enables `compression-deflate`. Its `compression`
target accepts at most 65,536 raw bytes and checks counted length against a
materialized stream, decoding, ratio units, and empty-input behavior. Nine seeds
include empty/single bytes, repetition, diverse data, and 32 KiB boundaries.
The codec round-trip oracle shares miniz and is supplemented by independent
C-zlib decoding in `scripts/reference_compression.py --check`.
Run `python3 scripts/fuzz_campaign.py --target compression --runs 100000`.
See [the retained campaign](fuzzing/compression.md).
