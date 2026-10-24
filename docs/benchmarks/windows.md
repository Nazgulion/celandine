# Sliding-window latency and memory baseline

Measured 2026-10-24 on Linux x86-64, Intel Core i7-13700HX, Rust 1.90.0
(`1159e78c4`, LLVM 20.1.8), Criterion 0.7, optimized bench profile without custom
RUSTFLAGS, affinity or governor changes. This measures the initial bounded ordinal
iterator, with input buffers created outside timing and operation heap accounting.

## Workloads and units

Forty extraction cases use width eight and steps 1, 4, 8 and 16, labelled overlap,
half_overlap, adjacent and gaps. Cycling `i mod 256` inputs have 16, 64, 256, 1024,
4096, 16384, 65536, 1048576, 10485760 or 104857600 bytes. Every yielded slice is
passed to `black_box`; contents are not scanned. Work scales with the number of
windows, `1 + floor((L-8)/step)`, for L input bytes. CSV throughput uses input
extent once: `L / 2^20 * 10^9/t` MiB/s for latency t nanoseconds. This is not
memory-read bandwidth or the throughput of applying a metric to every window.

Twelve access cases measure construction plus `nth(usize::MAX)`,
`nth_back(usize::MAX)`, `last` or `count`, with width three and step two at 16 B,
64 KiB and 100 MiB. Inputs and parameters/results pass through `black_box`;
method selection is part of the benchmark closure. These cases have no byte
throughput: none scans the input. See the [regression record](../fuzzing/windows.md)
for why maximum skips are measured explicitly.

Extraction uses flat sampling; access uses Criterion's default sampling selection.
All cases request ten samples, 100 ms warm-up and 500 ms measurement, extended for
slow cases. CSV selects a fitted slope when available, otherwise the mean, with
95% confidence intervals. Measurements describe this host, not a performance gate.

## Memory methodology

The calibrated [memory probe](large-input-memory.md) measures 96 fresh processes:
cycling inputs at 16 B, 64 KiB, 10 MiB and 100 MiB, four steps, paired input controls,
three repetitions. Width stays eight. Only for these rows, `ngram_length` stores
the step and `support` stores the number of yielded windows. Extraction visits
each slice without reading its contents; the resident input is excluded from
operation heap. RSS includes input, runtime, stack and allocator metadata; it is
not auxiliary heap. Stack high-water usage and cycles/window were not measured.

## Reproduction and retained evidence

```sh
python3 scripts/benchmark_windows.py
python3 scripts/measure_memory.py --windows
```

Run sequentially, after correctness and fuzz checks. The parent commit is
`4d4a015`; manifests identify measured working-tree sources by SHA-256. Raw
archives normalize checkout paths to `$REPO` and include commands and host details.

- [Latency CSV](windows.csv) and [raw Criterion data](windows-raw.tar.gz).
- [Memory CSV](windows-memory.csv) and [raw memory readings](windows-memory-raw.tar.gz).

## Results

Traversal latency in microseconds (width eight):

| Input | Step 1 | Step 4 | Step 8 | Step 16 |
| --- | ---: | ---: | ---: | ---: |
| 16 B | 0.017 | 0.010 | 0.009 | 0.007 |
| 64 KiB | 79.422 | 19.879 | 9.922 | 4.979 |
| 10 MiB | 12670.231 | 3175.910 | 1584.519 | 792.022 |
| 100 MiB | 127692.719 | 31751.543 | 15853.281 | 7917.341 |

Across the three input sizes, maximum skips took 6.24–6.31 ns, count
6.27 ns, and last 8.06–8.07 ns, including construction and benchmark dispatch.
This agrees with bounded constant-time access; the index invariant also ensures
skips never multiply an unbounded caller-supplied index by the step. Full
traversal follows the number of selected windows.

All 48 window runs and 48 controls reported **zero operation heap bytes
and zero allocation calls**. The isolated allocation tests also cover error
paths, cloning, iteration and direct access in debug and release. At 100 MiB,
window RSS was 104,256–104,640 KiB;
input-control RSS was 104,256–104,448 KiB.
The resident input dominates RSS; its small process-to-process variation
should not be interpreted as an extraction allocation.
