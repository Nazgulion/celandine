# Block-entropy profiles: initial baseline

Measured 2026-11-05 on Linux x86-64, Intel Core i7-13700HX, Rust 1.90.0
(`1159e78c4`, LLVM 20.1.8), Criterion 0.7, optimized bench profile, without
custom RUSTFLAGS, CPU affinity or governor changes.

## Workloads and timing

Maximum order one covers four shapes at ten sizes: 16, 64, 256, 1024, 4096,
16384, 65536, 1048576, 10485760 and 104857600 bytes. Shapes are constant zero,
cycling `i mod 256` (labelled uniform), one at indices divisible by 100 with
zeros elsewhere (skewed), and the high byte of xorshift64 with seed
`0x123456789abcdef0`, shifts 13/7/17 (mixed). Inputs are generated outside timing.

Maximum orders 2 and 8 additionally cover every shape at 16 B, 256 B, 4 KiB
and 64 KiB. The 1/10/100 MiB maximum-two tier uses constant and cycling inputs,
bounding block support to 1/256. These 78 prefix cases consume all requested
entries into a sum, including counting, entropy and table destruction, without
allocating an output vector. Twelve full-profile cases cover all four shapes
at 16, 64 and 256 bytes, using usize::MAX as the limit.

Eighteen access cases use cycling inputs at 16 B, 64 KiB and 100 MiB, with
six operations: construction, count, last, nth(L-1), nth_back(L-1), and zero
limit. Last and nth select H_L=0 without counting; nth_back selects H_1 and
therefore scans the input. Count consumes a cloned iterator without evaluating
entries. Construction deliberately leaves the iterator unconsumed. The zero
limit case sums an empty profile. These tests distinguish range advancement
from the cost of a selected value.

There are 108 latency cases. Prefix cases use flat sampling; other groups use
Criterion's default selection. All request ten samples, 100 ms warm-up and
500 ms measurement, extended for slow calls. CSV selects fitted slope where
available, otherwise mean, with 95% confidence intervals. Prefix/full throughput
is `L/2^20 * 10^9/t` MiB/s for input size L bytes and latency t nanoseconds:
**logical input bytes per complete profile**, not total bytes scanned across
orders. Access throughput is blank. Results describe this host and are not CI gates.

## Memory

The calibrated [probe](large-input-memory.md) runs 159 fresh processes: three
repetitions per case, including resident-input controls. Maximum one covers all
shapes at 10/100 MiB; maximum two covers constant/cycling there. Mixed inputs
at 64 KiB and 1 MiB use maximum four. Full profiles cover all four shapes at
16/64 bytes. Metadata and selected-entry operations use 100 MiB cycling input
and maximum usize; empty, zero-limit and capped-maximum cases are also included.

For profile operations and their controls, CSV `ngram_length` stores
**max_order**, and `support` stores the number of profile entries for metric
operations (zero for input controls). It is not distinct n-gram support.
`block_profile` consumes the profile into a sum. `block_profile_metadata`
constructs it and queries length, size hint and cloned count.
`block_profile_select` queries cloned last, nth(len-1) and nth_back(len-1):
with the measured full-profile limit, these select only H_L and H_1.

Operation heap includes temporary counting tables but excludes input allocation
and an output vector. Separate tables are dropped between orders, so peak heap
is not the sum of every order's table. Allocation calls are cumulative.
RSS includes resident input, runtime, stack and allocator metadata; requested
heap is not allocator-resident memory. Stack high-water usage and cycles/byte
were not measured. Isolated debug/release tests also verify metadata and skipped
orders allocate nothing through 10 MiB inputs.

## Reproduce and inspect

```sh
python3 scripts/benchmark_block_profile.py
python3 scripts/measure_memory.py --block-profile
```

Run sequentially after validation and fuzzing. Parent commit: `0b48016`;
manifest SHA-256 values identify the measured working-tree sources. Archives
normalize checkout paths to `$REPO`.

- [Latency CSV](block_profile.csv) and [raw samples](block_profile-raw.tar.gz).
- [Memory CSV](block_profile-memory.csv) and [raw readings](block_profile-memory-raw.tar.gz).

## Results

Consumed-prefix latency in microseconds, including counting and destruction:

| Input / maximum | Constant | Cycling | Skewed | Mixed |
| --- | ---: | ---: | ---: | ---: |
| 65,536 B / 1 | 96.796 | 15.005 | 96.484 | 17.390 |
| 65,536 B / 2 | 244.803 | 1542.948 | 256.690 | 8776.038 |
| 65,536 B / 8 | 1143.864 | 10846.699 | 1384.941 | 68917.726 |
| 104,857,600 B / 1 | 156207.992 | 20011.838 | 153565.677 | 25626.479 |
| 104,857,600 B / 2 | 392939.793 | 2356704.420 | not measured | not measured |

Full 256-byte profiles took 95.444 µs (constant), 1410.198 µs (uniform), 784.043 µs (skewed), 1233.064 µs (mixed). These evaluate all 256 entries.

Lazy access on 100 MiB cycling input, nanoseconds per call:

| Operation | Latency ns |
| --- | ---: |
| construct | 4.436 |
| count | 5.847 |
| last | 6.852 |
| nth | 8.021 |
| nth_back | 19815371.433 |
| zero_limit | 4.151 |

Construction/count and selecting H_L avoid intermediate computations. Reverse
skipping to H_1 still scans 100 MiB; this is the selected value's cost, not
work on discarded entries. Tiny timings include the harness and black boxes.

Requested operation heap and allocation calls, identical across three repetitions:

| Operation / input | Maximum | Heap bytes | Allocation calls |
| --- | ---: | ---: | ---: |
| block_profile, constant, 104,857,600 B | 2 | 280 | 1 |
| block_profile, uniform, 104,857,600 B | 2 | 12,336 | 42 |
| block_profile, mixed, 1,048,576 B | 4 | 40,753,592 | 284,351 |
| block_profile, mixed, 64 B | 64 | 2,616 | 298 |
| block_profile_metadata, uniform, 104,857,600 B | usize::MAX | 0 | 0 |
| block_profile_select, uniform, 104,857,600 B | usize::MAX | 0 | 0 |

All maximum-one, empty, zero-limit, metadata and selected-entry cases reported
zero additional heap and zero allocation calls. Higher-order full/prefix
profiles can allocate substantially; bounding the maximum does not bound
observed support independently of input length.
At 100 MiB, profile operations reported RSS 104,064–104,640 KiB.
At 100 MiB, input controls reported RSS 104,256–104,640 KiB.
RSS includes resident input and process overhead. Raw intervals, samples and
readings are retained for future comparisons.
