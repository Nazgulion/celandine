# Initial dedicated fuzz campaign

Date: 2026-09-20. Library implementation: parent commit `5969f8b`, with the fuzz
increment represented by the source hashes in the [retained manifest](initial-manifest.json).
Host: Intel Core i7-13700HX, Linux 6.8.0-134-generic, x86-64 GNU target.
Compiler: nightly-2025-10-25, rustc 1.92.0-nightly (`2aaa62b89`, LLVM 21.1.3).
Tools: cargo-fuzz 0.13.1 and libfuzzer-sys 0.4.10, with `fuzz/Cargo.lock`.

Command from the repository root:

```sh
python3 scripts/fuzz_campaign.py --runs 100000
```

All targets used mutation seed `20260920`, fresh copies of committed seed files,
one worker, optimized builds with debug assertions and integer overflow checks,
AddressSanitizer, a ten-second timeout per input, and a 2048 MiB process RSS
limit. The runner retained per-target logs, source and seed SHA-256 hashes,
commands, and exit codes. No failing input artifacts were produced in the
successful campaign.

| Target | Executions | Maximum encoded input | Final `cov` / `ft` | Fuzzer elapsed time | Exit |
| --- | ---: | ---: | ---: | ---: | ---: |
| `byte_entropy` | 100,000 | 4112 B | 821 / 3415 | 19 s | 0 |
| `count_entropy` | 100,000 | 2064 B | 277 / 685 | 3 s | 0 |
| `ngrams` | 100,000 | 520 B | 448 / 1750 | 31 s | 0 |

The `cov` and `ft` fields are libFuzzer's instrumentation coverage/feature counts
for the complete target, including oracles and dependencies. They are not
percentages of library source coverage. Execution totals include corpus replay;
elapsed time excludes build/setup and is not a library performance benchmark.
Initial tracked seed counts are 21, 22, and 12 respectively (55 total); each
includes an empty file checked by stable replay even when skipped by libFuzzer's
corpus loader.

Retained compressed logs: [byte entropy](byte_entropy.log.gz),
[count entropy](count_entropy.log.gz), and [n-grams](ngrams.log.gz).
Use `gzip -dc docs/fuzzing/byte_entropy.log.gz` to inspect a log. Absolute checkout
paths were replaced with `$REPO` in the archived logs and manifest. Working
mutation corpora remain local and are intentionally excluded from the commit.

An earlier sandboxed byte-entropy run reached 100,000 executions but exited with
LeakSanitizer's documented inability to run under ptrace. It is not counted as a
successful campaign or a library regression. The recorded campaign ran outside
that sandbox with sanitizer checks enabled and all three targets exited zero.

The normal debug/release suites also passed: 66 integration/property entry
points, 21 doctests, and the isolated allocation executable in each profile.
All 1,828 independent reference rows verified; all seven examples ran; root and
fuzz formatting/lints, API documentation, and benchmark compilation passed.
Stable replay checked all 55 fuzz seeds. No library algorithm changed, so no new
performance baseline was measured.

This is initial bounded evidence, not proof that every input is correct. Follow
the [fuzzing workflow](../fuzzing.md) for longer campaigns, failure minimization,
and regression preservation. Large-input and peak-memory measurements remain
separate release work.
