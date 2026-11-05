# Mutual information: fuzz campaign

On 2026-11-05, **100,000 executions passed** in 13 reported seconds, without
sanitizer or assertion failures. Final counters: `cov=661`, `ft=2885`, RSS
462 MiB including the harness/sanitizer. These are not line-coverage percentages;
executions include initialization, not just distinct mutated inputs.

The header is a mode byte followed by a little-endian u64 split. Even modes
reduce that split modulo payload length plus one; odd modes use the midpoint.
Short headers are zero-padded. Maximum input is 1033 bytes, including up to
1024 payload bytes (512 pairs for equal lengths). Unequal split lengths exercise
the error path, including either input being empty.

The independent oracle counts each coordinate directly, sorts u16-encoded
joint pairs, and calculates H(X)+H(Y)-H(X,Y) with ordinary floating entropy
sums. It checks the API against this result with tolerance
`1e-11 * max(1,abs(reference))`, along with nonnegativity, marginal bounds,
symmetry, diagonal identity, exact constant/empty zeros, length errors and
byte/reuse bit agreement. This oracle has cancellation near independence;
separate 100-digit fixtures and relative checks establish tiny-value accuracy.

Eighteen seeds include the joint campaign's empty/short, singleton, constant,
independent/dependent, weighted, unequal, rare, binary and diverse cases, plus
near-independence, nonuniform independence and a series-boundary example.
Stable replay covers 263 generated seeds plus the retained TV oracle regression
across seventeen targets. No production defect was found by this campaign.

```sh
python3 scripts/fuzz_campaign.py --target mutual --runs 100000
```

Settings: nightly-2025-10-25, cargo-fuzz 0.13.1, libfuzzer-sys 0.4.10,
AddressSanitizer, debug assertions and overflow checks; seed 20260920, one worker,
ten-second per-input timeout, 2048 MiB RSS limit. Execution was outside the
restricted sandbox for LeakSanitizer process inspection, with sanitizers enabled.

- [Manifest](mutual-manifest.json): versions, commands, source and seed hashes.
- [Campaign log](mutual.log.gz).

Parent commit: `6d65658`. SHA-256 values identify the tested working-tree code;
archived checkout paths use `$REPO`. CI includes a bounded 10,000-execution run.
Passing fuzzing is not proof of correctness for all inputs or source independence.
