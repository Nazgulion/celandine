# Min-entropy baseline

The results below preserve the original baseline. See the [100 MiB and peak-memory
supplement](large-input-memory.md) for later measurements and reproduction commands.
Current byte harnesses also include the added 100 MiB tier.

## Reproduction

```sh
cargo bench --locked --bench min_entropy
cargo test --locked --release --test allocations
```

The original measured harness covered 40 cases: four byte-input shapes at nine sizes, and four
reused distributions. Sizes are 16 B, 64 B, 256 B, 1 KiB, 4 KiB, 16 KiB,
64 KiB, 1 MiB, and 10 MiB, using binary units (`1 KiB = 1024 B`). Inputs match
the [Shannon baseline](../benchmarks.md): constant zero bytes, cycling uniform
bytes, about 99% zeros with 1% ones, and fixed-seed xorshift64 mixed bytes.
The seed is `0x123456789abcdef0`, shifts are 13, 7, 17, and the high byte is used.

Criterion uses 30 samples, 300 ms warmup, and at least one second of measurement
per case. Input generation is outside the timed loop; both input and output pass
through `black_box`. Byte timings include counting and entropy evaluation.
Distribution timings reuse a histogram built from 64 KiB of data and report
latency only, because they do not rescan those bytes.

## Environment and limitations

- Run date: 2026-09-19.
- CPU: Intel Core i7-13700HX; x86_64 Linux 6.8.0-134-generic.
- Toolchain: rustc 1.90.0 (`1159e78c4`), LLVM 20.1.8; Criterion 0.7.0.
- Cargo's optimized benchmark profile; no custom compiler flags supplied in the
  command, CPU pinning, or controlled frequency policy.

These are local measurements affected by frequency scaling, heterogeneous cores,
scheduling, cache state, and other system load. Reused buffers may be cache-resident.
The initial implementation delegates to the existing Rényi calculation at positive
infinity, establishing a baseline before any specialized optimization. Older baselines
are retained; these runs are not a controlled same-run performance comparison.

The fixed histogram takes 2056 bytes on this target (256 counts and a total).
Entropy evaluation uses constant additional storage. Isolated debug/release
allocation checks cover both entry points from 0 B through 10 MiB. Peak stack
usage and cycles per byte are not measured.

Local Criterion reports are under `target/criterion/min_entropy_bytes` and
`target/criterion/min_entropy_distribution`. The retained
[baseline CSV](min_entropy-baseline.csv) contains point estimates, 95% confidence
intervals, and byte throughput. Estimates use the regression slope if available,
otherwise the mean, matching Criterion's reported latency. Use longer runs before
interpreting small performance differences:

```sh
cargo bench --locked --bench min_entropy -- --sample-size 100 --measurement-time 5 --warm-up-time 1
```

## Measured byte-path latency

Latencies are microseconds and include counting.

| Input | Constant (µs) | Uniform (µs) | Skewed (µs) | Mixed (µs) | Mixed throughput (MiB/s) |
| --- | ---: | ---: | ---: | ---: | ---: |
| 16 B | 0.189 | 0.197 | 0.200 | 0.195 | 78.4 |
| 64 B | 0.260 | 0.206 | 0.266 | 0.205 | 298.4 |
| 256 B | 0.493 | 0.241 | 0.493 | 0.243 | 1006.2 |
| 1 KiB | 1.563 | 0.364 | 1.530 | 0.391 | 2498.0 |
| 4 KiB | 5.995 | 0.855 | 6.055 | 2.246 | 1739.3 |
| 16 KiB | 24.503 | 2.781 | 24.255 | 4.700 | 3324.6 |
| 64 KiB | 97.379 | 13.897 | 96.238 | 18.499 | 3378.5 |
| 1 MiB | 1569.109 | 212.019 | 1545.293 | 276.575 | 3615.7 |
| 10 MiB | 15601.867 | 2007.352 | 15564.036 | 2602.748 | 3842.1 |

## Measured distribution reuse

| Shape | Latency (ns) |
| --- | ---: |
| constant | 158.9 |
| uniform | 161.3 |
| skewed | 162.0 |
| mixed | 161.3 |
