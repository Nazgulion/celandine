# Block-entropy fuzz campaign

On 2026-11-01, **100,000 executions passed** in 66 reported seconds with no
sanitizer or assertion failures. Final counters: `cov=506`, `ft=2338`, RSS
433 MiB including harness/sanitizer overhead. Coverage counters are not line
percentages; execution counts include corpus initialization.

The header has a mode byte and little-endian u64 block length. Even mode uses
the raw length (unrepresentable values map to `usize::MAX`); odd mode reduces it
modulo `payload length + 2` to exercise useful and boundary orders. The campaign
limits the payload to 1024 bytes (1033 total); short headers are zero-padded.
The oracle explicitly slices by position, sorts blocks and counts equal runs,
then sums `-p*ln(p)/ln(2)` without using the library count tree or compensated sum.
It checks errors, finite support bounds, positive endpoint zero, byte/reuse
agreement, byte renaming, reversal and n=1 Shannon agreement. Its floating
oracle is complementary to the 1,254 high-precision fixtures, not a replacement.

Sixteen starting seeds cover empty/short headers, zero length, single-byte,
worked overlapping examples, unequal boundary counts, constant data, full and
oversized lengths, maximum integers, rare transitions and diverse blocks.
Stable replay covers 198 generated seeds plus the retained TV oracle regression
across thirteen targets. No new production defect was found by this campaign.

```sh
python3 scripts/fuzz_campaign.py --target block_entropy --runs 100000
```

Settings: nightly-2025-10-25, cargo-fuzz 0.13.1, libfuzzer-sys 0.4.10,
AddressSanitizer, debug assertions and overflow checks; seed 20260920, one worker,
ten-second per-input timeout, 2048 MiB RSS limit. The campaign ran outside the
restricted sandbox for LeakSanitizer inspection, with sanitizers enabled.

- [Manifest](block_entropy-manifest.json): versions, commands and source hashes.
- [Campaign log](block_entropy.log.gz).

Parent commit: `2b5c378`. SHA-256 values identify the tested working-tree code;
archived checkout paths use `$REPO`. CI also runs a bounded 10,000-execution
campaign. Passing fuzzing is not proof of correctness for every possible input.
