# Conditional entropy by order: fuzz campaign

On 2026-11-05, **100,000 executions passed** in 76 reported seconds, without
sanitizer or assertion failures. Final counters: `cov=455`, `ft=2181`, RSS
414 MiB including the harness/sanitizer. These counters are not line-coverage
percentages; executions include initialization, not just distinct mutated inputs.

The header contains a mode byte and little-endian u64 context order. Mode modulo
three selects raw order (0), order reduced modulo `payload length + 2` (1), or
order reduced modulo 5 (2). Raw values wider than usize map to usize::MAX;
short headers are zero-padded. Maximum campaign input is 1034 bytes, including
up to 1025 payload bytes. Small orders encourage repeated contexts and branching.

The independent oracle counts joint blocks and context prefixes separately by
target position in hash maps, then subtracts their Shannon entropies. It checks
that the library's direct conditional sum agrees within the shared floating
comparison tolerance `1e-11 * max(1,abs(reference))`. It also checks finite
`0..=8` bounds with tolerance, the observed-target entropy bound, exact positive
zero for insufficient observations, order-zero Shannon identity, byte renaming
and reuse whenever order+1 is representable. High-precision fixtures separately
establish numerical accuracy; the floating chain rule cannot certify tiny-value
relative error. Hash-map iteration order can vary without changing these checks.

Seventeen starting seeds include empty/short headers, order zero, alternating
transitions, matched-boundary and weighted-context examples, deterministic laws,
one/no observation, maximum order, rare transitions, all bytes, all 256
successors of one context, and both bounded-order modes. Stable replay covers
215 generated seeds plus the retained TV oracle regression across fourteen targets.
No new production defect was found by this campaign.

```sh
python3 scripts/fuzz_campaign.py --target conditional_order --runs 100000
```

Settings: nightly-2025-10-25, cargo-fuzz 0.13.1, libfuzzer-sys 0.4.10,
AddressSanitizer, debug assertions and overflow checks; seed 20260920, one worker,
ten-second per-input timeout, 2048 MiB RSS limit. Execution was outside the
restricted sandbox for LeakSanitizer process inspection, with sanitizers enabled.
The temporary cargo-fuzz executable was restored from its cached pinned package
before this campaign; the first setup attempt did not execute fuzz inputs.

- [Manifest](conditional_order-manifest.json): versions, commands, source/seed hashes.
- [Campaign log](conditional_order.log.gz).

Parent commit: `e431490`. SHA-256 values identify the tested working-tree code;
archived checkout paths use `$REPO`. CI includes a bounded 10,000-execution run.
Passing fuzzing is not proof of correctness for all inputs or of source predictability.
