# Normalized Jensen–Shannon statistical-complexity baseline

Measured 2026-11-01 on Linux x86-64, Intel Core i7-13700HX, Rust 1.90.0
(`1159e78c4`, LLVM 20.1.8), Criterion 0.7, optimized bench profile without custom
RUSTFLAGS, affinity or governor changes. This is the first baseline, including
stable near-certainty entropy and count-based JS comparison.

## Workloads

Forty byte cases fix K=256 and cover four shapes at 16, 64, 256, 1024, 4096,
16384, 65536, 1048576, 10485760 and 104857600 bytes. Shapes are constant zero,
cycling `i mod 256` (labelled uniform), one at indices divisible by 100 with
zeros elsewhere (skewed), and the high byte of xorshift64 with seed
`0x123456789abcdef0`, shifts 13/7/17 (mixed). Input generation is outside timing;
histogram construction and metric evaluation are inside. Cycling inputs below
256 bytes are uniform only over their observed support, not the full alphabet.

Eight distribution cases omit histogram construction: the same four shapes
at 64 KiB with K=256, plus counts `[usize::MAX-1,1]`,
`[usize::MAX/2,usize::MAX/2+1]`, and `[3,1]` with K=2, and `[1,1]` with K=4.
These cover near certainty, near uniformity, the worked example and partial
support. Distribution throughput is intentionally blank.

Byte cases use flat sampling; reuse cases use Criterion's default selection.
All request ten samples, 100 ms warm-up and 500 ms measurement (extended for
slow calls). CSV selects fitted slope where available, otherwise mean, with
95% confidence intervals. Byte throughput is `L/2^20 * 10^9/t` MiB/s for L input
bytes and latency t nanoseconds. Timings describe this host, not a CI gate.

## Memory

The calibrated [probe](large-input-memory.md) uses 96 fresh processes: four
shapes at 16 B, 64 KiB, 10 MiB and 100 MiB, paired input controls, three
repetitions. K=256 throughout. Input is resident before operation accounting;
histogram construction, uniform-reference construction and distribution
calculation are inside. `support` stores observed support; `ngram_length` is
unused. RSS includes the input, runtime, stack and allocator metadata, whereas
operation heap excludes the input. Stack high-water use and cycles/byte were
not measured.

## Reproduce and inspect

```sh
python3 scripts/benchmark_statistical_complexity.py
python3 scripts/measure_memory.py --statistical-complexity
```

Run sequentially after tests and fuzzing. Parent commit: `e291330`; manifests
identify measured working-tree sources by SHA-256. Archives normalize checkout
paths to `$REPO` and preserve host/toolchain information and commands.

- [Latency CSV](statistical_complexity.csv) and [raw data](statistical_complexity-raw.tar.gz).
- [Memory CSV](statistical_complexity-memory.csv) and [raw readings](statistical_complexity-memory-raw.tar.gz).

## Results

Byte API latency in microseconds, K=256:

| Input | Constant | Cycling | Skewed | Mixed |
| --- | ---: | ---: | ---: | ---: |
| 16 B | 0.078 | 2.697 | 2.414 | 2.847 |
| 64 KiB | 97.241 | 13.564 | 98.220 | 22.537 |
| 10 MiB | 15738.143 | 1888.039 | 15431.376 | 2453.223 |
| 100 MiB | 156864.442 | 19402.367 | 155612.743 | 25167.052 |

Reused distributions (nanoseconds per call):

| Law | Latency ns |
| --- | ---: |
| constant | 46.280 |
| mixed | 5479.194 |
| near_uniform | 546.267 |
| partial | 589.968 |
| rare | 554.885 |
| skewed | 2383.697 |
| uniform | 2087.889 |
| worked | 570.252 |

Long-input latency is dominated by histogram counting. Constant and skewed
inputs repeatedly increment the same counters; the distribution-only constant
case instead returns immediately after domain/support validation. No timing
implies evidence of temporal structure; this function measures frequencies.

All 48 metric runs and 48 controls reported **zero operation heap bytes
and zero allocation calls**. The debug/release allocation executable also covers
both APIs and all error paths. At 100 MiB, metric RSS was 104,256–104,640 KiB;
input-control RSS was 104,064–104,640 KiB.
RSS includes the resident input; small process-to-process differences do
not indicate operation heap allocation. Raw intervals and samples are retained
for inspecting timing variability.
