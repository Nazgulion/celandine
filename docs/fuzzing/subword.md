# Subword complexity fuzz campaign

The sixth dedicated target checks scalar counts and complete finite profiles
against an independent oracle that sorts explicit positional slices and removes
duplicates. Integer results require exact equality. It also checks zero and
oversized lengths, forward/reverse profiles, reversal and bijective relabeling,
and direct/reverse skipping (including maximum integer selectors).

```sh
python3 scripts/fuzz_campaign.py --target subword --runs 100000
```

The target reads a little-endian u64 block length and at most 64 payload bytes,
for a maximum total length of 72. Short words are zero-padded and extra trailing
bytes ignored. Complete profiles are deliberately bounded because computing
all lengths is costly. Fifteen generated seeds cover empty input, the empty
word, single bytes, constant and alternating strings, grouped bytes, distinct
bytes, full/oversized lengths, binary data, and short headers.

The campaign uses Linux x86-64, nightly-2025-10-25, cargo-fuzz 0.13.1,
libfuzzer-sys 0.4.10, AddressSanitizer, debug assertions, and overflow checks.
Settings are seed 20260920, one worker, a ten-second per-input timeout, and a
2048 MiB RSS limit. It starts from a fresh copy of retained seeds. Counts include
corpus initialization and are not counts of distinct new inputs.

The ordinary stable replay now exercises 105 generated seeds and one retained
TV oracle regression across all six targets. The separate allocation regression
for profile skipping is documented in the [reference notes](../references/subword.md):
it was found during allocation testing and is retained in normal tests.

On 2026-10-23, the target passed **100,000 executions** in 152 reported seconds,
with no sanitizer failure or mathematical assertion failure. Final counters were
`cov=403`, `ft=1957`, and RSS 431 MiB. Coverage features are not source-line
coverage percentages, and instrumented RSS includes oracle and sanitizer memory.
This is bounded evidence, not exhaustive verification.

- [Manifest](subword-manifest.json): commands, tool versions, parent commit,
  source/corpus hashes, and successful exit status.
- [Compressed campaign log](subword.log.gz).

Absolute checkout paths are normalized to `$REPO`. The parent commit is the
pushed Jensen–Shannon increment, `92ca125`; source hashes identify the tested
working-tree implementation. The campaign ran outside the restricted sandbox
so LeakSanitizer could inspect the process, with sanitizers enabled. CI includes
this target in its bounded 10,000-execution smoke campaigns.
