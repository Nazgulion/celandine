# Optional zlib compression baseline

Measured 2026-10-23 on Linux x86-64, Intel Core i7-13700HX, Rust 1.90.0
(`1159e78c4`, LLVM 20.1.8), Criterion 0.7, optimized bench profile, no custom
RUSTFLAGS, affinity or governor changes. Default allocator policy applies to the
main baseline; the separately labelled allocator experiment changes one setting.

The backend counts complete zlib streams using miniz_oxide 0.9.1 with level 6,
default strategy, a 32 KiB window, and no dictionary. Each call starts fresh and
includes initialization and finalization. This measures one fixed encoder's
cost; it is not an optimization comparison or a benchmark of other codecs.

## Workloads and units

There are **80 latency cases**: compressed length and compression ratio, each
with four shapes at 16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760,
and 104857600 bytes. Input generation is outside timing. Shapes match the
existing suite: constant zero; cycling `i mod 256`; one at indices divisible by
100 and zero otherwise; high-byte xorshift64 with seed `0x123456789abcdef0` and
shifts 13/7/17. Mixed input is deterministic workload data, not a source model.

Criterion uses flat sampling, ten samples, 100 ms warm-up and a requested 500 ms
measurement period, extended when necessary to collect ten samples. CSV values
are means and 95% confidence intervals. Throughput is input bytes divided by
2^20, multiplied by 10^9/latency_ns, in MiB/s. Both APIs compress once; differences
between their timings are not evidence of a different compression algorithm.

## Memory methodology

The calibrated [memory probe](large-input-memory.md) measures **96 fresh-process
readings**: four shapes at 16 B, 64 KiB, 10 MiB and 100 MiB, each with input-only
controls and three repetitions. Requested operation heap excludes the resident
input and includes the compressor's allocations. Encoded output is counted and
discarded through a fixed 32 KiB scratch buffer. Length is the compression work
shared by both APIs; ratio adds integer-to-float conversion and division.
The CSV `support` field records compressed bytes; `ngram_length` is unused.

GNU time reports whole-process maximum RSS in KiB. It includes the resident input,
stack, allocator overhead, runtime and executable pages. Differences between
independent process peaks are not exact auxiliary-memory measurements. Stack
high-water usage and cycles/byte were not measured.

## Reproduction

```sh
python3 scripts/benchmark_compression.py
python3 scripts/measure_memory.py --compression
```

Run latency and memory sequentially, without project tests, fuzzing or other
benchmarks running concurrently. Results below retain host/toolchain information,
commands and source hashes. The parent commit is `f30cd44` (LZ76); hashes identify
the measured working-tree implementation. Archived checkout paths are normalized
to `$REPO`; originals remain under `target/measurements/`.

- [Latency CSV](compression.csv) and [raw Criterion data](compression-raw.tar.gz).
- [Memory CSV](compression-memory.csv) and [raw memory record](compression-memory-raw.tar.gz).

CI compiles both feature configurations and exercises the memory instrument;
shared-runner timings are not performance gates. These are bounded measurements
on one host, not guarantees for other machines or data.

## Results

| API / shape | 16 B (µs) | 64 KiB (ms) | 100 MiB (ms) |
| --- | ---: | ---: | ---: |
| Length / constant | 21.691 | 0.07305 | 110.705 |
| Length / cycling | 5.356 | 0.07718 | 109.889 |
| Length / skewed | 5.521 | 0.10716 | 165.610 |
| Length / mixed | 5.360 | 0.91405 | 1859.825 |
| Ratio / constant | 5.317 | 0.07312 | 110.231 |
| Ratio / cycling | 5.366 | 0.07698 | 110.474 |
| Ratio / skewed | 5.322 | 0.10653 | 172.573 |
| Ratio / mixed | 5.370 | 0.91331 | 1857.535 |

At 100 MiB the length API processes approximately 903, 910, 604, and 54 MiB/s
for constant, cycling, skewed and mixed input respectively. Complete compressed
sizes are 101,864, 407,007, 356,046, and 104,874,126 bytes. The mixed stream is
larger than its input; no overhead is subtracted from that result.

Every compression memory scope reported **253,614 peak requested heap bytes in
five allocation calls**, from 16 B through 100 MiB across all shapes. This is
observed fixed workspace on this build; stack high-water is not included.
At 100 MiB, operation RSS ranged from 104,448 to 104,832 KiB and input controls
from 104,256 to 104,448 KiB. Resident input dominates process memory.

### Small-input allocator sensitivity

The first constant-16-byte length case was noticeably slower than the ratio
case. A separately retained two-case check (20 samples, 300 ms warm-up, one-second
requested measurement) reproduced 21.376 µs for length and 5.362 µs for ratio.
Repeating that check with `MALLOC_MMAP_THRESHOLD_=1048576` gave 21.471 and
21.478 µs respectively. The [glibc allocator documentation](https://sourceware.org/glibc/manual/latest/html_node/Malloc-Tunable-Parameters.html)
explains that this setting fixes an otherwise dynamically adjusted threshold.
The disappearance of the gap is evidence of allocator-policy sensitivity; it
does not establish which individual allocation caused it. Both APIs execute the
same compression path. Do not interpret the default-run gap as an algorithmic
speed advantage of ratio over length. The baseline is preserved unchanged.

The filter still visits the benchmark's input-generation loops for unselected
cases, so allocation history includes those buffers. Reproduce both checks in
fresh processes, with separate `CRITERION_HOME` directories:

```sh
cargo bench --locked --features compression-deflate --bench compression -- \
  'compression_(length|ratio)_zlib/constant/16$' \
  --sample-size 20 --warm-up-time 0.3 --measurement-time 1 --noplot
# Repeat with MALLOC_MMAP_THRESHOLD_=1048576 in the environment.
```

- [Default-policy check CSV](compression-small-check.csv) and [raw data](compression-small-check-raw.tar.gz).
- [Fixed-policy check CSV](compression-allocator-check.csv) and [raw data](compression-allocator-check-raw.tar.gz).
