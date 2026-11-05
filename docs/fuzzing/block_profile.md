# Block-entropy profiles: fuzz campaign

On 2026-11-05, **100,000 executions passed** in 27 reported seconds, without
sanitizer or assertion failures. Final counters: `cov=420`, `ft=2370`, RSS
433 MiB including the harness/sanitizer. These counters are not line-coverage
percentages; executions include initialization, not just distinct mutated inputs.

The first two little-endian u64 words encode maximum order and skip count.
Short words are zero-padded; values wider than usize map to usize::MAX. Bytes
16 through 31 encode traversal operations, followed by the data payload.
The campaign's maximum input of 64 bytes bounds that payload to 32 bytes.
Actions select next, next_back, nth, nth_back, clone/collect, count, last or
reverse/collect. Skips use either the header count or a small action-derived
count. The full profile is bounded by the input length even for maximum orders.

The independent oracle enumerates overlapping blocks by position, sorts them,
counts equal runs and sums `-p*ln(p)/ln(2)`. A deque models the remaining entries.
Every state checks exact length and size hint; operations compare results with
tolerance `1e-11 * max(1,abs(reference))`, and exhausted traversal is checked
from both ends, including maximum skips. Independent 100-digit fixtures and
literal scalar bit-agreement tests supplement this floating oracle.

Fifteen starting seeds cover empty/short headers, zero limits, singleton and
constant inputs, worked and nonmonotone profiles, prefixes, oversized/maximum
limits, maximum/tail skips, and binary/diverse data. Stable replay covers 230
generated seeds plus the retained TV oracle regression across fifteen targets.
No new production defect was found by this campaign.

```sh
python3 scripts/fuzz_campaign.py --target block_profile --runs 100000
```

Settings: nightly-2025-10-25, cargo-fuzz 0.13.1, libfuzzer-sys 0.4.10,
AddressSanitizer, debug assertions and overflow checks; seed 20260920, one worker,
ten-second per-input timeout, 2048 MiB RSS limit. Execution was outside the
restricted sandbox for LeakSanitizer process inspection, with sanitizers enabled.

- [Manifest](block_profile-manifest.json): versions, commands, source/seed hashes.
- [Campaign log](block_profile.log.gz).

Parent commit: `0b48016`. SHA-256 values identify the tested working-tree code;
archived checkout paths use `$REPO`. CI includes a bounded 10,000-execution run.
Passing fuzzing is not proof of correctness for all inputs or source properties.
