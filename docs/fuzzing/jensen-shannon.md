# Jensen–Shannon fuzz campaign

On 2026-10-23, the new Jensen–Shannon target and the existing total variation
target each passed **100,000 executions** on Linux x86-64 with
nightly-2025-10-25, cargo-fuzz 0.13.1, libfuzzer-sys 0.4.10, and AddressSanitizer.
The shared empty-state error was generalized, so TV was included as a regression
campaign. No sanitizer failure or library defect was found in this run.

| Target | Executions | Reported seconds | cov | ft | Final RSS (MiB) |
| --- | ---: | ---: | ---: | ---: | ---: |
| `total_variation` | 100000 | 16 | 635 | 3192 | 341 |
| `jensen_shannon` | 100000 | 24 | 701 | 3453 | 416 |

These are instrumented coverage features, not source-line coverage percentages.
RSS includes sanitizer and oracle allocations, not just library memory. Counts
include corpus initialization and are not distinct-input counts.

```sh
python3 scripts/fuzz_campaign.py --target total_variation --target jensen_shannon --runs 100000
```

The run used seed 20260920, one worker, maximum input length 4105 bytes, a
10-second per-input timeout, and 2048 MiB RSS limit. Campaigns start from fresh
copies of retained seeds. The Jensen–Shannon corpus has 20 generated seeds;
the TV corpus has 15 generated seeds plus its retained oracle regression.
Across all five targets there are 90 generated seeds and that one extra seed.

Jensen–Shannon uses the TV input layout: byte pairs selected by a split word,
raw count pairs, or bounded count pairs. Checks cover empty errors, count-total
overflow, byte/distribution agreement, exact symmetry and identity, distance
as square root, positive results for different laws, disjoint-support ones,
TV bounds, and an independent entropy-of-mixture reference with pairwise
summation. The latter uses `2e-14` absolute tolerance and cannot establish
relative accuracy for tiny results; high-precision fixtures cover that case.
The seed set adds five cases straddling the implementation's series threshold.

- [Manifest](jensen-shannon-manifest.json): commands, compiler versions,
  source/corpus hashes, parent commit, and successful exit codes.
- [Jensen–Shannon log](jensen-shannon-jensen_shannon.log.gz).
- [Total variation regression log](jensen-shannon-total_variation.log.gz).

Absolute checkout paths are normalized to `$REPO`. Source hashes identify the
working-tree implementation tested; the parent commit contains the earlier TV
increment. The campaigns ran outside the restricted sandbox to allow
LeakSanitizer inspection, with sanitizers enabled. Passing is bounded evidence,
not exhaustive verification. CI automatically includes the new fifth target.
