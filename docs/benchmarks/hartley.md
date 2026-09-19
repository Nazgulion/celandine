# Hartley baseline

## Reproduction

```sh
cargo bench --locked --bench hartley
cargo test --locked --release --test allocations
```

The harness measures 40 cases: byte inputs at nine sizes for four shapes, plus
four reused-distribution cases. Sizes range from 16 B to 10 MiB and use binary
units (`1 KiB = 1024 B`). It uses the same deterministic inputs and settings as
the [original Shannon baseline](../benchmarks.md): constant zeros, cycling
uniform bytes, about 99% zeros with 1% ones, and fixed-seed xorshift64 mixed bytes.
The seed is `0x123456789abcdef0`, shifts are 13, 7, 17, and the high byte is used.

Criterion uses 30 samples, a 300 ms warmup, and at least one second of measurement
per case. Input generation and allocation are outside the timed loop. Inputs
and outputs pass through `std::hint::black_box`. Byte timings include histogram
construction, support counting, and the logarithm. Distribution timings scan an
existing histogram built from a 64 KiB sample, without recounting the sequence.
Those cases report latency only, because no input bytes are scanned.

Use longer measurements before drawing conclusions from small differences:

```sh
cargo bench --locked --bench hartley -- --sample-size 100 --measurement-time 5 --warm-up-time 1
```

## Environment and limitations

- Run date: 2026-09-19.
- CPU: Intel Core i7-13700HX; x86_64 Linux 6.8.0-134-generic.
- Toolchain: rustc 1.90.0 (`1159e78c4`), LLVM 20.1.8; Criterion 0.7.0.
- Cargo's optimized benchmark profile, with no custom compiler flags supplied
  in the command; one benchmark process, without CPU pinning or a controlled
  frequency policy.

These are local baseline measurements. Frequency scaling, heterogeneous cores,
scheduling, cache state, and other system load affect results. The repeatedly
used input buffers can be cache-resident. This is not a same-run controlled
comparison with the older Shannon measurements. No algorithm optimization was
performed for this milestone.

The implementation uses the existing fixed histogram (256 `usize` counts plus
one total, 2056 bytes on this target). The reused-distribution function uses
constant additional storage. The isolated allocator check reports zero heap
allocation/reallocation calls in both Hartley paths at sizes from 0 B to 10 MiB,
in debug and release builds. Peak stack memory and cycles per byte were not
measured.

Raw local Criterion reports live under `target/criterion/hartley_bytes` and
`target/criterion/hartley_distribution`. The retained
[baseline CSV](hartley-baseline.csv) contains point estimates, 95% confidence
intervals, and byte throughput. Typical latency uses the regression slope if
available, otherwise the sample mean, matching Criterion's reported estimate.

## Measured byte-path latency

Point estimates in microseconds, including counting.

| Input | Constant (µs) | Uniform (µs) | Skewed (µs) | Mixed (µs) | Mixed throughput (MiB/s) |
| --- | ---: | ---: | ---: | ---: | ---: |
| 16 B | 0.075 | 0.074 | 0.077 | 0.074 | 206.6 |
| 64 B | 0.146 | 0.083 | 0.146 | 0.083 | 735.5 |
| 256 B | 0.425 | 0.114 | 0.427 | 0.119 | 2052.3 |
| 1 KiB | 1.423 | 0.244 | 1.407 | 0.274 | 3560.1 |
| 4 KiB | 5.879 | 0.746 | 6.061 | 0.886 | 4408.4 |
| 16 KiB | 24.015 | 2.732 | 24.111 | 3.307 | 4725.1 |
| 64 KiB | 97.270 | 12.959 | 96.187 | 15.874 | 3937.2 |
| 1 MiB | 1563.790 | 198.907 | 1544.622 | 248.522 | 4023.8 |
| 10 MiB | 15743.268 | 1968.376 | 15467.823 | 2551.147 | 3919.8 |

## Measured distribution reuse

| Shape | Latency (ns) |
| --- | ---: |
| constant | 45.9 |
| uniform | 52.2 |
| skewed | 52.2 |
| mixed | 52.3 |
