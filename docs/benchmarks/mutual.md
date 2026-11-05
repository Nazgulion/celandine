# Mutual information: initial baseline

Measured 2026-11-05 on Linux x86-64, Intel Core i7-13700HX, Rust 1.90.0
(`1159e78c4`, LLVM 20.1.8), Criterion 0.7, optimized bench profile, without
custom RUSTFLAGS, CPU affinity or governor changes.

## Workloads and timing

The [joint-law generator](joint.md) creates two N-byte slices: constant (0,0),
uniform `(i mod 256, floor(i/256) mod 256)`, skewed divisibility indicators for
100/127, or the top two bytes of each xorshift64 state (seed
`0x123456789abcdef0`, shifts 13/7/17). Inputs are generated outside timing.

Forty byte API cases cover every shape at N=16, 64, 256, 1024, 4096, 16384,
65536, 1048576, 10485760 and 104857600. Each includes joint-table construction,
mutual information and table destruction. Four reuse cases prebuild 64 KiB
laws outside timing. Three further reuse cases use symmetric 2x2 cell counts
`[k+1,k-1;k-1,k+1]` for k=64,4096,65536; these exercise small departures from
independence. A diagonal case pairs 64 KiB of cycling bytes with itself, yielding
8 bits. Empty/empty and unequal-length calls complete the 50 latency cases.

Byte cases use flat sampling; other groups use Criterion's default selection.
All request ten samples, 100 ms warm-up and 500 ms measurement, extended for
slow calls. CSV selects fitted slope where available, otherwise mean, with 95%
confidence intervals. `bytes_per_input` is N; throughput counts **both slices**:
`2*N/2^20 * 10^9/t` MiB/s for latency t nanoseconds. Reuse/edge throughput is
blank because they do not scan the input slices. Results describe this host,
not a shared-runner performance gate.

## Memory

The calibrated [probe](large-input-memory.md) runs 153 fresh processes: three
repetitions of input control, byte API and reuse at four shapes and 16 B,
64 KiB, 10 MiB and 100 MiB per input, plus three empty cases. `mutual_input`
keeps both buffers resident; `mutual` includes construction, calculation and
destruction; `mutual_reuse` constructs its law before operation accounting.

CSV `input_bytes` stores N (buffers contain 2N bytes). `ngram_length=1` is an
unused placeholder. Reuse rows report observed pair support; byte API and input
controls report zero in that column. For the byte API this means **support not
reported**, avoiding another tally. Operation heap excludes input buffers and,
for reuse, the prebuilt law. That law still contributes to process RSS.
Input pages are written and both buffers read before the measured operation.

RSS also includes runtime, stack and allocator metadata, and is distinct from
requested operation heap. Two fixed 256-count histograms supply marginals;
stack high-water usage and cycles/byte were not measured. Isolated debug/release
allocation tests separately check empty/errors and law reuse. The historical
joint workload list has a retained regression against accidental changes.

## Reproduce and inspect

```sh
python3 scripts/benchmark_mutual.py
python3 scripts/measure_memory.py --mutual
```

Run sequentially after validation and fuzzing. Parent commit: `6d65658`;
manifest SHA-256 values identify the measured working-tree sources. Archives
normalize checkout paths to `$REPO`.

- [Latency CSV](mutual.csv) and [raw samples](mutual-raw.tar.gz).
- [Memory CSV](mutual-memory.csv) and [raw readings](mutual-memory-raw.tar.gz).

## Results

Byte API latency in microseconds, including joint-table construction and destruction:

| Bytes per input | Constant | Uniform pairs | Skewed | Mixed |
| --- | ---: | ---: | ---: | ---: |
| 16 | 0.052 | 0.523 | 0.306 | 0.730 |
| 65,536 | 119.384 | 4770.090 | 132.212 | 6260.287 |
| 1,048,576 | 1917.669 | 52130.497 | 2101.504 | 71663.697 |
| 104,857,600 | 196058.149 | 5021280.065 | 213761.038 | 6755256.778 |

Reusable laws, nanoseconds per call:

| Shape | Original bytes per input | Latency ns |
| --- | ---: | ---: |
| constant | 65,536 | 1.413 |
| diagonal | 65,536 | 4666.691 |
| mixed | 65,536 | 1498079.975 |
| near_independent | 16,384 | 266.310 |
| near_independent | 256 | 281.143 |
| near_independent | 262,144 | 259.766 |
| skewed | 65,536 | 277.798 |
| uniform | 65,536 | 501504.611 |

Empty/empty and unequal-length calls took 9.064 and 7.659 ns respectively. Tiny timings include harness overhead.

Counting dominates large diverse byte inputs. Reuse avoids that pass, but
still computes marginals and traverses observed cells; small tables are much
cheaper than large supports. Constant pair laws return immediately. Uniform
full-pair laws have exact zero information yet still require traversal to
establish it. Timings do not justify using a statistical independence shortcut
or a cutoff for small information values.

Requested operation heap and allocation calls, identical across three repetitions:

| Operation / shape | Bytes per input | Heap bytes | Allocation calls |
| --- | ---: | ---: | ---: |
| mutual / constant | 104,857,600 | 128 | 1 |
| mutual / uniform | 104,857,600 | 1,508,288 | 10,699 |
| mutual / skewed | 104,857,600 | 128 | 1 |
| mutual / mixed | 104,857,600 | 1,221,280 | 8,768 |
| mutual_reuse / mixed | 104,857,600 | 0 | 0 |

All reusable-law, empty and input-control operations reported zero additional
heap and zero allocation calls. The byte API allocates the sparse joint table.
With 100 MiB per input, metric operations had RSS 206,400–208,512 KiB.
With 100 MiB per input, paired input controls had RSS 206,400–206,784 KiB.
RSS includes both resident buffers, process overhead and any prebuilt joint
law. Raw samples and confidence intervals are retained for future comparisons.
