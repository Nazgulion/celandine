# Aligned conditional entropy: initial performance and memory baseline

Measured 2026-11-05 on Linux x86-64, Intel Core i7-13700HX, Rust 1.90.0
(`1159e78c4`, LLVM 20.1.8), Criterion 0.7, optimized bench profile without custom
RUSTFLAGS, CPU affinity or governor changes. This establishes an initial baseline;
it does not claim an optimization over another implementation.

## Workloads and timing

The shared [paired-input generator](joint.md) creates two N-byte slices:
constant (0,0), cycling `(i mod 256, floor(i/256) mod 256)`, skewed divisibility
indicators for 100/127, or the top two bytes of xorshift64 (seed
0x123456789abcdef0, shifts 13/7/17). Generation is outside timing.

Forty byte API cases use all four shapes at N=16, 64, 256, 1024, 4096, 16384,
65536, 1048576, 10485760 and 104857600. Each includes counting, direct conditional
entropy and table destruction. Four reuse cases prebuild 64 KiB laws outside
timing. Three more use pair counts `(0,0):k, (1,0):1, (2,1):k` with k=64,4096,65536,
so N=2k+1: these combine a nearly certain conditional group with a deterministic
group. A diagonal 64 KiB case tests zero uncertainty with 256 observed pairs.
Empty/empty and unequal-length calls complete the **50 latency cases**.

Byte cases use flat sampling; reuse and edge groups use Criterion's default
selection. All request ten samples, 100 ms warm-up and 500 ms measurement,
extended for slow cases. CSV uses fitted slopes when available, otherwise means,
with 95% confidence intervals. `bytes_per_input` is N; throughput counts both
slices: `2*N/2^20 * 10^9/latency_ns` MiB/s. Cached queries and edge cases have no
throughput value because they do not scan the original two byte slices.

| Shape | Byte API, 16 B/slice (ns) | Byte API, 100 MiB/slice (ms) | Reuse of 64 KiB law (ns) |
| --- | ---: | ---: | ---: |
| Constant | 58.126 | 201.013 | 1.157 |
| Cycling | 489.535 | 5192.016 | 668943.290 |
| Skewed | 160.013 | 219.657 | 131.310 |
| Mixed | 414.169 | 6882.844 | 457038.772 |

Diagonal-law reuse took 1101.969 ns; the three near-deterministic laws took
112.252–113.103 ns. Empty/empty and unequal-length calls took 8.718 and 7.642 ns
respectively. Reuse excludes construction: it must not be described as the time
to analyze a new input. Constant-law reuse returns directly for support size one;
larger laws build a fixed second marginal and visit every observed pair.

The sparse joint-table construction dominates large diverse-input calls. The
64 KiB reuse laws have their own support sizes and count distributions; they
are not interchangeable with the 100 MiB byte workloads. Use the retained
confidence intervals rather than treating small timing differences as universal.

## Heap and process memory

The calibrated [probe](large-input-memory.md) recorded **153 fresh-process
readings**: four shapes at N=16, 65536, 10 MiB and 100 MiB, plus one empty case;
three operations per case and three repetitions. Operations are paired-input
controls, byte API evaluation, and reuse after prebuilding the law.

All reused evaluations reported **zero additional heap bytes and zero allocation
calls**; empty byte calls also allocated nothing. Nonempty byte calls allocate
the joint BTreeMap. At 100 MiB per slice, requested peak operation heap was
128 bytes / 1 allocation for constant and skewed shapes, 1,508,288 bytes /
10,699 allocations for cycling, and 1,221,280 bytes / 8,768 allocations for mixed.
These tree-layout observations depend on this toolchain and workload.

At 100 MiB per slice, operation and reuse process RSS ranged from 206592 to
208512 KiB; paired-input controls ranged from 206400 to 206784 KiB. RSS includes
the two resident inputs, runtime, stack and allocator state; reuse also retains
the prebuilt table outside its operation-allocation scope. Zero additional heap
is not zero total process memory. Stack high-water usage was not measured.
CSV support is the number of distinct observed pairs for reuse and zero (unreported) for the
byte API/controls; ngram_length is unused. Isolated debug/release checks cover
reuse, empty inputs and length errors independently of timing.

## Reproduction and retained evidence

```sh
python3 scripts/benchmark_conditional_pair.py
python3 scripts/measure_memory.py --conditional-pair
cargo test --locked --release --test allocations
```

Timing and memory ran sequentially without concurrent project tests, fuzzing or
other benchmarks. Parent commit: `d2bafa5`; SHA-256 hashes identify the measured
working tree and were verified before archiving.

- [Latency CSV](conditional_pair.csv) and [raw Criterion record](conditional_pair-raw.tar.gz):
  50 estimates, samples, descriptors, outliers, logs and host/toolchain manifest.
- [Memory CSV](conditional_pair-memory.csv) and [raw memory record](conditional_pair-memory-raw.tar.gz):
  153 readings with commands, source hashes and host/toolchain metadata.

Archived text normalizes checkout paths to `$REPO`. Original outputs remain
under target/measurements. CI compiles the benchmark and checks the probe;
shared-runner timing is not a performance gate. Measurements supplement,
but do not establish, mathematical correctness.
