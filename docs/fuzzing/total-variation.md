# Total variation fuzz campaign

On 2026-09-20 the new `total_variation` target passed **100,000 executions**
with the pinned nightly-2025-10-25 compiler, cargo-fuzz 0.13.1, libfuzzer-sys
0.4.10, and AddressSanitizer on Linux x86-64. libFuzzer reported 12 seconds,
coverage counters `cov: 635`, `ft: 3189`, and final RSS 314 MiB. These counters
are instrumented features, not source-line coverage percentages; the run count
includes seed initialization.

The target covers raw byte pairs and raw/bounded imported count pairs. It checks
empty errors, import overflow, bounds, symmetry, identity, byte/distribution
agreement, an exact integer overlap-complement reference, and an independent
compensated floating half-L1 reference. The 15 generated seeds plus one retained
regression seed are replayed on stable and copied into each fresh campaign.
See the [target layout and setup](../fuzzing.md).

```sh
python3 scripts/fuzz_campaign.py --target total_variation --runs 100000
```

Settings: seed 20260920, maximum input length 4105 bytes, one worker,
10-second per-input timeout, 2048 MiB RSS limit, debug assertions and overflow
checks enabled. A fixed seed does not promise identical traces across machines.
The local campaign ran outside the restricted sandbox so LeakSanitizer could
complete its process inspection. Sanitizers were not disabled.

## Oracle regression discovered and corrected

The initial run stopped at an assertion in the independent floating oracle.
For that input, exact rational TV and the library result agreed at
`0.9366721854304636`; the ordinary 256-term oracle sum gave
`0.9366721854304528`, drifting by about `1.08e-14` beyond its `1e-14` bound.
This was a test-oracle defect; the library's exact count algorithm was unchanged.

`cargo fuzz tmin total_variation <artifact> -- -runs=10000` reduced the
2429-byte input to 248 bytes, retained as
`fuzz/seeds/total_variation/regression-oracle-summation.bin`. Its sample lengths
are 4 and 235; independent Python `Fraction` arithmetic gives **224/235**.
Stable replay failed on that seed before the correction and passed afterward.
The floating oracle now uses compensated summation with the same tolerance;
the exact overlap-complement check remains unchanged. A fresh full campaign,
including the retained seed, then passed.

- [Successful manifest](total-variation-manifest.json): source and seed SHA-256
  hashes, tool versions, command, parent commit, and successful exit status.
- [Successful log](total-variation.log.gz): retained libFuzzer output.
- [Oracle failure evidence](total-variation-oracle-failure.tar.gz): original
  artifact, initial log/manifest, minimization log, and failing stable replay.

`$REPO` replaces the absolute checkout path in retained text. The parent commit
predates this uncommitted increment; source hashes identify the tested code.
CI automatically includes this fourth target in its bounded campaigns. Passing
this run is not proof of exhaustive coverage or an accuracy guarantee; the
independent rational fixtures and mathematical properties remain necessary.
