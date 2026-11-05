# Explicit LZ76 scales: initial performance and memory baseline

Measured 2026-11-05 on Linux x86-64, Intel Core i7-13700HX, Rust 1.90.0
(`1159e78c4`, LLVM 20.1.8), Criterion 0.7, optimized bench profile without custom
RUSTFLAGS, CPU affinity or governor changes. This is an initial baseline, not
an optimization claim against historical measurements.

## Workloads and timing

The 72 latency cases measure four operations separately: phrase density,
asymptotic normalization with K=256, summary construction, and both cached
normalizations from a prebuilt summary. Constant, cycling and skewed shapes
run at 16 B, 4 KiB, 64 KiB, 10 MiB and 100 MiB; diverse mixed inputs stop at
64 KiB because the raw parser has quadratic worst-case cost.

Shapes match the [raw LZ76 baseline](lz76.md): constant zero; cycling i modulo
256; skewed one at indices divisible by 100 and zero otherwise; mixed xorshift64
with seed 0x123456789abcdef0, shifts 13/7/17 and the high byte. Input creation
and the reusable summary are outside timing. Each cached operation evaluates
both c/N and c*log_256(N)/N; scalar timings each compute their own raw count.

All groups use flat sampling, ten samples, 100 ms warm-up and a requested 500 ms
measurement, extended for slow cases. CSV estimates are means with 95%
confidence intervals. Byte API throughput counts the original input once:
bytes/2^20 * 10^9/latency_ns MiB/s. Cached queries do not read the input and
therefore have no throughput value; bytes_per_input records their source size.

| Operation | Constant 16 B (ns) | Constant 100 MiB (ms) | Mixed 64 KiB (ms) |
| --- | ---: | ---: | ---: |
| Phrase density | 9.058 | 43.218 | 557.714 |
| Asymptotic, K=256 | 88.190 | 198.665 | 556.669 |
| Summary construction | 78.512 | 194.195 | 553.147 |

Both cached scales together took **11.735–11.803 ns** across all source shapes
and sizes. These timings exclude the potentially expensive construction and
must not be interpreted as a time to analyze a new input. Normalization does
not accelerate parsing: mixed 64 KiB still takes about 0.55 seconds. On large
periodic inputs, where raw parsing terminates quickly on a reproducible suffix,
the extra histogram pass in the asymptotic wrapper and summary is substantial.
Compare the full confidence intervals before interpreting small differences.

## Memory

The calibrated [probe](large-input-memory.md) recorded **270 fresh-process
readings**: 90 cases repeated three times. Cases use the same shapes at empty,
16 B, 64 KiB, 10 MiB and 100 MiB, again bounding mixed inputs at 64 KiB. Each
includes an input-only control, density, asymptotic normalization, summary
construction and cached queries. All scopes reported **zero requested heap
bytes and zero allocation calls**. Stack high-water usage was not measured.

For cached queries, summary construction is outside the operation-allocation
scope, but still inside the process measured for RSS. At 100 MiB, combined
operation RSS ranged from 104064 to 104640 KiB; input controls ranged from
104256 to 104640 KiB. RSS includes input residency, runtime, stack and allocator
state, so it is not an exact auxiliary-memory measure. CSV support records
observed support for summaries/reuse and zero (unreported) for scalar operations;
ngram_length is unused. Isolated debug/release tests also cover valid/invalid
alphabets, empty inputs, direct queries and 10 MiB constants.

## Reproduction and retained evidence

```sh
python3 scripts/benchmark_lz76_normalized.py
python3 scripts/measure_memory.py --lz76-normalized
cargo test --locked --release --test allocations
```

Timing and memory ran sequentially, without concurrent project tests, fuzzing
or other benchmarks. Parent commit: `5fe8e31`; source SHA-256 hashes identify
the measured working tree. All current hashes were verified before archiving.

- [Latency CSV](lz76_normalized.csv) and [raw Criterion record](lz76_normalized-raw.tar.gz):
  72 estimates with samples, descriptors, outliers, logs and host/toolchain manifest.
- [Memory CSV](lz76_normalized-memory.csv) and [raw memory record](lz76_normalized-memory-raw.tar.gz):
  270 readings, commands, hashes and host/toolchain metadata.

Archived text replaces the checkout path with `$REPO`. Original outputs remain
under target/measurements. CI compiles benchmarks and checks the memory probe;
it does not enforce machine-specific latency or RSS thresholds. Allocation-free
operation is verified separately from numerical correctness.
