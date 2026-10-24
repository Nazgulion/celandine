# Ordered NCD latency and memory baseline

Measured 2026-10-24 on Linux x86-64, Intel Core i7-13700HX, Rust 1.90.0
(`1159e78c4`, LLVM 20.1.8), Criterion 0.7, optimized bench profile without custom
RUSTFLAGS, affinity or governor changes. This measures the initial contiguous
concatenation implementation using miniz_oxide 0.9.1, fixed zlib level 6.

## Workloads and units

There are 45 cases: four shaped pairs at ten sizes, plus five component-only
normalizations. Each pair contains an input and its byte reversal. Each side has
16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760 or 104857600 bytes.
Shapes match prior baselines: constant zeros, cycling `i mod 256`, a one at
indices divisible by 100 with zeros elsewhere, and the high byte of xorshift64
with seed `0x123456789abcdef0` and shifts 13/7/17. Inputs are created outside
measurement. Constant pairs are identical; the others compare ordered reversals.

A scalar call performs three complete fresh compressions and allocates/copies
xy. Throughput counts the original two inputs once: `2*N / 2^20 * 10^9/t` MiB/s
for N bytes per side and latency t nanoseconds. The encoder actually consumes
4*N bytes across its three calls, and concatenation adds copying. Do not compare
this throughput directly with a single compression's byte scan rate.

Byte-pair cases use flat sampling, ten samples, 100 ms warm-up, and a requested
500 ms measurement extended for slower cases. Component-only cases use Criterion's
default sampling selection and no compressor: worked values, negative result,
zero denominator, a one-byte gap at usize::MAX, and a value above one. CSV selects
the fitted slope when available, otherwise the mean, with 95% confidence intervals.
Small-input timings can depend on allocation history; see the previous
[compression baseline](compression.md) for a controlled allocator experiment.

## Memory methodology

The calibrated [memory probe](large-input-memory.md) measures 96 fresh processes:
four shapes, 16 B/64 KiB/10 MiB/100 MiB per side, paired input controls, three
repetitions. Both input buffers are prepared before operation heap accounting;
concatenation and all three compressor initializations are inside it. The probe
computes components and normalizes them once, the work performed by the scalar API.
`support` records C(xy) bytes, and `ngram_length` is unused. The `ncd_input` control
retains the same input and reversal. RSS includes both inputs, stack, allocator
metadata and runtime; requested heap excludes those resident inputs. Stack
high-water usage and cycles/byte were not measured.

## Reproduction and evidence

```sh
python3 scripts/test_measure_memory.py
python3 scripts/benchmark_ncd.py
python3 scripts/measure_memory.py --ncd
```

Timing and memory run sequentially after sanitizer validation, without concurrent
project tests or other measurements. The parent commit is `99625fe`; manifests
record the measured working-tree source hashes, commands, host and compiler.
The earlier baselines remain unchanged. Archived checkout paths use `$REPO`;
original result directories remain under `target/measurements/`.

- [Latency CSV](ncd.csv) and [raw Criterion data](ncd-raw.tar.gz).
- [Memory CSV](ncd-memory.csv) and [raw measurements](ncd-memory-raw.tar.gz).

The memory-tool regression test was observed failing before correcting accidental
NCD-control additions to legacy workloads, then passing afterward. CI runs that
regression test and the small memory smoke check; timing is not a shared-runner
gate. All measurements are bounded evidence on this host, not correctness proofs.

## Results

| Shape | 16 B per side (µs) | 64 KiB per side (ms) | 100 MiB per side (s) |
| --- | ---: | ---: | ---: |
| Constant | 68.125 | 0.30417 | 0.53602 |
| Cycling | 17.125 | 0.31909 | 0.50025 |
| Skewed | 16.028 | 0.42360 | 0.72245 |
| Mixed | 16.359 | 4.02953 | 7.73639 |

Combined-input throughput at 100 MiB per side is approximately 373, 400, 277,
and 26 MiB/s respectively. These are shape-dependent encoder costs, not a
universal NCD latency. Component-only normalization takes 1.41–1.78 ns in these
five cases and allocates no memory, as also checked by isolated allocation tests.
The slower first tiny constant case is consistent with the allocator sensitivity
already recorded for compression; this run does not isolate its cause anew.

All NCD operation scopes used 16 allocation calls. Peak requested heap was
`2*N + 253614` bytes: 253,646 at 16 B per side; 384,686 at 64 KiB; 21,225,134 at
10 MiB; and 209,968,814 at 100 MiB. This includes the temporary concatenation plus
one live compressor workspace; the three streams are measured sequentially.
Input controls reported no operation allocations. For 100 MiB per side, process
RSS was 411,648–412,032 KiB for NCD and 206,592–206,976 KiB for paired controls.

C(xy) at 100 MiB per side was 203,705, 813,994, 712,120, and 209,748,241 bytes for
constant, cycling, skewed and mixed pairs. Even long identical constant inputs
need additional output when doubled; real zlib does not satisfy ideal compressor
idempotence. Use the returned components to assess the compressor on the intended
workload. The concatenation buffer and finite-window limitations are explicit
tradeoffs of this first increment; no symmetry or identity correction is hidden.
