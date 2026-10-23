# Subword complexity performance and memory baseline

Measured 2026-10-23 on Linux x86-64, Intel Core i7-13700HX, Rust 1.90.0
(`1159e78c4`, LLVM 20.1.8), Criterion 0.7, using Cargo's default optimized bench
profile with no custom RUSTFLAGS, affinity, or governor changes. This is a
baseline for the simple n-gram-backed implementation, not an optimized suffix
index or a cross-machine comparison.

## Workloads and units

The benchmark has **88 cases**:

- Length-one counts: four shapes at 16, 64, 256, 1024, 4096, 16384, 65536,
  1048576, 10485760, and 104857600 bytes (40 cases).
- Length-two and length-eight counts: four shapes at 256, 4096, and 65536 bytes
  (24 cases). Length two also runs constant/cycling inputs at 10/100 MiB (4).
- Complete profiles, including length zero: four shapes at 16/64/256/1024 bytes
  (16 cases). A further four cases consume lengths zero through eight at 64 KiB.

Shapes match the n-gram baseline: `constant` is zero; `uniform` cycles through
byte values; `skewed` is one every 100 positions and zero elsewhere; `mixed`
uses xorshift64 with seed `0x123456789abcdef0`, shifts 13/7/17, and the high byte.
Buffers are constructed outside timing. Profile cases consume values into a
wrapping checksum without allocating an output vector. The iterator actually
computes every requested entry; simply constructing it is not a profile timing.

Criterion uses flat sampling, 10 samples, 100 ms warm-up, and a requested 500 ms
measurement period, extended for slow cases. CSVs retain mean latency and 95%
confidence intervals. Scalar throughput counts original input bytes once:
`L/2^20 * 10^9/latency_ns` MiB/s, not bytes internally compared by the tree.
Profile cases report latency only because work spans multiple lengths.

## Latency results

| Workload | Constant | Cycling | Skewed | Mixed |
| --- | ---: | ---: | ---: | ---: |
| Scalar n=1, 100 MiB (ms) | 154.213 | 20.242 | 154.390 | 25.690 |
| Scalar n=2, 64 KiB (ms) | 0.148 | 1.593 | 0.160 | 8.302 |
| Scalar n=8, 64 KiB (ms) | 0.149 | 1.548 | 0.215 | 9.464 |
| Full profile, 1 KiB (ms) | 2.877 | 15.610 | 10.759 | 27.507 |
| Profile n=0..8, 64 KiB (ms) | 1.136 | 10.981 | 1.399 | 64.617 |

Length-two counts at 100 MiB took 237.785 ms for constant input and 2322.752 ms
for cycling input. The latter repeatedly searches 256 distinct keys; the
constant case has only one. Shape-dependent histogram dependencies and tree
comparisons make input size alone a poor predictor of runtime.

Complete profiles rebuild a table for each interior length. Their conservative
worst-case bound is `O(L^3 log(L+1))`, with `O(L)` peak temporary storage.
These results justify keeping complete profiles small and selecting explicit
lengths or bounded prefixes for larger inputs. No high-diversity 100 MiB tree
or large complete profile was measured. No concurrent project tests, fuzzing,
or other benchmarks ran during the final latency/memory measurements.

## Memory results

The calibrated [memory probe](large-input-memory.md) uses a resident input and
records requested operation heap independently of fresh-process maximum RSS.
There are **34 workloads with three repetitions each, 102 readings**: all four
shapes at 10/100 MiB for length one and input controls; length two on the two
bounded-support shapes at those sizes; mixed length-eight counts at 64 KiB and
1 MiB with controls; constant/mixed complete profiles at 64 B and 1 KiB with
controls; a mixed 64 KiB prefix through length eight; and a mixed 100 MiB prefix
through length one.

| Workload | Peak requested heap (bytes) | Process RSS range (KiB) |
| --- | ---: | ---: |
| n=1, mixed 100 MiB | 0 | 104064–104256 |
| n=2, constant 100 MiB | 280 | 104064–104256 |
| n=2, cycling 100 MiB | 12336 | 104640–104640 |
| n=8, mixed 64 KiB | 2560832 | 4608–4800 |
| n=8, mixed 1 MiB | 40760984 | 43776–43968 |
| Full profile, constant 1 KiB | 280 | 2112–2112 |
| Full profile, mixed 1 KiB | 40928 | 2112–2112 |
| Profile n=0..8, mixed 64 KiB | 2560832 | 4608–4608 |
| Profile n=0..1, mixed 100 MiB | 0 | 104256–104448 |

Length-one counts and the two-entry prefix made zero heap allocation calls.
Every evaluated interior length drops its table before returning its count;
full-profile peaks therefore reflect a single live table, not their total
allocation traffic. For example, the mixed 1 KiB full profile made 70,450 calls
but peaked at 40,928 requested bytes. Collecting output into a `Vec` would add
caller-owned storage and is excluded here. All measured temporary allocations
were freed within the measurement scope. RSS also includes input, runtime,
allocator overhead, and stack; it is not an exact auxiliary-memory measure.

In profile rows, `ngram_length` is the number of yielded entries and `support`
is their wrapping checksum, not a single support size. Scalar rows use those
fields in their usual meanings. Allocation tests also cover empty/full/oversized
scalar lengths, cloning, and direct/reverse/extreme profile skipping.

## Reproduction and retained evidence

```sh
python3 scripts/benchmark_subword.py
python3 scripts/measure_memory.py --subword
cargo test --locked --release --test allocations
```

Run measurements sequentially on a quiet machine. The parent commit is
`92ca125` (Jensen–Shannon); source hashes identify the measured working-tree
implementation, including the corrected iterator skipping behavior.
An initial pre-fix exploratory timing run remains under ignored local results;
the retained baseline below was measured after the regression fix.

- [Latency CSV](subword.csv) and [raw Criterion record](subword-raw.tar.gz):
  88 results, samples, estimates, outlier data, descriptors, manifest, and log.
- [Memory CSV](subword-memory.csv) and [raw memory record](subword-memory-raw.tar.gz):
  102 readings, source hashes, commands, and host/toolchain information.

Absolute checkout paths in archived text are normalized to `$REPO`. Original
results remain under `target/measurements/`. CI compiles the benchmarks and
checks the small memory probe; shared-runner timings are not performance gates.
Confidence intervals and RSS ranges describe these observations, not universal
performance guarantees or mathematical correctness.
