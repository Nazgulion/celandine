# Fixed-block latency and memory baseline

Measured 2026-10-24 on Linux x86-64, Intel Core i7-13700HX, Rust 1.90.0
(`1159e78c4`, LLVM 20.1.8), Criterion 0.7, optimized bench profile without custom
RUSTFLAGS, affinity or governor changes. The implementation validates size then
uses the standard exact-chunk iterator. Inputs are prepared outside timing and
operation heap accounting.

## Workloads and units

Forty traversal cases use widths 1, 3, 8 and 64 bytes. Cycling `i mod 256` inputs
have 16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760 or 104857600 bytes.
Every yielded slice is passed to `black_box`; contents are not scanned. Work
scales with K = floor(L/b) complete blocks for L bytes and width b. Width three
exercises incomplete tails; width 64 at 16 B yields zero blocks.

CSV throughput counts input extent once: `L / 2^20 * 10^9/t` MiB/s for latency t
nanoseconds. It is not read bandwidth or the throughput of a metric applied to
block contents. In particular the zero-block case measures construction and
exhaustion only, while still using its input extent in the CSV rate.

Fifteen access cases measure construction plus `nth(usize::MAX)`,
`nth_back(usize::MAX)`, `last`, `count`, or `remainder`, with width three at 16 B,
64 KiB and 100 MiB. Data, width and results pass through `black_box`; method
selection is inside the closure. These cases have no byte throughput because
none scans the input.

Traversal uses flat sampling; access uses Criterion's default sampling selection.
Both request ten samples, 100 ms warm-up and 500 ms measurement, extended for slow
cases. CSV selects fitted slope when available, otherwise mean, with 95%
confidence intervals. This is a host-specific baseline, not a performance gate.

## Memory methodology

The calibrated [memory probe](large-input-memory.md) measures 96 fresh processes:
cycling input at 16 B, 64 KiB, 10 MiB and 100 MiB, four widths, paired input
controls, three repetitions. `ngram_length` stores width; `support` stores complete
block count. The operation inspects the original remainder and visits each slice,
without reading the contents. Input is excluded from operation heap. RSS includes
resident input, runtime, stack and allocator metadata. Stack high-water usage
and cycles/block were not measured.

## Reproduction and retained evidence

```sh
python3 scripts/benchmark_blocks.py
python3 scripts/measure_memory.py --blocks
```

Run sequentially after correctness and fuzz checks. The parent commit is
`168fdab`; manifests identify measured working-tree sources by SHA-256.
Raw archives normalize checkout paths to `$REPO`, retaining commands and host data.

- [Latency CSV](blocks.csv) and [raw Criterion data](blocks-raw.tar.gz).
- [Memory CSV](blocks-memory.csv) and [raw memory readings](blocks-memory-raw.tar.gz).

## Results

Traversal latency in microseconds:

| Input | Width 1 | Width 3 | Width 8 | Width 64 |
| --- | ---: | ---: | ---: | ---: |
| 16 B | 0.005 | 0.004 | 0.002 | 0.001 |
| 64 KiB | 14.944 | 4.976 | 1.877 | 0.242 |
| 10 MiB | 2397.345 | 800.441 | 300.708 | 37.327 |
| 100 MiB | 23963.606 | 8019.077 | 3040.607 | 376.952 |

Direct access across all three sizes took 2.45–2.61 ns for maximum skips,
2.62–2.63 ns for count, 2.03 ns for last, and 1.82–1.86 ns for remainder. These
include construction and benchmark dispatch; they agree with constant-time
access. Traversal follows complete block count. No timing of this transform
includes computing entropy or reading every byte in a block.

All 48 block runs and 48 controls reported **zero operation heap bytes
and zero allocation calls**, including the zero-block case. Debug/release
allocation tests separately cover invalid size, tail access, clones and skips.
At 100 MiB, block RSS was 104,256–104,640 KiB;
input-control RSS was 104,064–104,448 KiB.
The resident input dominates RSS; the small process variation is not
evidence of an extraction allocation.
