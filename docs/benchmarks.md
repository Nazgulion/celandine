# Shannon baseline

## Reproduction

```sh
cargo bench --locked --bench shannon
cargo test --locked --release --test allocations
```

Criterion reports latency and throughput for 16, 64, and 256 B; 1, 4, 16, and
64 KiB; and 1 and 10 MiB. Sizes are binary (`1 KiB = 1024 B`). The benchmark
uses 30 samples, 300 ms warmup, and at least 1 second of measurement per case.
Criterion may extend measurement to obtain enough iterations. Use a longer run
before making close performance comparisons, for example:

```sh
cargo bench --locked --bench shannon -- --sample-size 100 --measurement-time 5 --warm-up-time 1
```

Input generation and heap allocation happen outside the timed loop. Both input
and output pass through `std::hint::black_box`. Every byte benchmark includes
histogram construction and entropy computation. Reusable-distribution cases
measure only entropy over an already constructed histogram of a 64 KiB sample;
they intentionally do not report byte throughput because the input is not rescanned.

Four deterministic input shapes are measured:

- **constant:** every byte is zero;
- **uniform:** byte values cycle from 0 through 255 (short samples use a smaller
  uniform observed support);
- **skewed:** byte 1 at every hundredth position, byte 0 elsewhere;
- **mixed:** the high byte of fixed-seed xorshift64 outputs, with seed
  `0x123456789abcdef0` and shifts 13, 7, 17. This is reproducible test data,
  not a model for an unknown source or a claim about randomness quality.

The single-threaded allocation test has its own executable and global allocator
counter. It excludes input creation and asserts that histogram construction,
checked count import, probability iteration, and both Shannon paths make zero
allocation/reallocation calls. It runs in both debug and release validation.
It does not measure peak stack memory or cycles per byte. A histogram stores
256 `usize` counts and one `usize` total (2056 bytes on this 64-bit target);
`Distribution` wraps exactly that representation.

## Baseline environment

- Run date: 2026-09-17.
- CPU: Intel Core i7-13700HX; x86_64 Linux 6.8.0-134-generic.
- Compiler: rustc 1.90.0 (`1159e78c4`), LLVM 20.1.8.
- Criterion 0.7.0, default Cargo release optimization, no custom target flags.
- One benchmark process; no CPU pinning or controlled frequency policy.

These are local baseline measurements, not portable performance guarantees.
Frequency scaling, scheduling on different core types, cache state, and system
load can affect results. Repeated input buffers can be cache-resident. A lower
entropy does not necessarily imply a faster histogram: repeated updates to one
counter can have different costs from updates spread across counters.

Criterion's raw local reports are under `target/criterion/`. The retained
[baseline CSV](benchmarks/shannon-baseline.csv) records each point estimate and
its 95% confidence interval, with corresponding throughput for byte cases.
The estimate is the regression slope when available, otherwise the sample mean,
matching Criterion's reported typical latency. No optimization was applied
after collecting this baseline.


## Measured byte-path latency

Point estimates in microseconds; all cases include counting. Full precision
and confidence intervals are retained in the CSV.

| Input | Constant (µs) | Uniform (µs) | Skewed (µs) | Mixed (µs) | Mixed throughput (MiB/s) |
| --- | ---: | ---: | ---: | ---: | ---: |
| 16 B | 0.247 | 0.318 | 0.261 | 0.287 | 53.2 |
| 64 B | 0.307 | 0.563 | 0.341 | 0.501 | 121.9 |
| 256 B | 0.532 | 1.569 | 0.570 | 1.092 | 223.7 |
| 1 KiB | 1.555 | 1.695 | 1.701 | 1.715 | 569.3 |
| 4 KiB | 5.909 | 2.209 | 6.350 | 2.337 | 1671.2 |
| 16 KiB | 24.295 | 4.315 | 24.151 | 4.735 | 3300.1 |
| 64 KiB | 98.029 | 13.750 | 97.786 | 14.875 | 4201.6 |
| 1 MiB | 1571.408 | 209.158 | 1570.647 | 268.315 | 3727.0 |
| 10 MiB | 15414.863 | 2125.781 | 15892.088 | 2602.900 | 3841.9 |

## Measured distribution reuse

| Shape | Latency (ns) |
| --- | ---: |
| constant | 216.7 |
| uniform | 1481.0 |
| skewed | 227.0 |
| mixed | 1482.8 |

Allocation checks passed in debug and release: **zero heap allocation calls**
for the checked paths at every tested size from 0 B to 10 MiB.
