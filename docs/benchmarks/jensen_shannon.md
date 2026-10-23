# Jensen–Shannon performance and memory baseline

Measured 2026-10-23 on Linux x86-64, Intel Core i7-13700HX, Rust 1.90.0
(`1159e78c4`, LLVM 20.1.8), Criterion 0.7. Cargo's default optimized bench
profile was used without custom RUSTFLAGS, affinity, or governor changes.
This is an initial baseline, not an optimization or cross-machine comparison.

## Workloads and units

Each of divergence and distance has 40 byte cases: four paired shapes at
16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760, and 104857600 bytes
per input. `equal` is two zero-filled buffers; `disjoint` is zero versus one;
`skewed` has ones every 100 positions on the left and every two on the right.
`mixed` uses independent xorshift64 streams, shifts 13/7/17, seeds
`0x123456789abcdef0` and `0xfedcba9876543210`, and the high byte of each state.

Each metric also has five reuse cases: prebuilt distributions from the four
64 KiB shapes, plus imported counts `(usize::MAX-1,1)` versus
`(usize::MAX-2,1)`. Buffers and reused histograms are constructed outside timing.
This gives **90 cases**. Separate APIs are measured; timings include their
normal validation and `Result` handling.

Settings are 20 samples, 300 ms warm-up, and a requested one-second measurement
period, extended by Criterion for slow cases. CSV latency is slope where
available, otherwise mean, with 95% confidence intervals. Throughput counts
both inputs: `(len(x)+len(y))/2^20 * 10^9/latency_ns` MiB/s. Two 100 MiB inputs
in 50 ms correspond to `200/0.050 = 4000 MiB/s`. Reuse cases report latency only.

## Results

| API / shape | 16 B each (ns) | 1 MiB each (ms) | 100 MiB each (ms) | Combined throughput at 100 MiB (MiB/s) |
| --- | ---: | ---: | ---: | ---: |
| Divergence / equal | 305.352 | 3.118 | 316.692 | 631.5 |
| Divergence / disjoint | 305.881 | 3.114 | 317.982 | 629.0 |
| Divergence / skewed | 341.402 | 2.381 | 238.340 | 839.1 |
| Divergence / mixed | 509.599 | 0.505 | 51.572 | 3878.1 |
| Distance / equal | 297.843 | 3.265 | 330.419 | 605.3 |
| Distance / disjoint | 307.953 | 3.132 | 308.511 | 648.3 |
| Distance / skewed | 339.349 | 2.351 | 234.347 | 853.4 |
| Distance / mixed | 494.372 | 0.494 | 51.588 | 3876.8 |

Reused-distribution divergence took 219.173 ns (equal), 235.009 ns (disjoint),
275.807 ns (skewed), 3737.272 ns (mixed), and 244.696 ns (tiny difference).
Distance took 216.282, 231.284, 271.475, 3679.241, and 238.343 ns respectively.
The mixed reuse case evaluates differences across many labels, while other
cases need few or no nonzero contributions. The smaller observed distance times
do not establish that an extra square root speeds up the computation; cases ran
separately and timings vary with machine state. No concurrent project tests,
fuzzing, or other benchmarks ran during these measurements.

## Memory

The calibrated [memory probe](large-input-memory.md) measured all four JS APIs
in one allocation scope, using a resident input and its suffix starting at index
one. Reuse histograms were constructed before the scope. Four memory shapes
(constant, cycling uniform, skewed, mixed), 10/100 MiB inputs, input-only controls,
and three fresh processes per case give **48 readings**.

All JS cases recorded **zero allocation calls and zero peak requested heap
bytes**. At 100 MiB, process RSS was 104256–104448 KiB for JS and
104064–104448 KiB for controls. Both views share one buffer; this RSS does not
represent two separately allocated 100 MiB samples. RSS includes runtime,
allocator metadata, and stack; it is not an exact auxiliary-memory measurement.
Two fixed byte histograms are 2056 bytes each on this host; actual peak stack
layout was not measured. Debug/release allocation tests cover both success and
empty-error paths. The CSV's n-gram-length field is unused for JS.

## Reproduction and retained evidence

```sh
python3 scripts/benchmark_jensen_shannon.py
python3 scripts/measure_memory.py --jensen-shannon
cargo test --locked --release --test allocations
```

Run timing and memory sequentially on a quiet machine. Fresh directories under
`target/measurements/` retain source hashes, parent commit, commands, compiler
and host details, and raw data. The parent commit is the published TV increment;
source SHA-256 hashes identify the new working-tree implementation.

- [Latency CSV](jensen-shannon.csv) and [raw Criterion data](jensen-shannon-raw.tar.gz):
  90 results, samples, descriptors, estimates, outlier data, manifest, and log.
- [Memory CSV](jensen-shannon-memory.csv) and [raw memory record](jensen-shannon-memory-raw.tar.gz):
  48 readings, manifest, commands, and environment.

Absolute checkout paths are normalized to `$REPO` in archived text. CI compiles
the benchmark and exercises the small memory smoke case; timings are not a CI
performance gate. These results cover the specified workloads and host only.
