# Shannon explicit-base baseline

The results below preserve the original baseline. See the [100 MiB and peak-memory
supplement](large-input-memory.md) for later measurements and reproduction commands.
Current byte harnesses also include the added 100 MiB tier.

## Reproduction and workloads

```sh
cargo bench --locked --bench shannon_base
cargo test --locked --release --test allocations
```

This baseline measures the new base-conversion APIs and the existing bit-valued
APIs as controls in the same run. The original Shannon calculation is unchanged:
the wrappers validate the base, call Shannon, and divide by `log2(base)`.
Earlier [Shannon measurements](../benchmarks.md) remain preserved separately.

The original measured harness covered 168 cases:

- 144 byte-input cases: four shapes, nine sizes, and four calls (original
  `shannon`, explicit base 2, base `E`, and base 10).
- 24 reused-distribution cases: four shapes and six calls (original
  `shannon_distribution`, bases `1.0.next_up()`, 2, `E`, 10, and `f64::MAX`).

Byte sizes are 16 B, 64 B, 256 B, 1 KiB, 4 KiB, 16 KiB, 64 KiB, 1 MiB, and
10 MiB. Binary units are used (`1 MiB = 1048576 B`). Shapes match the existing
Shannon suite: constant zero, cycling bytes `i mod 256` (uniform), byte one
every hundred positions and zero otherwise (skewed), and deterministic mixed
bytes. Mixed data uses xorshift64 with seed `0x123456789abcdef0`, shifts 13,
7, 17, and the high byte of each state. It is benchmark input, not a source model.

Input generation and allocation are outside timing. Inputs, bases, and outputs
pass through `black_box`, so even the base-two wrapper receives a runtime base.
Byte timings include counting and conversion. Reuse timings read a distribution
already built from 64 KiB and report latency only, since input is not rescanned.
The default controls do not validate a base or return `Result`.

Criterion uses 10 samples, 100 ms warmup, and at least 500 ms measurement per
case, extending duration for slow cases. These survey settings differ from the
older Shannon baseline's 30 samples and one-second measurement. Use the controls
in this run for comparisons; do not attribute changes from historical timing
to this addition alone. For closer comparisons, preserve this baseline and run:

```sh
cargo bench --locked --bench shannon_base -- --sample-size 100 --measurement-time 5 --warm-up-time 1
```

## Environment and limitations

- Run date: 2026-09-20.
- CPU: Intel Core i7-13700HX; x86_64 Linux 6.8.0-134-generic.
- Toolchain: rustc 1.90.0 (`1159e78c4`), LLVM 20.1.8; Criterion 0.7.0.
- Cargo's optimized benchmark profile; no custom compiler flags, CPU pinning,
  or controlled frequency policy supplied.
- No other project tests or benchmark processes ran during the measurements.

Scheduling, frequency scaling, heterogeneous cores, cache residency, and other
system activity affect results. Small differences in the same-run controls can
also be noise; these measurements do not establish a universal speed guarantee.

Isolated debug/release allocation checks cover valid common and extreme bases,
invalid bases, and both API paths from 0 B through 10 MiB. They confirm zero heap
allocation calls. The byte histogram still uses 2056 bytes on this target;
conversion needs constant additional storage. Peak stack usage and cycles per
byte are not measured. Numerical accuracy is checked independently of timing.

The retained [CSV](shannon_base-baseline.csv) contains all 168 point estimates,
95% confidence intervals, and input throughput where applicable. Estimates
use the regression slope when available, otherwise the mean. Local raw reports
are in `target/criterion/shannon_base_bytes` and
`target/criterion/shannon_base_distribution`.

## Measured comparison

Latencies below are microseconds. Original denotes the existing bit-valued
API; the other columns include validation and runtime base conversion.

| Workload | Original (µs) | Base 2 (µs) | Base E (µs) | Base 10 (µs) |
| --- | ---: | ---: | ---: | ---: |
| 16 B constant | 0.241 | 0.243 | 0.243 | 0.243 |
| 16 B mixed | 0.300 | 0.301 | 0.301 | 0.300 |
| 64 KiB constant | 97.202 | 97.136 | 97.309 | 97.398 |
| 64 KiB mixed | 17.451 | 17.479 | 17.571 | 17.522 |
| 10 MiB mixed | 2525.774 | 2517.120 | 2512.221 | 2534.970 |
| Reused constant distribution | 0.210 | 0.208 | 0.208 | 0.208 |
| Reused mixed distribution | 1.477 | 1.481 | 1.484 | 1.483 |

At larger input sizes counting dominates. These short-run results should not
be read as evidence that conversion accelerates any workload; use the full
confidence intervals and longer repeated runs to assess small differences.
