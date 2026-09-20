# Rényi baseline

The results below preserve the original baseline. See the [100 MiB and peak-memory
supplement](large-input-memory.md) for later measurements and reproduction commands.
Current byte harnesses also include the added 100 MiB tier.

## Reproduction

```sh
cargo bench --locked --bench renyi
cargo test --locked --release --test allocations
```

The original measured harness covered 77 cases: 36 byte cases at order two (nine sizes, four
shapes), nine mixed-byte cases at order `1 + 1e-8`, and 32 reused-distribution
cases (four shapes, eight orders). Sizes are 16 B, 64 B, 256 B, 1 KiB, 4 KiB,
16 KiB, 64 KiB, 1 MiB, and 10 MiB. Binary units use `1 KiB = 1024 B`.

Inputs match the [Shannon baseline](../benchmarks.md): constant zeros, cycling
uniform bytes, roughly 99% zeros and 1% ones, and fixed-seed xorshift64 mixed
bytes (seed `0x123456789abcdef0`, shifts 13, 7, 17, high byte). Distribution cases
reuse counts from 64 KiB inputs and cover orders 0, 0.5, `1-1e-8`, 1, `1+1e-8`,
2, one million, and positive infinity.

Criterion uses 30 samples, 300 ms warmup, and at least one second of measurement
per case. Inputs and outputs, including the order parameter, pass through
`black_box`. Input generation is outside the timed loop. Byte timings include
validation, counting, and entropy evaluation; distribution timings omit counting
and report latency only. Both timings include unwrapping the successful result.

## Environment and limitations

- Run date: 2026-09-19.
- CPU: Intel Core i7-13700HX; x86_64 Linux 6.8.0-134-generic.
- Toolchain: rustc 1.90.0 (`1159e78c4`), LLVM 20.1.8; Criterion 0.7.0.
- Cargo's optimized benchmark profile; no custom compiler flags supplied in the
  command, CPU pinning, or controlled frequency policy.

These are local baseline measurements; frequency scaling, scheduling, cache
state, heterogeneous cores, and system load affect them. Reused buffers may be
cache-resident. The older Shannon/Hartley baselines are preserved and are not a
same-run controlled comparison. This milestone establishes the baseline without
performance tuning. Near-one stability and large-order scaling implement the
numerical contract. No special fast path for finite order two has been added.

The fixed histogram occupies 2056 bytes on this target (256 `usize` counts and
a total); entropy evaluation uses constant auxiliary storage. The allocation
check covers valid and invalid orders, both API paths, and 0 B through 10 MiB.
Peak stack memory and cycles per byte are not measured.

Local Criterion reports are under `target/criterion/renyi_*`. The retained
[baseline CSV](renyi-baseline.csv) records point estimates, 95% confidence
intervals, and byte throughput. The point estimate uses regression slope when
available, otherwise the mean, matching Criterion's reported latency. Longer
runs are advisable before interpreting small changes:

```sh
cargo bench --locked --bench renyi -- --sample-size 100 --measurement-time 5 --warm-up-time 1
```

## Measured byte-path latency

Point estimates include counting. Latencies are microseconds.

| Input | Order 2 constant (µs) | Uniform (µs) | Skewed (µs) | Mixed (µs) | Mixed (MiB/s) | Near-1 mixed (µs) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 16 B | 0.200 | 0.335 | 0.288 | 0.341 | 44.8 | 0.437 |
| 64 B | 0.273 | 0.596 | 0.362 | 0.755 | 80.8 | 0.863 |
| 256 B | 0.506 | 1.622 | 0.591 | 1.816 | 134.4 | 2.106 |
| 1 KiB | 1.584 | 1.732 | 1.605 | 3.076 | 317.5 | 4.379 |
| 4 KiB | 6.131 | 2.210 | 5.943 | 5.278 | 740.1 | 3.804 |
| 16 KiB | 24.579 | 4.154 | 24.033 | 7.810 | 2000.7 | 9.045 |
| 64 KiB | 97.109 | 13.900 | 96.297 | 21.401 | 2920.5 | 21.381 |
| 1 MiB | 1560.711 | 206.779 | 1544.443 | 255.415 | 3915.2 | 256.333 |
| 10 MiB | 15699.678 | 2042.975 | 15476.500 | 2577.505 | 3879.7 | 2559.706 |

## Measured distribution reuse

Latencies are nanoseconds; the histogram is already available.

| Shape | α=0 | α=0.5 | α=1−1e-8 | α=1 | α=1+1e-8 | α=2 | α=1e6 | α=∞ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| constant | 47.2 | 170.4 | 168.7 | 211.5 | 168.9 | 169.5 | 168.7 | 169.7 |
| uniform | 52.7 | 1541.6 | 2945.5 | 1481.9 | 2953.9 | 1541.8 | 1547.2 | 172.8 |
| skewed | 53.2 | 253.2 | 278.9 | 225.6 | 279.0 | 253.8 | 251.6 | 173.5 |
| mixed | 53.2 | 2976.0 | 2964.3 | 1479.4 | 2964.8 | 2967.5 | 2192.3 | 173.0 |
