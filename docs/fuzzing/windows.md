# Sliding-window fuzz campaign

On 2026-10-24, **100,000 executions passed** in two reported seconds with no
sanitizer or assertion failures. Final counters: `cov=112`, `ft=407`, RSS 209 MiB
(including the harness and sanitizer). Coverage counters are not line percentages;
execution counts include corpus initialization and are not distinct input counts.

Three little-endian u64 values select width, step and a wide skip. Next come 32
operation bytes and up to 256 payload bytes (312 total). Short headers are
zero-padded. Each operation's remainder modulo four selects next, next_back, nth
or nth_back. If bit 2 is set, its skip is the header value; otherwise the skip is
the operation byte shifted right three bits. Excess payload is ignored.

The independent oracle enumerates every input position, filters by divisibility
and available length, and stores the selected slices in a deque. Mixed operations
must match values, borrowed addresses, remaining exact length, size hints and
cloned traversal. Zero parameters check size-before-step validation. Fourteen
seeds cover overlaps, gaps, adjacency, binary input, empty/short headers, oversized
widths and maximum-integer skips/steps. Stable replay covers all 152 generated
seeds plus one retained TV regression across ten targets.

```sh
python3 scripts/fuzz_campaign.py --target windows --runs 100000
```

Settings: nightly-2025-10-25, cargo-fuzz 0.13.1, libfuzzer-sys 0.4.10,
AddressSanitizer, debug assertions and overflow checks; seed 20260920, one worker,
maximum length 312, ten-second per-input timeout, 2048 MiB RSS limit. The campaign
runs outside the restricted sandbox for LeakSanitizer process inspection, with
sanitizers enabled. Runtime evidence targets Linux x86-64.

- [Manifest](windows-manifest.json): versions, commands, source and seed hashes.
- [Compressed log](windows.log.gz).

The parent commit is `4d4a015`; hashes identify tested working-tree source.
Archived checkout paths use `$REPO`. CI includes this target in its bounded
10,000-execution campaigns. Independent Python fixtures and stateful normal tests
supplement fuzzing; a passing campaign is not exhaustive proof.

## Regression found before the campaign

The first implementation composed Rust 1.90 `slice::windows` and `step_by`.
Its trait test failed because that `StepBy` type does not implement `FusedIterator`.
Adding `fuse()` resolved the trait requirement, but the extreme-skip test hung for
more than 60 seconds on a tiny input when `nth(usize::MAX)` combined with a huge
step. Inspection of the supported compiler's `StepBy::spec_nth` showed an overflow
resolution loop that continued after the underlying iterator was exhausted.

The implementation now bounds skips by remaining window count before offset
arithmetic. `regression_iterator_traits_and_extreme_skips` retains both cases;
the same suite passes with the bounded ordinal implementation. Direct skips are
also included in the [performance baseline](../benchmarks/windows.md).
