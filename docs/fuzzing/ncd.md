# Ordered NCD fuzz campaign

On 2026-10-24, **100,000 executions passed** in 81 reported seconds with no
sanitizer or assertion failures. Final counters: `cov=780`, `ft=3022`, RSS 361 MiB
(including the harness and sanitizer).

The NCD target exercises both component normalization and byte-pair compression.
Three little-endian u64 values select compressed sizes, followed by a u16 split
selector and up to 1024 payload bytes (1050 total). Short headers are zero-padded;
excess payload is ignored. The split is selector modulo (payload length + 1),
including both empty-side cases. Supported runtime is Linux x86-64.

The arithmetic oracle widens counts to signed i128 before subtraction, independent
of the library's unsigned difference/sign branches. It checks zero denominators,
negative and above-one results, and unit gaps near the largest sizes. The byte
oracle materializes zlib streams and checks components and the formula exactly.
It shares miniz with the backend; independent C-zlib decoding and rational checks
are supplied separately by `scripts/reference_ncd.py --check`.

```sh
python3 scripts/fuzz_campaign.py --target ncd --runs 100000
```

Settings: nightly-2025-10-25, cargo-fuzz 0.13.1, libfuzzer-sys 0.4.10,
AddressSanitizer, debug assertions and overflow checks; seed 20260920, one worker,
maximum length 1050, ten-second per-input timeout, 2048 MiB RSS limit. The run
executes outside the restricted sandbox for LeakSanitizer process inspection,
with sanitizers enabled. Ten seeds cover short/empty input, wide counts, zeros,
signed cases, repetition and mixed bytes. Stable replay covers 138 generated
seeds plus one retained TV oracle regression across nine targets.

- [Manifest](ncd-manifest.json): tool versions, commands, source and seed hashes.
- [Compressed log](ncd.log.gz).

The parent commit is `99625fe`; hashes identify tested working-tree source.
Archived checkout paths use `$REPO`. Counts include corpus initialization and
are not distinct input counts; coverage counters are not line percentages.
CI includes this target in its bounded 10,000-execution campaigns. Passing
fuzzing does not establish ideal metric axioms for a real compressor.
