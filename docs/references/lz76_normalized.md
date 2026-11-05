# LZ76 normalization: references and verification

A. Lempel and J. Ziv (1976), *On the Complexity of Finite Sequences*,
IEEE Transactions on Information Theory 22(1), 75–81,
[original paper](https://doi.org/10.1109/TIT.1976.1055501).
The raw exhaustive production convention remains unchanged; see
[raw-count verification](lz76.md).

E. Estevez-Rams, R. Lora Serrano, B. Aragón Fernández and I. Brito Reyes (2013),
*On the non-randomness of maximum Lempel Ziv complexity sequences of finite size*,
[arXiv:1311.0546](https://arxiv.org/abs/1311.0546), equations (4)–(6) and section IV.
This primary research defines the asymptotic scale using the alphabet logarithm,
and explains why finite scores may exceed one. The library exposes that formula
with a declared alphabet; it does not implement the paper's finite-maximum
normalization. Phrase density c/N is a simple library rescaling by the trivial
bound c<=N, not a separate historically named entropy estimator.

## Independent calculations

`python3 scripts/reference_lz76_normalized.py --check` verifies 7,224 cases.
The Python oracle imports substring-novelty parsing from `reference_lz76.py`;
it never calls the Rust library or scans earlier source positions for the
longest match. It then uses 100-digit Decimal arithmetic and natural logarithms
to rescale, independently of Rust's f64 base-two logs.

Cases include all binary strings through length nine, empty/singleton inputs,
high byte labels, paper examples, full 256-symbol support, UTF-8 bytes, periodic
and long constant strings. Each uses invalid and valid declared alphabets,
including unobserved symbols. Mathematical normalizations use 1e-12 absolute
tolerance; errors and integer summaries are exact. Hand examples establish
ABAB density 0.75 and binary asymptotic 1.5, preventing accidental clamping.

Seeded properties check density bounds, the loose asymptotic log_K(N) bound,
nonincrease with growing K, bijective label invariance, constant-string formulas
and scalar/summary agreement. Tests cover maximum machine-integer alphabets,
validation precedence, retained summaries after input destruction and error traits.
The existing raw parser tests independently enumerate minimum production
histories, including overlapping copies and final reproducible suffixes.

Allocation checks cover empty/valid/invalid calls and 10 MiB constants. Benchmarks
separate scalar calls, summary construction and cached queries; bounded diverse
inputs respect the parser's quadratic worst case. Fuzzing checks independent
novelty parsing and normalizations, supplementing the high-precision cases.

## Review outcome

Review checked denominator conventions, label/cardinality distinctions,
validation before parsing, final-suffix counting, positive-zero conventions,
floating range, summary ownership and allocation promises. It found no new
production defect. Validation corrected a fuzz-harness item-order lint before
the campaign. All 62 standard validation commands passed, including debug and
release tests with default/all features, examples, documentation and seed replay.
See [fuzz evidence](../fuzzing/lz76_normalized.md) and
[performance evidence](../benchmarks/lz76_normalized.md) for their separate limits.
