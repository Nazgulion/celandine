# Tsallis baseline

## Reproduction

```sh
cargo bench --locked --bench tsallis
cargo test --locked --release --test allocations
```

The harness measures 81 cases: 36 byte cases at order two (nine sizes, four
shapes), nine mixed-byte cases at order `1 + 1e-8`, and 36 reused-distribution
cases (four shapes, nine orders). Sizes are 16 B, 64 B, 256 B, 1 KiB, 4 KiB,
16 KiB, 64 KiB, 1 MiB, and 10 MiB. Binary units use `1 KiB = 1024 B`.

Inputs match the [Shannon baseline](../benchmarks.md): constant zeros, cycling
uniform bytes, roughly 99% zeros and 1% ones, and fixed-seed xorshift64 mixed
bytes (seed `0x123456789abcdef0`, shifts 13, 7, 17, high byte). Distribution cases
reuse counts from 64 KiB inputs and cover orders 0, 0.1, 0.5, `1-1e-8`, 1, `1+1e-8`,
2, one million, and the largest finite `f64`.

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
cache-resident. The older entropy baselines are preserved and are not a
same-run controlled comparison. This milestone establishes the baseline without
performance tuning. The stable per-symbol formula implements the numerical
contract across orders. No special fast path for finite order two has been added.

The fixed histogram occupies 2056 bytes on this target (256 `usize` counts and
a total); entropy evaluation uses constant auxiliary storage. The allocation
check covers valid and invalid orders, both API paths, and 0 B through 10 MiB.
Peak stack memory and cycles per byte are not measured.

Local Criterion reports are under `target/criterion/tsallis_*`. The retained
[reviewed results CSV](tsallis-reviewed.csv) records point estimates, 95% confidence
intervals, and byte throughput. The point estimate uses regression slope when
available, otherwise the mean, matching Criterion's reported latency. Longer
runs are advisable before interpreting small changes:

```sh
cargo bench --locked --bench tsallis -- --sample-size 100 --measurement-time 5 --warm-up-time 1
```

The [original 77-case baseline](tsallis-baseline.csv) is retained from before the
near-zero accuracy fix. The tables below describe the corrected implementation
and add four distribution cases at `q=0.1` to exercise the new formula branch.
These separately timed runs are not a controlled performance comparison.

## Measured byte-path latency

Point estimates include counting. Latencies are microseconds.

| Input | Order 2 constant (µs) | Uniform (µs) | Skewed (µs) | Mixed (µs) | Mixed (MiB/s) | Near-1 mixed (µs) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 16 B | 0.077 | 0.373 | 0.171 | 0.380 | 40.1 | 0.289 |
| 64 B | 0.148 | 1.093 | 0.245 | 0.987 | 61.9 | 0.698 |
| 256 B | 0.431 | 3.930 | 0.483 | 2.616 | 93.3 | 1.868 |
| 1 KiB | 1.423 | 4.055 | 1.484 | 4.077 | 239.6 | 2.902 |
| 4 KiB | 5.908 | 4.566 | 5.901 | 4.688 | 833.2 | 3.525 |
| 16 KiB | 23.829 | 7.940 | 23.840 | 7.110 | 2197.7 | 5.940 |
| 64 KiB | 97.500 | 16.115 | 96.040 | 19.465 | 3211.0 | 20.410 |
| 1 MiB | 1567.181 | 210.785 | 1549.446 | 244.644 | 4087.6 | 242.430 |
| 10 MiB | 15651.409 | 1994.220 | 15442.010 | 2575.443 | 3882.8 | 2570.013 |

## Measured distribution reuse

Latencies are nanoseconds; the histogram is already available.

| Shape | q=0 | q=0.1 | q=0.5 | q=1−1e-8 | q=1 | q=1+1e-8 | q=2 | q=1e6 | q=f64::MAX |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| constant | 44.9 | 44.6 | 45.0 | 44.8 | 212.6 | 44.7 | 44.7 | 44.6 | 44.7 |
| uniform | 45.4 | 2430.6 | 3974.4 | 2701.0 | 1471.3 | 2700.0 | 3867.1 | 1512.3 | 1556.7 |
| skewed | 45.4 | 134.5 | 141.7 | 134.0 | 224.9 | 134.7 | 144.1 | 119.4 | 176.4 |
| mixed | 46.0 | 2418.0 | 3979.8 | 2706.2 | 1473.3 | 2699.4 | 3911.6 | 1506.4 | 1554.2 |
