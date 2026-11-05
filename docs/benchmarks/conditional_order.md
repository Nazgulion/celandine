# Conditional entropy by order: initial baseline

Measured 2026-11-05 on Linux x86-64, Intel Core i7-13700HX, Rust 1.90.0
(`1159e78c4`, LLVM 20.1.8), Criterion 0.7, optimized bench profile, without
custom RUSTFLAGS, CPU affinity or governor changes.

## Workloads and timing

Order zero covers all four shapes at ten sizes: 16, 64, 256, 1024, 4096,
16384, 65536, 1048576, 10485760 and 104857600 bytes. Shapes are constant zero,
cycling `i mod 256` (labelled uniform), one at indices divisible by 100 with
zeros elsewhere (skewed), and the high byte of xorshift64 with seed
`0x123456789abcdef0`, shifts 13/7/17 (mixed). Inputs are generated outside timing.

Orders 1 and 7 additionally cover every shape at 16 B, 256 B, 4 KiB and 64 KiB.
The 1/10/100 MiB order-one tier uses constant and cycling inputs, bounding
joint support to 1/256. Constant and mixed 64 KiB inputs at orders 127 and 4095
exercise long-prefix comparisons. Diverse higher-order large inputs are bounded;
100 MiB diversity is measured only at order zero. Cycling byte frequencies do
not imply unpredictable transitions: at positive order the cycle is deterministic.

The 82 byte cases include table construction, conditional entropy and destruction.
Eight reuse cases build 64 KiB n-gram tables outside timing, covering four
shapes at orders 1 and 7. Two edge cases use 64 KiB input with one observation
(order L-1) or maximum usize order. Reuse and edge throughput are blank because
they do not scan the source bytes. There are 92 latency cases in total.

Byte cases use flat sampling; other groups use Criterion's default selection.
All request ten samples, 100 ms warm-up and 500 ms measurement, extended for
slow calls. CSV selects fitted slope where available, otherwise mean, with 95%
confidence intervals. Throughput is `L/2^20 * 10^9/t` MiB/s for input size L
bytes and latency t nanoseconds. Results describe this host and are not CI gates.

## Memory

The calibrated [probe](large-input-memory.md) runs 126 fresh processes: three
repetitions per case, including resident-input controls. Order zero covers
all shapes at 10/100 MiB; order one covers constant/cycling there. Mixed inputs
at 64 KiB and 1 MiB cover construction and reuse at orders 1 and 7. Empty,
one-observation and oversized cases are included.

CSV `ngram_length` stores **order + 1**, the joint block length, for both
conditional operations and their controls. The oversized memory case uses
n=usize::MAX (order MAX-1); the latency and normal tests also exercise order MAX.
The byte API reports zero as an **unreported support field**, avoiding another
counting pass. Reuse rows report actual distinct joint-block support.

Operation heap includes counting and entropy for the byte API. Reuse constructs
its table before accounting and measures only the conditional calculation;
that resident table still contributes to process RSS. Input allocation is also
outside operation accounting. RSS includes the input, table, runtime, stack
and allocator metadata; requested heap is not allocator-resident memory.
The fixed 256-count scratch array is 2048 bytes on this host; stack high-water
usage and cycles/byte were not measured. Isolated debug/release tests verify
allocation-free reuse and order-zero/insufficient-observation paths.

## Reproduce and inspect

```sh
python3 scripts/benchmark_conditional_order.py
python3 scripts/measure_memory.py --conditional-order
```

Run sequentially after validation and fuzzing. Parent commit: `e431490`;
manifest SHA-256 values identify the measured working-tree sources. Archives
normalize checkout paths to `$REPO`.

- [Latency CSV](conditional_order.csv) and [raw samples](conditional_order-raw.tar.gz).
- [Memory CSV](conditional_order-memory.csv) and [raw readings](conditional_order-memory-raw.tar.gz).

## Results

Byte API latency in microseconds, including counting and destruction:

| Input and order k | Constant | Cycling | Skewed | Mixed |
| --- | ---: | ---: | ---: | ---: |
| 16 B, k=1 | 0.063 | 0.405 | 0.099 | 0.406 |
| 65,536 B, k=0 | 97.107 | 12.691 | 95.447 | 19.045 |
| 65,536 B, k=1 | 205.717 | 1409.312 | 216.640 | 8915.605 |
| 65,536 B, k=7 | 206.949 | 1464.847 | 259.270 | 10122.719 |
| 104,857,600 B, k=0 | 156904.057 | 20585.209 | 153999.740 | 25454.115 |
| 104,857,600 B, k=1 | 350599.269 | 2301758.689 | not measured | not measured |

Reused distributions from 64 KiB inputs, nanoseconds per call:

| Shape and order | Latency ns |
| --- | ---: |
| constant_k1 | 1.209 |
| constant_k7 | 1.209 |
| mixed_k1 | 419831.721 |
| mixed_k7 | 377575.566 |
| skewed_k1 | 37.142 |
| skewed_k7 | 55.390 |
| uniform_k1 | 960.335 |
| uniform_k7 | 992.875 |

Reuse avoids counting, but still traverses distinct contexts. Constant inputs
return immediately from one joint outcome. Diverse high-order samples can
have zero empirical conditional entropy while still requiring table traversal.
This illustrates sparse-sample limitations, not evidence of source predictability.

Requested operation heap and allocation calls, identical across three repetitions:

| Operation / input | Order k | Heap bytes | Allocation calls |
| --- | ---: | ---: | ---: |
| conditional_order, constant, 104,857,600 B | 1 | 280 | 1 |
| conditional_order, uniform, 104,857,600 B | 1 | 12,336 | 42 |
| conditional_order, mixed, 1,048,576 B | 1 | 2,550,760 | 8,755 |
| conditional_order, mixed, 1,048,576 B | 7 | 40,760,984 | 139,937 |
| conditional_order_reuse, mixed, 1,048,576 B | 7 | 0 | 0 |

All order-zero, empty/one-observation/oversized and reuse operations reported
zero additional heap and zero allocation calls. At 100 MiB, metric RSS ranged
from 104,064 to 104,448 KiB; input controls ranged from 104,064 to 104,640 KiB.
RSS includes resident input and process overhead, not just operation allocations.
Raw intervals and samples are retained for later performance comparisons.
