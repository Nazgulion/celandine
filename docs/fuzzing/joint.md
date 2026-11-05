# Joint distributions and entropy: fuzz campaign

On 2026-11-05, **100,000 executions passed** in 14 reported seconds, without
sanitizer or assertion failures. Final counters: `cov=734`, `ft=3208`, RSS
447 MiB including the harness/sanitizer. These are not line-coverage percentages;
executions include initialization, not just distinct mutated inputs.

A mode byte and little-endian u64 split precede the payload. Even modes reduce
the split modulo payload length plus one; odd modes split at the midpoint.
Short headers are zero-padded. The maximum campaign input is 1033 bytes,
including up to 1024 payload bytes (512 pairs for equal lengths). Odd payload
lengths also exercise rejection. Both empty and one-empty cases are covered.

The oracle encodes each aligned pair as a u16, sorts these values and counts
equal runs. It computes entropy with `-sum p*ln(p)/ln(2)` and forms both
marginals by direct independent byte scans. Checks include exact counts and
support, sample size, empty state, probability queries/sums, marginal tables,
unequal-length errors across all constructors, entropy reuse bit agreement,
coordinate symmetry, diagonal identity and marginal entropy bounds. Floating
comparisons use `1e-11 * max(1,abs(reference))`. Independent 100-digit fixtures
provide a separate numerical reference.

Fifteen starting seeds cover empty/short inputs, singleton/constant laws,
independent/dependent pairings, weighted counts, one-empty and unequal lengths,
all bytes, a rare pair, binary values and midpoint splitting. Stable replay
covers 245 generated seeds plus the retained TV oracle regression across sixteen
targets. No new production defect was found by this campaign.

```sh
python3 scripts/fuzz_campaign.py --target joint --runs 100000
```

Settings: nightly-2025-10-25, cargo-fuzz 0.13.1, libfuzzer-sys 0.4.10,
AddressSanitizer, debug assertions and overflow checks; seed 20260920, one worker,
ten-second per-input timeout, 2048 MiB RSS limit. Execution was outside the
restricted sandbox for LeakSanitizer process inspection, with sanitizers enabled.

- [Manifest](joint-manifest.json): versions, commands, source and seed hashes.
- [Campaign log](joint.log.gz).

Parent commit: `af4e1cd`. SHA-256 values identify the tested working-tree code;
archived checkout paths use `$REPO`. CI includes a bounded 10,000-execution run.
Passing fuzzing is not proof of correctness for every input or source model.
