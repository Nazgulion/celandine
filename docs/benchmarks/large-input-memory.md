# Large-input performance and peak memory

This increment extends the existing entropy byte benchmarks to **100 MiB** and
measures memory independently of latency. It changes development tooling only;
library algorithms and public APIs remain unchanged. Historical baseline CSVs
are preserved. The initial measurement results are recorded below.

## Reproduce the measurements

From the repository root, using Rust 1.90.0 and Python 3.10+:

```sh
python3 scripts/benchmark_large_inputs.py
python3 scripts/measure_memory.py
```

The memory script additionally requires Linux, `lscpu`, `getconf`, and GNU
`/usr/bin/time`. Each script creates a fresh directory under `target/measurements/`
and prints its location. Timing output includes Criterion raw data, logs, a CSV,
compiler/host information, commands, and source hashes. Memory output includes
per-process readings, commands, tool versions, page size, and source hashes.
Keep the result directories when comparing later runs. Run the scripts
sequentially and avoid other project tests or benchmarks during measurement.

For a short tooling check rather than a performance measurement:

```sh
cargo bench --locked --bench peak_memory -- --self-check
python3 scripts/measure_memory.py --smoke --repetitions 1
```

CI compiles all benchmark harnesses and runs the small memory check. It does not
enforce latency or RSS thresholds on shared runners. `peak_memory` is a
development benchmark executable, not a new application interface.

## Latency workload and units

All seven entropy harnesses now include 100 MiB in their normal size loops.
The new script selects only 10 MiB and 100 MiB, giving **84 cases**:

- Shannon, Hartley, collision, min-entropy, Rényi order two, and Tsallis order
  two, each with constant, cycling/uniform, skewed, and mixed inputs at both sizes.
- Rényi and Tsallis at order `1 + 1e-8` on mixed input at both sizes.
- Original Shannon and its explicit bases 2, `E`, and 10, with all four shapes
  at both sizes. The original call provides a control in the same run.

Shapes and seed match the older harnesses: constant zero; bytes `i mod 256`;
one at every hundredth position and zero otherwise; and high-byte xorshift64
output with seed `0x123456789abcdef0` and shifts 13, 7, 17. Mixed bytes are
deterministic workload data, not a probabilistic source assumption.

Inputs are generated outside timing. Every call includes byte counting; inputs,
parameters, and outputs use `black_box`. Reused distributions operate on fixed
256-entry tables, so their existing latency benchmarks remain appropriate and
are not described as scanning 100 MiB. The new run uses 20 Criterion samples,
300 ms warmup, and at least one second of measurement; slow cases extend that
duration to collect enough samples. The reported estimate is the regression
slope when available, otherwise the sample mean, with a 95% confidence interval.
See the [Criterion documentation](https://bheisler.github.io/criterion.rs/book/user_guide/command_line_options.html).

Binary units are explicit: `100 MiB = 104,857,600 B`, exceeding the plan's
decimal 100 MB tier. For input size `N` bytes and measured latency `t` nanoseconds,
throughput is `(N / 2^20) * (10^9 / t)` MiB/s. For example, a 100 MiB scan taking
25 ms has throughput `100 / 0.025 = 4000 MiB/s`. This is input bytes per second,
not a guarantee about physical memory bandwidth. The 10 MiB cases are controls
from the same run; older measurements are not used to claim an optimization.

## What the memory measurements mean

The probe uses a benchmark-local wrapper around Rust's
[`System` global allocator](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html).
It records successful allocation/reallocation calls and the largest sum of live
requested allocation sizes during one operation. For live requested bytes `L`,
allocation of `s` bytes adds `s`; freeing subtracts `s`; a successful reallocation
replaces the old size with the new size. Peak heap is `max(L)` over the measured
interval. All allocations created inside that interval are dropped before it
ends; inputs and prebuilt reuse tables stay alive outside the interval.

The instrument's self-check independently exercises a 64-byte allocation plus
32 zeroed bytes, growth from 64 to 128, shrinkage from 128 to 32, and both frees.
The expected peak is `128 + 32 = 160 B`, with four allocation/reallocation calls
and zero live bytes at the end. A second empty measurement must reset to zero.
Small real n-gram checks confirm construction allocates while extraction and
probability reuse do not.

Requested heap excludes allocator metadata, size-class padding, stack memory,
input storage, and internal transient storage during reallocation. It describes
observed calls in this compiled executable, not every possible compiler's
allocation behavior. The global counter is deliberately single-threaded and
scoped to complete operations; it is not a general-purpose profiler.

Separately, GNU `time -f %M` records maximum process resident set size (RSS) in
KiB on this Linux target; see Linux's
[resource-usage definition](https://man7.org/linux/man-pages/man2/getrusage.2.html).
RSS includes the input, live tables, allocator overhead,
stack, code, and runtime pages. It covers the whole child lifetime, including
input construction and output. Each case runs in a new process three times.
The input is written at 4 KiB intervals and fully read before measuring the
operation, ensuring zero-filled inputs also have resident private pages on the
recorded 4 KiB-page host. Compare RSS alongside an input-only control; subtracting
two process peaks does not give an exact operation allocation size.

For **reuse**, the table is built before heap accounting starts: zero new heap
does not mean the existing table or borrowed input occupies no memory. RSS still
includes both. Probe timing is not reported as library latency because allocator
instrumentation adds overhead. Dedicated stack high-water and cycles/byte remain
unmeasured; neither is inferred from zero heap allocations or process RSS.

## Memory workloads

There are 48 cases, each run three times (**144 fresh-process readings**):

- Input-only, all byte/distribution entropy APIs, and length-four extraction at
  10/100 MiB for all four shapes.
- Length-two count construction, probability construction, and probability reuse
  at 10/100 MiB for constant and cycling inputs (one or 256 distinct blocks).
- Input-only and the same three table operations for mixed length-eight blocks
  at 64 KiB, 1 MiB, and 4 MiB, exposing growth with distinct-block support.

Large count cases deliberately bound support. High-diversity 100 MiB n-gram
tables are not measured or claimed to fit a fixed memory budget. Their borrowed
keys avoid copying block contents but still retain the source input and allocate
tree nodes proportional to the number of distinct blocks. The mixed series
measures this growth at controlled sizes rather than extrapolating it as a bound.

## Environment and results

Run date: 2026-09-20; library parent commit `bf4f8b4`, with measurement source
hashes retained in each archive's manifest. Host: Intel Core i7-13700HX, x86-64
Linux 6.8.0-134-generic; Rust 1.90.0 (`1159e78c4`), LLVM 20.1.8, Criterion 0.7.0,
default Cargo optimized benchmark profile. There were no custom Rust flags,
CPU pinning, or frequency-policy changes (cpu0 reported `powersave`). No other
project tests or benchmarks ran concurrently with either measurement script.

Retained evidence:

- [84 latency estimates and 95% confidence intervals](large-input.csv), with
  [raw Criterion samples, estimates, logs, and manifest](large-input-raw.tar.gz).
- [144 memory readings](peak-memory.csv), with
  [commands, source hashes, and environment manifest](peak-memory-raw.tar.gz).

Archives use paths relative to the run directory; `$REPO` substitutes the
original checkout path in text. For example,
`tar -xOf docs/benchmarks/peak-memory-raw.tar.gz manifest.json` prints its metadata.

100 MiB byte-path latency point estimates, in milliseconds:

| API | Constant | Cycling | Skewed | Mixed |
| --- | ---: | ---: | ---: | ---: |
| Shannon | 157.351 | 20.274 | 154.788 | 25.892 |
| Hartley | 157.303 | 20.218 | 154.836 | 25.408 |
| Rényi, order 2 | 157.664 | 20.428 | 155.004 | 25.550 |
| Collision | 157.083 | 20.708 | 155.102 | 25.808 |
| Min-entropy | 157.536 | 19.279 | 155.745 | 25.222 |
| Tsallis, order 2 | 157.670 | 19.799 | 155.296 | 25.424 |
| Shannon, explicit base 2 | 157.551 | 19.756 | 154.897 | 25.513 |
| Shannon, explicit base E | 157.523 | 19.881 | 155.284 | 25.620 |
| Shannon, explicit base 10 | 158.033 | 19.652 | 155.183 | 25.507 |

Near-one mixed cases took 25.613 ms for Rényi and 25.473 ms for Tsallis. Default
Shannon throughput was 635.5 MiB/s on constant input and 3862.2 MiB/s on mixed
input. Frequent updates to the same histogram counter are a plausible reason
for shape-dependent costs; this run does not isolate that cause. Small timing
differences between APIs are not evidence of speedups.

Selected memory readings (RSS range over three fresh processes):

| Operation/input | Distinct blocks | Peak operation heap (B) | Whole-process RSS (MiB) |
| --- | ---: | ---: | ---: |
| Input-only, mixed 100 MiB | — | 0 | 102.000–102.000 |
| All entropy APIs, mixed 100 MiB | — | 0 | 102.000–102.188 |
| Extract n=4, mixed 100 MiB | — | 0 | 101.812–102.000 |
| Count n=2, constant 100 MiB | 1 | 280 | 101.625–102.000 |
| Count n=2, cycling 100 MiB | 256 | 12,336 | 101.812–102.000 |
| Construct probabilities n=2, cycling 100 MiB | 256 | 12,336 | 101.812–102.000 |
| Reuse probabilities n=2, cycling 100 MiB | 256 | 0 | 101.812–102.000 |
| Count n=8, mixed 64 KiB | 65,529 | 2,560,832 | 4.500–4.688 |
| Count n=8, mixed 1 MiB | 1,048,569 | 40,760,984 | 42.750–42.938 |
| Count n=8, mixed 4 MiB | 4,194,297 | 162,933,912 | 165.562–165.750 |
| Construct probabilities n=8, mixed 4 MiB | 4,194,297 | 162,933,912 | 165.375–165.750 |
| Reuse probabilities n=8, mixed 4 MiB | 4,194,297 | 0 | 165.188–165.562 |

Entropy and extraction made zero heap allocation calls across every measured
shape and size. Table construction had identical requested peaks across the
three repetitions of each workload. The 4 MiB mixed count case used about
155.386 MiB of requested table heap; its 4,194,297 distinct blocks explain why
small input size alone does not imply a small n-gram table. Its reuse operation
allocated nothing new but retained that table and the borrowed input.

The probe's calibration and small real operations also passed Valgrind 3.18.1
Memcheck with zero errors and zero definitely/indirectly/possibly lost bytes
(456 bytes remained reachable in the process runtime). This validates the
instrument's exercised allocation contracts independently of its counters.

Frequency scaling, core scheduling, allocator behavior, cache state, and other
machine activity affect both latency and RSS. Confidence intervals describe the
sampled timings; repeated RSS ranges are observations, not universal bounds.
