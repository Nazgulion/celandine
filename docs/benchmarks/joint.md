# Joint byte laws and entropy: initial baseline

Measured 2026-11-05 on Linux x86-64, Intel Core i7-13700HX, Rust 1.90.0
(`1159e78c4`, LLVM 20.1.8), Criterion 0.7, optimized bench profile, without
custom RUSTFLAGS, CPU affinity or governor changes.

## Workloads and timing

The shared paired-input generator creates two slices of N bytes each:

- Constant: (0,0) at every position.
- Uniform: `(i mod 256, floor(i/256) mod 256)`, cycling through all 65,536 pairs.
- Skewed: `(i divisible by 100, i divisible by 127)`, each coordinate encoded as 0/1.
- Mixed: the top two bytes of each xorshift64 state, seed `0x123456789abcdef0`,
  shifts 13/7/17. These are reproducible diverse bytes, not a certified random source.

Inputs are generated outside timing. Forty joint-entropy cases cover all shapes
at N=16, 64, 256, 1024, 4096, 16384, 65536, 1048576, 10485760 and 104857600.
Each includes tree construction, entropy and destruction. Eight count-only cases
cover every shape at 16 B/64 KiB per input, including tree destruction.
Eight reuse cases prebuild 64 KiB-per-input laws outside timing: four entropy
calculations and four operations returning both marginals. Two edge cases
measure empty/empty and rejected unequal lengths. There are 58 latency cases.

Construction cases use flat sampling; other groups use Criterion's default
selection. All request ten samples, 100 ms warm-up and 500 ms measurement,
extended for slow calls. CSV chooses fitted slope where available, otherwise
mean, with 95% confidence intervals. `bytes_per_input` stores N, while throughput
is **combined input bytes**, `2*N/2^20 * 10^9/t` MiB/s for latency t nanoseconds.
Reuse/edge throughput is blank because these operations do not scan the inputs.
Results describe this host and are not CI gates.

## Memory

The calibrated [probe](large-input-memory.md) runs 255 fresh processes: three
repetitions of five operations across four shapes at 16 B, 64 KiB, 10 MiB and
100 MiB **per input**, plus five empty cases. `joint_input` is a paired resident
input control; `joint_counts` constructs/drops the table; `joint_entropy`
constructs, evaluates and drops it. `joint_reuse` and `joint_marginals` build
a law before accounting, then measure entropy or both marginal queries.

CSV `input_bytes` stores N, so resident input buffers contain 2N bytes.
`ngram_length=1` is an unused placeholder. Count/reuse/marginal rows report
observed pair support. Input and direct entropy rows report zero in that column;
for direct entropy it means **support not reported**, avoiding a second tally.

Operation heap excludes input allocation. Reuse excludes the prebuilt table
from operation accounting, but that table contributes to process RSS. Input
pages are written and both buffers read before measurement. Requested heap is
not allocator-resident memory: RSS also includes buffers, runtime, stack and
allocator metadata. Stack high-water usage and cycles/byte were not measured.
No dense 65,536-count stack array is used. Each marginal uses fixed 256-count
storage and accumulates exact integers. Isolated allocation tests separately
check empty/error paths, ownership transfer, queries, marginals and entropy reuse.

## Reproduce and inspect

```sh
python3 scripts/benchmark_joint.py
python3 scripts/measure_memory.py --joint
```

Run sequentially after validation and fuzzing. Parent commit: `af4e1cd`;
manifest SHA-256 values identify the measured working-tree sources. Archives
normalize checkout paths to `$REPO`.

- [Latency CSV](joint.csv) and [raw samples](joint-raw.tar.gz).
- [Memory CSV](joint-memory.csv) and [raw readings](joint-memory-raw.tar.gz).

## Results

Joint entropy latency in microseconds, including table construction and destruction:

| Bytes per input | Constant | Uniform pairs | Skewed | Mixed |
| --- | ---: | ---: | ---: | ---: |
| 16 | 0.054 | 0.361 | 0.086 | 0.359 |
| 65,536 | 123.186 | 4914.151 | 133.188 | 5352.585 |
| 1,048,576 | 1961.523 | 53604.054 | 2122.946 | 72567.620 |
| 104,857,600 | 196731.789 | 5184692.947 | 215166.278 | 7069497.423 |

Reuse of a law built from 64 KiB per input, nanoseconds per call:

| Shape | Joint entropy | Both marginals |
| --- | ---: | ---: |
| constant | 1.007 | 175.731 |
| uniform | 504505.705 | 267200.469 |
| skewed | 31.253 | 193.485 |
| mixed | 326292.080 | 219289.438 |

Empty/empty and unequal-length calls took 8.304 and 7.456 ns respectively. These tiny timings include harness overhead.

The initial sparse tree keeps small supports small and avoids a large dense
stack table, but building high-support laws dominates latency. Reuse avoids
that input pass; entropy and marginal queries still traverse observed pairs.
No claim is made that this is the fastest representation for dense byte-pair
laws. This baseline makes future representation changes measurable.

Requested operation heap and allocation calls, identical across three repetitions:

| Operation / shape | Bytes per input | Pair support | Heap bytes | Allocation calls |
| --- | ---: | ---: | ---: | ---: |
| joint_counts / constant | 104,857,600 | 1 | 128 | 1 |
| joint_counts / uniform | 104,857,600 | 65,536 | 1,508,288 | 10,699 |
| joint_counts / skewed | 104,857,600 | 4 | 128 | 1 |
| joint_counts / mixed | 104,857,600 | 65,536 | 1,221,280 | 8,768 |
| joint_reuse / mixed | 104,857,600 | 65,536 | 0 | 0 |
| joint_marginals / mixed | 104,857,600 | 65,536 | 0 | 0 |

Direct entropy reported the same peak heap and allocation calls as count
construction for each input. Every reuse/marginal, empty and input-control
operation reported zero additional heap and zero allocation calls.
With 100 MiB per input, metric operations had RSS 206,400–208,512 KiB.
With 100 MiB per input, paired input controls had RSS 206,592–206,784 KiB.
RSS includes both resident input buffers and process overhead, plus any
prebuilt law. Raw confidence intervals, samples and readings are retained for
future comparisons.
