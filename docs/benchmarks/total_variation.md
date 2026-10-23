# Total variation performance and memory baseline

Measured 2026-09-20 on Linux x86-64, Intel Core i7-13700HX, Rust 1.90.0
(`1159e78c4`, LLVM 20.1.8), Criterion 0.7. Default optimized Cargo bench profile;
no custom RUSTFLAGS, CPU affinity, or governor changes. These are initial
measurements, not an optimization comparison or a cross-machine guarantee.

## Workloads and units

Forty byte cases cover four paired shapes at 16, 64, 256, 1024, 4096, 16384,
65536, 1048576, 10485760, and 104857600 bytes **per input**. Each pair has equal
length for this timing baseline; unequal lengths are covered by correctness
and memory checks. Buffers are built outside the timed loop.

- `equal`: both buffers contain only byte zero.
- `disjoint`: left is byte zero, right is byte one.
- `skewed`: left has byte one every 100 positions; right every two positions.
- `mixed`: independent fixed-seed xorshift64 streams with shifts 13, 7, 17 and
  seeds `0x123456789abcdef0` and `0xfedcba9876543210`, taking the top byte.

Five reuse cases compare prebuilt distributions: four shapes built from 64 KiB
samples plus `(usize::MAX-1,1)` versus `(usize::MAX-2,1)` imported counts.
The reuse cases measure only the 256-label comparison, with no input recounting.

Each case uses 20 samples, 300 ms warm-up, and a requested one-second measurement
period. Criterion can extend that period for slow cases. Reported latency is
slope when available, otherwise mean; the CSV retains 95% confidence intervals.
Combined throughput is `(len(x)+len(y))/2^20 * 10^9/latency_ns`, in MiB/s.
For example, two 100 MiB inputs compared in 50 ms mean `200/0.050 = 4000 MiB/s`.
This counts both input scans; it must not be interpreted as a one-buffer rate.

## Observed latency

| Pair shape | 16 B each (ns) | 1 MiB each (ms) | 100 MiB each (ms) | 100 MiB combined throughput (MiB/s) |
| --- | ---: | ---: | ---: | ---: |
| Equal constant | 260.461 | 3.148 | 307.893 | 649.6 |
| Disjoint constant | 258.921 | 3.138 | 315.166 | 634.6 |
| Skewed | 247.641 | 2.366 | 238.295 | 839.3 |
| Mixed | 231.842 | 0.535 | 51.895 | 3853.9 |

Reusing distributions took 185.454 ns (equal), 186.413 ns (disjoint),
184.371 ns (skewed), 189.404 ns (mixed), and 184.741 ns (tiny difference).
Constant inputs repeatedly update the same histogram counter; input shape
substantially affects counting time. This baseline does not isolate CPU causes
or establish a performance target. No other project benchmarks, tests, or fuzz
campaigns ran concurrently with the timing or memory measurements.

## Heap and process memory

The isolated, calibrated [memory probe](large-input-memory.md) measured TV and
input-only controls for all four existing memory shapes (`constant`, cycling
`uniform`, `skewed`, `mixed`) at 10/100 MiB. Three fresh processes per case give
48 readings. TV compares the buffer against its suffix starting at index one,
and measures both the byte and reuse APIs in one allocation scope. Prebuilt
reuse histograms and the input buffer are outside that scope. Both views share
one backing buffer; RSS therefore does **not** measure two separately allocated
100 MiB samples. The n-gram-length CSV column is unused for TV.

Every TV reading recorded **zero allocation calls and zero peak requested heap
bytes**. At 100 MiB, process RSS was 104256–104448 KiB for TV and
104064–104448 KiB for input controls. This includes the resident input, process
runtime, allocator metadata, and stack. It is not an auxiliary-memory estimate;
subtracting independently measured RSS values would not yield exact stack use.
The byte algorithm holds two fixed histograms (2056 bytes each on this host),
plus scalar state; compiler stack layout was not measured. Allocation tests also
cover empty errors and reusable distributions in debug and release builds.

## Reproduce and inspect

```sh
python3 scripts/benchmark_total_variation.py
python3 scripts/measure_memory.py --total-variation
cargo test --locked --release --test allocations
```

Run latency and memory commands sequentially on an otherwise quiet machine.
Each writes a fresh directory under `target/measurements/`, with commands,
compiler/host details, parent commit, source SHA-256 hashes, and raw results.
The current uncommitted sources are identified by those hashes; the recorded
parent commit alone does not contain the new implementation.

- [Latency CSV](total-variation.csv): 45 estimates with intervals and throughput.
- [Raw Criterion evidence](total-variation-raw.tar.gz): samples, estimates,
  benchmark descriptors, outlier summaries, log, manifest, and CSV.
- [Memory CSV](total-variation-memory.csv): 48 individual readings.
- [Raw memory evidence](total-variation-memory-raw.tar.gz): manifest, commands,
  environment, and CSV. `$REPO` replaces the absolute checkout path in archives.

CI compiles the benchmark and runs a small memory-tool smoke check, including
TV. Shared-runner timing is not used as a performance gate. These measurements
support the documented bounded workloads; they do not establish arbitrary
platform performance or exact peak stack usage.
