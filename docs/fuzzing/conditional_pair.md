# Aligned conditional entropy: fuzz campaign

On 2026-11-05, **100,000 executions passed** in 12 reported seconds, without
sanitizer or assertion failures. Final counters: `cov=656`, `ft=2845`, RSS
461 MiB including the harness/sanitizer. These are not line-coverage percentages;
executions include initialization and repeated inputs.

The header contains a mode byte and little-endian u64 split. Even modes reduce
the split modulo payload length plus one; odd modes use the midpoint. Short
headers are zero-padded. The target bounds payloads at 1024 bytes; campaign
inputs are at most 1033 bytes, so equal-length samples contain at most 512 pairs.
Unequal splits exercise `LengthMismatch`, including either input being empty.

The oracle counts each marginal separately, sorts u16-encoded pairs and evaluates
H(X,Y)-H(Y) with ordinary floating entropy sums, independently of production
conditional ratios. It checks the direct and reused APIs with tolerance
`1e-11 * max(1,abs(reference))`, plus bounds, reverse-direction identities,
exact diagonal/deterministic zeros, empty/error behavior and bitwise reuse
agreement. This floating oracle can cancel near zero; separate 100-digit
fixtures and the private extreme-count test verify small-value accuracy.

Nineteen seeds cover empty/short inputs, singletons, constant and independent
laws, dependent pairs, unequal lengths, all byte labels, rare observations,
binary inputs, both conditioning directions, unequal group weights and a rare
ambiguous group. Stable replay now covers 300 generated seeds plus one retained
TV oracle regression across nineteen targets. No production defect was found.

```sh
python3 scripts/fuzz_campaign.py --target conditional_pair --runs 100000
```

Settings: nightly-2025-10-25, cargo-fuzz 0.13.1, libfuzzer-sys 0.4.10,
AddressSanitizer, debug assertions and overflow checks; seed 20260920, one worker,
ten-second per-input timeout, 2048 MiB RSS limit. The campaign ran outside the
restricted sandbox for LeakSanitizer process inspection, with sanitizers enabled.

- [Manifest](conditional_pair-manifest.json): versions, commands, source and seed hashes.
- [Campaign log](conditional_pair.log.gz).

Parent commit: `d2bafa5`. SHA-256 values identify the tested working-tree code;
archived checkout paths use `$REPO`. CI includes a bounded 10,000-execution run.
Fuzzing supplements independent mathematical verification and does not prove
correctness for every input or predictability of an unknown source.
