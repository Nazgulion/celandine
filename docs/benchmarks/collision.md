# Collision entropy baseline

## Reproduction

```sh
cargo bench --locked --bench collision
cargo test --locked --release --test allocations
```

The harness measures 40 cases: four byte-input shapes at nine sizes, and four
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
The initial implementation delegates to the existing Rényi calculation at order
two, establishing a baseline before any specialized optimization. Older baselines
are retained; these runs are not a controlled same-run performance comparison.

The fixed histogram takes 2056 bytes on this target (256 counts and a total).
Entropy evaluation uses constant additional storage. Isolated debug/release
allocation checks cover both entry points from 0 B through 10 MiB. Peak stack
usage and cycles per byte are not measured.

Local Criterion reports are under `target/criterion/collision_bytes` and
`target/criterion/collision_distribution`. The retained
[baseline CSV](collision-baseline.csv) contains point estimates, 95% confidence
intervals, and byte throughput. Estimates use the regression slope if available,
otherwise the mean, matching Criterion's reported latency. Use longer runs before
interpreting small performance differences:

```sh
cargo bench --locked --bench collision -- --sample-size 100 --measurement-time 5 --warm-up-time 1
```

## Measured byte-path latency

Latencies are microseconds and include counting.

| Input | Constant (µs) | Uniform (µs) | Skewed (µs) | Mixed (µs) | Mixed throughput (MiB/s) |
| --- | ---: | ---: | ---: | ---: | ---: |
| 16 B | 0.200 | 0.334 | 0.290 | 0.339 | 45.1 |
| 64 B | 0.272 | 0.573 | 0.357 | 0.752 | 81.2 |
| 256 B | 0.505 | 1.503 | 0.586 | 1.813 | 134.6 |
| 1 KiB | 1.565 | 1.620 | 1.766 | 3.095 | 315.6 |
| 4 KiB | 6.060 | 3.407 | 5.938 | 4.043 | 966.2 |
| 16 KiB | 24.469 | 5.367 | 24.005 | 6.545 | 2387.2 |
| 64 KiB | 98.221 | 13.749 | 96.034 | 16.257 | 3844.4 |
| 1 MiB | 1563.691 | 188.588 | 1539.434 | 234.118 | 4271.3 |
| 10 MiB | 15702.466 | 1961.334 | 15492.335 | 2552.902 | 3917.1 |

## Measured distribution reuse

| Shape | Latency (ns) |
| --- | ---: |
| constant | 167.6 |
| uniform | 1432.7 |
| skewed | 253.7 |
| mixed | 2972.1 |
