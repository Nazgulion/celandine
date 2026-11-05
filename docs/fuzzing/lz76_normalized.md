# Explicit LZ76 scales: fuzz campaign

On 2026-11-05, **100,000 executions passed** in 16 reported seconds, without
sanitizer or assertion failures. Final counters were `cov=613`, `ft=2014`,
RSS 150 MiB including the harness/sanitizer. These counters are not line-coverage
percentages; executions include initialization and repeated inputs.

The target reads a little-endian u64 declared alphabet followed by up to 128
payload bytes, for a maximum of 136 bytes. Short headers are zero-padded;
unrepresentable alphabets map to usize::MAX on narrower targets. Range errors
and support errors are checked separately, with range taking precedence.

The existing LZ76 substring-novelty oracle verifies the count and bijective
renaming. The normalized harness independently counts support using a set,
checks summary metadata, density bounds, scalar/summary bit agreement, the
natural-log normalization formula and nonincrease with larger valid alphabets.
Floating comparisons use `1e-11 * max(1,abs(reference))`; the normal test suite
adds 100-digit fixtures and exact examples that exceed one.

Eighteen seeds cover empty and short headers, singleton, constant/periodic
strings, published histories, late novelty, diverse bytes, high numeric labels,
invalid alphabets, insufficient alphabets and multiple valid alphabet sizes.
Stable replay now covers 281 generated seeds plus one retained TV oracle
regression across eighteen targets. No new production defect was found.

```sh
python3 scripts/fuzz_campaign.py --target lz76_normalized --runs 100000
```

Settings: nightly-2025-10-25, cargo-fuzz 0.13.1, libfuzzer-sys 0.4.10,
AddressSanitizer, debug assertions and overflow checks; seed 20260920, one worker,
ten-second per-input timeout, 2048 MiB RSS limit. Execution was outside the
restricted sandbox for LeakSanitizer process inspection, with sanitizers enabled.

- [Manifest](lz76_normalized-manifest.json): versions, commands, source and seed hashes.
- [Campaign log](lz76_normalized.log.gz).

Parent commit: `5fe8e31`. SHA-256 values identify the tested working-tree code;
archived checkout paths use `$REPO`. CI includes a bounded 10,000-execution run.
Passing fuzzing does not establish correctness for all inputs or entropy-rate
accuracy for an unknown source.
