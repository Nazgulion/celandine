# N-gram baseline

## Reproduction

```sh
cargo bench --locked --bench ngrams
cargo test --locked --release --test allocations
```

The initial scalar implementation uses borrowed slice windows and sorted
`BTreeMap` counts. This baseline precedes any specialized optimization; the
existing entropy baselines remain unchanged.

The harness measures 112 cases:

- Extraction: length four, at 16 B, 64 B, 256 B, 1 KiB, 4 KiB, 16 KiB,
  64 KiB, 1 MiB, 10 MiB, and 100 MiB (ten cases).
- Count construction: lengths 1, 2, and 8, four input shapes, and the first seven
  sizes above (84 cases). Constant and cycling inputs additionally cover
  1 MiB, 10 MiB, and 100 MiB at length two (six cases).
- Long blocks: lengths 128 and 4096 on 64 KiB constant and mixed inputs
  (four cases), exposing long common-prefix comparison costs.
- Probability construction and reused probability iteration: four shapes,
  length four, 64 KiB input (eight cases).

The four shapes match earlier entropy baselines: constant zero bytes, cycling
bytes `i mod 256` (called uniform), one byte equal to one per hundred positions
with zeros elsewhere (skewed), and fixed-seed xorshift64 mixed bytes. The seed
is `0x123456789abcdef0`, shifts are 13, 7, 17, and the high byte is selected.
Mixed data is deterministic benchmark input, not an assumed source model.
Large count cases deliberately use bounded support (one or 256 distinct blocks);
mixed high-order tables can require storage proportional to input length and
are measured only through 64 KiB. This does not establish large mixed-table
throughput or a memory bound independent of input.

Criterion uses 10 samples, 100 ms warmup, and at least 500 ms measurement per
case, increasing duration when slow workloads require it. This broad initial
survey uses fewer samples than the earlier entropy baselines. Input generation
is outside timing. Inputs, parameters, and outputs pass through `black_box`.
Construction timings include map allocation, counting, and destruction.
Probability construction builds the count-backed distribution; reuse timings
visit every stored pair and compute its probability, without recounting input.

Extraction visits and black-boxes every borrowed slice. It does not read every
byte in every slice: reported input MiB/s is a **nominal iterator workload
rate**, not measured memory bandwidth. Count throughput also divides original
input bytes by elapsed time, not the potentially much larger number of byte
comparisons. Reuse measurements report latency only because they scan support,
not the original sequence. Binary units are used: `1 MiB = 1048576 B`.

## Environment and interpretation

- Run date: 2026-09-20.
- CPU: Intel Core i7-13700HX; x86_64 Linux 6.8.0-134-generic.
- Toolchain: rustc 1.90.0 (`1159e78c4`), LLVM 20.1.8; Criterion 0.7.0.
- Cargo optimized benchmark profile; no custom compiler flags, CPU pinning,
  or controlled frequency policy supplied. No other project tests or benchmarks
  were run concurrently with the measurement.

Frequency scaling, heterogeneous cores, scheduling, cache residency, and other
system activity affect these local measurements. Small differences are not
evidence of a speedup. Counting length-one blocks through the tree API is not a
replacement for the specialized, allocation-free `ByteHistogram`.

Allocation checks independently verify zero allocations for extraction from
0 B through 10 MiB, invalid/oversized construction, moving counts into a
distribution, and table queries/iteration. Nonempty count construction allocates
`O(k)` tree storage for `k` distinct borrowed blocks; block contents are not
copied. Peak process memory, stack usage, and cycles per byte are not measured.

Local Criterion reports are in `target/criterion/ngrams_*`. The retained
[CSV](ngrams-baseline.csv) contains point estimates, 95% confidence intervals,
and nominal input throughput. Estimates use the regression slope when available,
otherwise the mean, matching Criterion's reported latency. For a more precise
comparison, preserve this baseline and use identical workloads with longer runs:

```sh
cargo bench --locked --bench ngrams -- --sample-size 100 --measurement-time 5 --warm-up-time 1
```

## Measured examples

Each construction latency includes destroying the resulting table.

| Workload | Latency (µs) |
| --- | ---: |
| Extract all length-4 blocks, 64 KiB | 13.805 |
| Count length-2 blocks, 64 KiB constant | 222.308 |
| Count length-2 blocks, 64 KiB cycling | 1648.914 |
| Count length-2 blocks, 64 KiB mixed | 8319.367 |
| Count length-8 blocks, 64 KiB mixed | 9390.653 |
| Count length-4096 blocks, 64 KiB constant | 2914.265 |
| Construct length-4 probabilities, 64 KiB mixed | 9330.634 |
| Iterate reused length-4 probabilities, 64 KiB mixed | 169.482 |
| Count length-2 blocks, 100 MiB cycling | 2430362.192 |

These are initial measurements, not a comparison with an earlier n-gram
implementation. Full workloads and confidence intervals are in the CSV.
