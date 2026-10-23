# Optional compression fuzz campaign

On 2026-10-23, **100,000 executions passed** in 678 reported seconds with no
sanitizer or assertion failures. Final counters were `cov=1213`, `ft=4810`, and
RSS 426 MiB (including the harness and sanitizer).

The target measures the fixed zlib backend through `compressed_length` and
`compression_ratio`. It compares the counted size with a materialized miniz
stream, decodes that stream, checks the ratio formula exactly, and rejects the
empty ratio. This codec oracle shares miniz with the backend; independent
C-zlib decoding of actual counted streams is separately provided by
`scripts/reference_compression.py --check`.

```sh
python3 scripts/fuzz_campaign.py --target compression --runs 100000
```

The run uses Linux x86-64, nightly-2025-10-25, cargo-fuzz 0.13.1,
libfuzzer-sys 0.4.10, AddressSanitizer, debug assertions, and overflow checks.
Settings are seed 20260920, one worker, maximum length 65,536 bytes, ten-second
per-input timeout, and a 2048 MiB RSS limit. Input is raw bytes; excess bytes are
ignored by the harness. Nine starting seeds cover empty/single input, periodic
and constant input, late novelty, mixed bytes, and sizes around the 32 KiB window.
Stable replay covers 128 generated seeds plus one retained TV oracle regression
across eight targets. Stable tests replay empty files even when libFuzzer omits
them during corpus initialization.

- [Manifest](compression-manifest.json): versions, commands, source/corpus hashes,
  parent commit, and exit status.
- [Compressed campaign log](compression.log.gz).

The parent commit is `f30cd44` (LZ76). Source hashes identify the tested working
tree; archived checkout paths are normalized to `$REPO`. The campaign runs
outside the restricted sandbox so LeakSanitizer can inspect the process, with
sanitizers enabled. CI includes this target in its 10,000-execution campaigns.
Execution counts include corpus initialization and are not distinct-input counts.
Coverage counters are instrumented features, not line-coverage percentages. This
is bounded evidence, not exhaustive verification or a substitute for references.
