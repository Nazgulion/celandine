# Normalized Jensen–Shannon statistical-complexity fuzz campaign

On 2026-11-01, **100,000 executions passed** in two reported seconds, with no
sanitizer or assertion failures. Final counters: `cov=248`, `ft=699`, RSS 411 MiB
(including the harness and sanitizer). Coverage counters are not line percentages;
execution counts include corpus initialization, not just distinct new inputs.

The header is a mode byte and little-endian u64 alphabet size K. Mode modulo
three selects up to 4096 byte observations (0), 256 raw u64 counts (1), or those
counts divided by 256 before import (2). Maximum input is 4105 bytes. Short
words are zero-padded and excess payload is ignored. Mode 1 verifies count-total
overflow independently in u128; mode 2 guarantees a representable sum. K casts
to usize; runtime evidence targets Linux x86-64.

For valid imported laws, the target checks alphabet/empty/support error
precedence, byte/distribution equivalence, finite range, exact positive endpoint
zeros, and strict positivity away from both endpoints. Its independent oracle
evaluates entropy of explicit mixtures and separately evaluates a point-mass
mixture for Jmax. This floating entropy-difference check uses absolute tolerance
2e-12; it cannot establish relative accuracy near zero. The 853 Decimal fixtures
supply that evidence separately, with relative tolerance 5e-13 for nonzero values.

Eighteen seeds cover zero/one/oversized/maximum alphabets, empty/short input,
point mass, full and partial uniformity, the worked case, excessive support,
wide counts, overflow, bounded counts, the near-certainty regression, and a
one-count departure from uniformity. Stable replay covers 182 generated seeds
plus the retained TV oracle regression across twelve targets.

```sh
python3 scripts/fuzz_campaign.py --target statistical_complexity --runs 100000
```

Settings: nightly-2025-10-25, cargo-fuzz 0.13.1, libfuzzer-sys 0.4.10,
AddressSanitizer, debug assertions and overflow checks; seed 20260920, one worker,
ten-second per-input timeout and 2048 MiB RSS limit. The run executes outside the
restricted sandbox for LeakSanitizer inspection, with sanitizers enabled.

- [Manifest](statistical_complexity-manifest.json): versions, commands and hashes.
- [Campaign log](statistical_complexity.log.gz).
- [Initial failing normal tests](statistical_complexity-initial-tests.log.gz):
  the near-certainty numerical issue, before the complementary-count fix.

Parent commit: `e291330`; source hashes identify tested working-tree files.
Archived checkout paths use `$REPO`. See [the regression explanation](../references/statistical_complexity.md).
All normal tests passed after the fix; no new defect was found by this campaign.
CI includes a bounded 10,000-execution campaign. Passing fuzzing is not proof of
exhaustive correctness or of an interpretation as temporal complexity.
