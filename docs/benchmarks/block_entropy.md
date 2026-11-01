# Overlapping block-entropy baseline

Measured 2026-11-01 on Linux x86-64, Intel Core i7-13700HX, Rust 1.90.0
(`1159e78c4`, LLVM 20.1.8), Criterion 0.7, optimized bench profile without custom
RUSTFLAGS, affinity or governor changes. This is the first baseline.

## Workloads and timing

All ten sizes (16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760,
104857600 bytes) cover n=1 for four shapes. Shapes are constant zero, cycling
`i mod 256` (labelled uniform), one at indices divisible by 100 with zeros
elsewhere (skewed), and the high byte of xorshift64 with seed
`0x123456789abcdef0`, shifts 13/7/17 (mixed). Cycling is uniform over bytes only
when length is a multiple of 256; its overlapping block counts can be unequal
at the boundary. Inputs are generated outside timing.

Additional n=2 and n=8 cases cover all shapes at 16 B, 256 B, 4 KiB and 64 KiB.
The 1/10/100 MiB n=2 tier uses constant and cycling inputs, bounding support to
1/256. Constant and mixed 64 KiB cases at n=128 and n=4096 expose long block
comparison costs. Diverse high-order large tables are deliberately bounded;
100 MiB diversity is not claimed as measured for n>1.

The 82 byte cases include counting, entropy and table destruction. Four reuse
cases construct n=8 tables from 64 KiB inputs outside timing, then call only
the distribution API. Two edge cases use 64 KiB input at full length and
`usize::MAX`. Reuse/edge throughput is blank: they do not scan the input bytes.

Byte cases use flat sampling; others use Criterion's default selection. All
request ten samples, 100 ms warm-up and 500 ms measurement, extended for slow
calls. CSV chooses a fitted slope when available, otherwise a mean, with 95%
confidence intervals. Throughput is `L/2^20 * 10^9/t` MiB/s for input length L
bytes and latency t nanoseconds. These timings are host-specific observations,
not correctness evidence or CI performance gates.

## Memory

The calibrated [probe](large-input-memory.md) uses three fresh processes per
case: 108 readings total, including paired resident-input controls. It measures
n=1 for all shapes at 10/100 MiB, n=2 for constant/cycling at 10/100 MiB, and
n=8 construction plus reuse for mixed inputs at 64 KiB/1 MiB. Empty, full-length,
and oversized cases are included. Operation heap counts requested allocations
only; input generation is outside accounting. For reuse, table construction is
also outside operation accounting but contributes to process RSS.

The byte operation calls the actual public API and records **0 as an unreported
support field**, avoiding an extra counting pass. This does not mean the input
has zero support. Reuse rows report actual support. `ngram_length` records n.
RSS includes input, tables, runtime, stack and allocator metadata. Requested
heap is not allocator-resident memory; stack high-water and cycles/byte were
not measured. The single-byte and boundary paths and all distribution reuse
also have isolated debug/release allocation assertions.

## Reproduce and inspect

```sh
python3 scripts/benchmark_block_entropy.py
python3 scripts/measure_memory.py --block-entropy
```

Run sequentially after validation and fuzzing. Parent commit: `2b5c378`;
manifest SHA-256 values identify the measured working-tree sources. Archived
checkout paths are normalized to `$REPO`.

- [Latency CSV](block_entropy.csv) and [raw samples](block_entropy-raw.tar.gz).
- [Memory CSV](block_entropy-memory.csv) and [raw readings](block_entropy-memory-raw.tar.gz).

## Results

Byte API latency in microseconds (construction and destruction included):

| Input and n | Constant | Cycling | Skewed | Mixed |
| --- | ---: | ---: | ---: | ---: |
| 16 B, n=2 | 0.048 | 0.371 | 0.084 | 0.365 |
| 65,536 B, n=1 | 99.241 | 14.146 | 104.701 | 16.445 |
| 65,536 B, n=2 | 157.833 | 1484.517 | 178.892 | 8502.564 |
| 65,536 B, n=8 | 158.932 | 1544.722 | 224.080 | 9856.042 |
| 104,857,600 B, n=1 | 163604.148 | 19260.183 | 155624.830 | 27580.157 |
| 104,857,600 B, n=2 | 244746.432 | 2257433.686 | not measured | not measured |

Reused n=8 distributions from 64 KiB input, nanoseconds per call:

| Shape | Latency ns |
| --- | ---: |
| constant | 1.266 |
| mixed | 573775.723 |
| skewed | 72.765 |
| uniform | 2136.441 |

Constant blocks share one outcome, so reused entropy returns immediately. Mixed
inputs have many distinct blocks: counting dominates the byte API, while reuse
traverses the existing table. Long-prefix comparisons remain visible: constant
64 KiB at n=4096 took 2840.493 microseconds per call.

Requested operation heap and allocation calls (identical across three repetitions):

| Operation / input | n | Heap bytes | Allocation calls |
| --- | ---: | ---: | ---: |
| block_entropy, constant, 104,857,600 B | 2 | 280 | 1 |
| block_entropy, uniform, 104,857,600 B | 2 | 12,336 | 42 |
| block_entropy, mixed, 65,536 B | 8 | 2,560,832 | 8,792 |
| block_entropy, mixed, 1,048,576 B | 8 | 40,760,984 | 139,937 |
| block_entropy_reuse, mixed, 1,048,576 B | 8 | 0 | 0 |

All n=1, empty/full/oversized and reuse calls reported zero operation heap and
zero allocation calls. At 100 MiB, metric RSS ranged from 104,256 to 104,448 KiB;
input controls ranged from 104,064 to 104,448 KiB. RSS includes resident input
and runtime overhead; it must not be read as operation heap. The raw tables and
confidence intervals retain variability for future comparisons.
