# Celandine — Sequence Information & Complexity Library
## Detailed Project Plan

**Project name:** `celandine`  
**Primary language:** Rust  
**Project type:** Pure mathematical / algorithmic library  
**Primary object:** Finite discrete sequences  
**Initial optimized representation:** `&[u8]`  
**Long-term representation:** Generic finite sequences over discrete alphabets

---

# 1. Vision

Build a high-performance, mathematically rigorous library for measuring:

- entropy,
- information,
- divergence,
- randomness,
- statistical structure,
- algorithmic / compression-based complexity,
- similarity and distance,

of finite strings and discrete sequences.

The library must be **domain-independent**.

It must not know whether a sequence represents:

- an HTTP request,
- source code,
- DNA,
- an opcode stream,
- a syscall sequence,
- a network packet,
- a log message,
- a text string,
- a protocol payload,
- or an arbitrary symbol sequence.

Those are applications of the library, not part of its mathematical core.

The conceptual object is:

\[
x = (x_1, x_2, \ldots, x_n), \qquad x_i \in \Sigma
\]

where \(\Sigma\) is a finite alphabet.

---

# 2. Core Design Principles

The project should follow these principles from the beginning.

## 2.1 Mathematical correctness first

Every metric must have:

1. a precise mathematical definition,
2. a clearly identified convention,
3. documented edge cases,
4. one or more primary references,
5. independently verifiable test vectors.

Do not implement a formula merely because it appears in a blog post or another library.

---

## 2.2 Exact quantities and estimators must be distinguished

Examples:

| Quantity | Status |
|---|---|
| Empirical Shannon entropy | Exact for the empirical distribution |
| Rényi entropy of an empirical distribution | Exact |
| Jensen-Shannon divergence | Exact given distributions |
| Entropy rate of an unknown source | Estimated |
| Topological entropy from one finite string | Estimated |
| Kolmogorov complexity | Uncomputable in general |
| Compression complexity | Proxy / upper-bound-style approximation |
| NCD | Compressor-dependent practical approximation |

Never expose an estimator as if it were an exact mathematical quantity.

---

## 2.3 No fake Kolmogorov complexity API

Do **not** provide:

```rust
kolmogorov_complexity(x)
```

True Kolmogorov complexity \(K(x)\) is not computable in general.

Instead expose explicitly named approximations such as:

```rust
compression_complexity(x, compressor)
compressed_length(x, compressor)
normalized_compression_distance(x, y, compressor)
```

The documentation should explain their relationship to algorithmic information theory.

---

## 2.4 Finite sequences are primary

Do not architect the project around UTF-8 strings.

The initial fast path should operate on:

```rust
&[u8]
```

because it provides:

- a bounded alphabet,
- predictable memory behavior,
- extremely fast counting,
- natural support for binary data,
- easy integration with strings via `.as_bytes()`.

Later, generic discrete symbols can be supported.

Conceptually:

```rust
fn shannon<T: Eq + Hash>(sequence: &[T]) -> f64
```

The generic API should not compromise the optimized byte path.

---

## 2.5 No application-specific semantics in the core

The core library must not contain:

- HTTP parsing,
- malware detection,
- attack classification,
- anomaly thresholds,
- CVE knowledge,
- machine-learning models,
- cybersecurity-specific rules,
- databases,
- web APIs.

A separate project may use this library for those purposes.

---

## 2.6 Deterministic behavior

Given identical input and configuration, every function should return identical output.

Any stochastic estimator must require an explicit random seed or RNG supplied by the caller.

---

## 2.7 Minimal hidden behavior

Do not silently:

- normalize invalid probability vectors,
- smooth zero probabilities,
- discard NaNs,
- substitute default alphabets,
- change logarithm bases.

If smoothing or normalization is useful, expose it explicitly.

---

# 3. Mathematical Scope

The project should eventually contain the following major families.

```text
Finite Sequence
│
├── Distribution
│   ├── symbol counts
│   ├── probabilities
│   ├── joint distributions
│   ├── conditional distributions
│   └── n-gram/context distributions
│
├── Entropy
│   ├── Shannon
│   ├── Hartley
│   ├── Rényi
│   ├── Collision
│   ├── Min-entropy
│   ├── Tsallis
│   ├── Joint entropy
│   ├── Conditional entropy
│   ├── Block entropy
│   └── Entropy-rate estimators
│
├── Information
│   ├── Self-information / surprisal
│   ├── Mutual information
│   ├── Conditional mutual information
│   ├── Pointwise mutual information
│   └── Total correlation
│
├── Divergence / Distance
│   ├── KL divergence
│   ├── Jensen-Shannon divergence
│   ├── Jensen-Shannon distance
│   ├── Hellinger distance
│   ├── Total variation
│   └── Bhattacharyya measures
│
├── Sequence Complexity
│   ├── LZ76
│   ├── normalized LZ76
│   ├── subword complexity
│   ├── linguistic complexity
│   ├── excess entropy
│   └── topological-entropy estimators
│
├── Compression / Algorithmic Proxies
│   ├── compressed length
│   ├── compression ratio
│   ├── compression complexity
│   ├── compression distance
│   └── normalized compression distance
│
├── Statistical Complexity
│   ├── approximate entropy
│   ├── sample entropy
│   ├── permutation entropy
│   └── explicitly named complexity definitions
│
└── Transformations
    ├── n-grams
    ├── blocks
    ├── sliding windows
    └── context extraction
```

---


# 4. Crate / Library Architecture

The project should **not** create one separate library for every mathematical family or metric.

The default rule is:

> **Split by dependency boundary or runtime environment, not by mathematical concept.**

That means mathematical areas such as Shannon entropy, Rényi entropy, Tsallis entropy, Jensen-Shannon divergence, LZ76 complexity, and subword complexity should live as modules inside the same core library.

Examples:

```text
Shannon vs Rényi vs Tsallis
→ separate modules/files inside the same crate

Entropy vs information vs divergence vs complexity
→ separate modules inside the same crate

Core mathematics vs compression engines
→ potentially separate crates

Rust library vs CLI
→ separate crates

Rust library vs Python bindings
→ separate crates/packages

Rust library vs WASM bindings
→ separate crates/packages
```

## 4.1 Initial architecture: one crate

The project should begin as a single Rust crate:

```text
celandine/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── distribution/
│   ├── entropy/
│   ├── information/
│   ├── divergence/
│   ├── complexity/
│   ├── sequence/
│   ├── transforms/
│   └── estimators/
├── docs/
├── benches/
└── tests/
```

The public API should look conceptually like:

```rust
use celandine::entropy::shannon;
use celandine::entropy::renyi;
use celandine::entropy::tsallis;

use celandine::divergence::jensen_shannon_divergence;

use celandine::complexity::lz76_complexity;
```

A normal user should install one dependency:

```toml
celandine = "0.1"
```

They should **not** need separate dependencies such as:

```text
shannon-rs
renyi-rs
tsallis-rs
lz76-rs
jensen-shannon-rs
```

That would create unnecessary versioning, API inconsistency, dependency management, and maintenance overhead.

## 4.2 Internal modularity

Although the project should initially be one crate, each algorithm should remain internally isolated.

Example:

```text
src/
├── entropy/
│   ├── mod.rs
│   ├── shannon.rs
│   ├── renyi.rs
│   ├── tsallis.rs
│   ├── hartley.rs
│   └── min_entropy.rs
│
├── information/
│   ├── mod.rs
│   ├── surprisal.rs
│   ├── mutual_information.rs
│   └── conditional_mutual_information.rs
│
├── divergence/
│   ├── mod.rs
│   ├── kl.rs
│   ├── jensen_shannon.rs
│   ├── hellinger.rs
│   └── total_variation.rs
│
└── complexity/
    ├── mod.rs
    ├── lz76.rs
    ├── subword.rs
    └── linguistic.rs
```

This gives the project:

- one coherent external API,
- independent algorithm implementations,
- easier testing,
- clear mathematical ownership,
- room for each algorithm to evolve without fragmenting the package ecosystem.

## 4.3 When to split into multiple crates

Only split when a component introduces a meaningful dependency, build, or runtime boundary.

A likely future workspace:

```text
celandine/
├── Cargo.toml
├── crates/
│   ├── celandine/
│   ├── celandine-core/
│   ├── celandine-compression/
│   ├── celandine-cli/
│   ├── celandine-python/
│   └── celandine-wasm/
├── docs/
├── benches/
└── examples/
```

### `celandine-core`

Contains pure mathematical algorithms with minimal dependencies:

```text
distribution
entropy
information
divergence
sequence complexity
transformations
estimators
```

Design goals:

- no networking,
- no CLI framework,
- no Python runtime,
- no compression libraries,
- no application-specific dependencies,
- ideally very small dependency graph.

### `celandine-compression`

Contains compressor-dependent functionality:

```text
DEFLATE
zstd
LZ4
Brotli

compression ratio
compression complexity
NCD
```

Reason for separation:

A user who only needs:

```rust
shannon(data)
```

should not have to compile several compression libraries.

Compression support may alternatively begin as feature flags before becoming a separate crate.

### `celandine-cli`

Provides:

```bash
celandine shannon file.bin
celandine renyi --alpha 2 file.bin
celandine lz76 file.bin
celandine ncd a.bin b.bin
```

This crate depends on the mathematical library and contains command-line concerns only.

### `celandine-python`

Python bindings, probably through PyO3 / maturin.

Example:

```python
import celandine

celandine.shannon(data)
celandine.renyi(data, alpha=2.0)
celandine.lz76(data)
```

Python-specific dependencies must not leak into the Rust core.

### `celandine-wasm`

Optional browser / WebAssembly bindings.

WASM-specific compatibility requirements should remain isolated from the core library.

### `celandine`

If the project becomes a workspace, this can become the facade crate.

It re-exports the normal public API so most Rust users still write:

```toml
celandine = "1.0"
```

rather than manually selecting internal crates.

Conceptually:

```rust
pub use celandine_core::entropy;
pub use celandine_core::information;
pub use celandine_core::divergence;
pub use celandine_core::complexity;

#[cfg(feature = "compression")]
pub use celandine_compression::*;
```

## 4.4 What should remain modules, not crates

The following should normally remain modules inside `celandine-core`:

```text
Shannon entropy
Hartley entropy
Rényi entropy
Tsallis entropy
collision entropy
min-entropy

joint entropy
conditional entropy
mutual information
surprisal

KL divergence
Jensen-Shannon divergence
Hellinger distance
total variation

block entropy
entropy-rate estimators

LZ76
subword complexity
linguistic complexity
excess entropy
```

The fact that these belong to different mathematical families does **not** justify separate packages.

## 4.5 Architectural principle

The architectural hierarchy should be:

```text
Repository
    ↓
Public library identity: celandine
    ↓
Core mathematical crate
    ↓
Mathematical modules
    ↓
Individual algorithm implementations
```

Optional ecosystems branch outward:

```text
                   celandine
                       │
         ┌─────────────┼─────────────┐
         │             │             │
       core      compression      bindings
         │             │          /      \
         │             │       Python    WASM
         │
   mathematical
      modules
```

The user-facing identity remains **one project and one library**.

## 4.6 Decision rule for future splits

Before creating a new crate, answer all of these questions:

1. Does this component introduce substantial external dependencies?
2. Does it target a different runtime or language ecosystem?
3. Does it have a different release or compatibility lifecycle?
4. Can core users reasonably want to avoid compiling it?
5. Would splitting it reduce coupling without making the API harder to use?

If most answers are **no**, keep it as a module.

---


# 5. Proposed Repository Structure

```text
celandine/
├── Cargo.toml
├── README.md
├── LICENSE
├── CHANGELOG.md
├── CONTRIBUTING.md
│
├── crates/
│   ├── celandine-core/
│   ├── celandine-compression/
│   ├── celandine-cli/
│   ├── celandine-python/
│   └── celandine-wasm/
│
├── docs/
│   ├── mathematical-conventions.md
│   ├── numerical-behavior.md
│   ├── references.md
│   │
│   ├── entropy/
│   ├── information/
│   ├── divergence/
│   ├── complexity/
│   ├── compression/
│   └── estimators/
│
├── benches/
├── examples/
├── test-data/
└── papers/
```

Initially, avoid a multi-crate workspace if it creates unnecessary complexity.

A sensible first repository may simply be:

```text
src/
├── distribution/
├── entropy/
├── information/
├── divergence/
├── complexity/
├── transforms/
└── lib.rs
```

Split optional compression backends and bindings later.

---

# 6. Phase 0 — Foundations

This phase should exist before implementing large numbers of metrics.

## 5.1 Symbol counting

Implement high-performance frequency counting.

Required operations:

```text
frequency table
unique-symbol count
alphabet cardinality
empirical probability distribution
```

For bytes, use a fixed-size table when appropriate:

```text
[usize; 256]
```

The byte path should avoid hashing entirely.

---

## 5.2 Probability distribution abstraction

Create a validated probability distribution abstraction.

Requirements:

- probabilities must be finite,
- probabilities must be non-negative,
- sum must be approximately one,
- tolerance must be explicit,
- empty distributions must have defined behavior.

Possible conceptual API:

```rust
Distribution::from_counts(...)
Distribution::from_probabilities(...)
```

Useful methods:

```text
len
probability
entropy-ready iterator
support size
normalize explicitly
```

---

## 5.3 Joint distributions

Needed for:

- joint entropy,
- conditional entropy,
- mutual information,
- conditional mutual information.

Support paired observations:

\[
(x_i, y_i)
\]

and later tuples.

---

## 5.4 N-grams

For sequence:

```text
ABCD
```

2-grams:

```text
AB
BC
CD
```

3-grams:

```text
ABC
BCD
```

Required primitives:

```rust
ngrams(x, n)
ngram_counts(x, n)
ngram_probabilities(x, n)
```

Special cases must be defined:

- `n = 0`,
- `n = 1`,
- `n > len(sequence)`,
- empty input.

---

## 5.5 Context counts

Needed for Markov-style conditional entropy.

For order \(k\), count:

\[
(x_{i-k}, \ldots, x_{i-1}) \to x_i
\]

This machinery should be generic enough to support multiple estimators later.

---

# 7. Phase 1 — Core Entropy

Target release: **v0.1**

Goal:

> Deliver an exceptionally correct and well-tested entropy core.

---

## 6.1 Shannon entropy

Definition:

\[
H(X)=-\sum_x p(x)\log_2 p(x)
\]

Required API concepts:

```rust
shannon(sequence)
shannon_distribution(distribution)
shannon_with_base(sequence, base)
```

Potential normalized form:

\[
H_N(X)=\frac{H(X)}{\log_2 |\Sigma|}
\]

But normalization must document what \(\Sigma\) means:

- observed support,
- theoretical alphabet,
- user-supplied alphabet.

Do not silently assume one interpretation.

### Required properties

\[
H(X)\ge0
\]

For a constant sequence:

\[
H(X)=0
\]

For a uniform empirical distribution with \(k\) observed symbols:

\[
H(X)=\log_2 k
\]

---

## 6.2 Hartley entropy

\[
H_0(X)=\log_2 |\operatorname{supp}(X)|
\]

Expose explicitly:

```rust
hartley_entropy(sequence)
```

Document its relationship to Rényi entropy of order 0.

---

## 6.3 Rényi entropy

\[
H_\alpha(X)=
\frac{1}{1-\alpha}
\log_2\left(
\sum_i p_i^\alpha
\right)
\]

Required support:

- finite positive \(\alpha\),
- \(\alpha = 0\),
- \(\alpha \to 1\),
- \(\alpha = 2\),
- \(\alpha \to \infty\).

Convenience functions may include:

```rust
renyi_entropy(sequence, alpha)
collision_entropy(sequence)
min_entropy(sequence)
hartley_entropy(sequence)
```

### Important property

For:

\[
\alpha_1 < \alpha_2
\]

Rényi entropy should satisfy:

\[
H_{\alpha_1}(X)\ge H_{\alpha_2}(X)
\]

for valid distributions.

### Numerical requirement

Near:

\[
\alpha=1
\]

avoid catastrophic numerical behavior.

Use a defined limit or stable branch.

---

## 6.4 Collision entropy

Rényi order 2:

\[
H_2(X)=
-\log_2
\sum_i p_i^2
\]

Provide a dedicated function because it has practical significance.

---

## 6.5 Min-entropy

\[
H_\infty(X)=
-\log_2\max_i p_i
\]

This should be separately exposed even though it is a Rényi limit.

---

## 6.6 Tsallis entropy

\[
S_q(X)=
\frac{1-\sum_i p_i^q}{q-1}
\]

Required:

```rust
tsallis_entropy(sequence, q)
```

Near:

\[
q=1
\]

the implementation should converge to Shannon entropy under the documented logarithm convention.

Document carefully that different communities sometimes use different constants / bases.

---

# 8. Phase 2 — Information Theory

Target release: **v0.2**

---

## 7.1 Joint entropy

\[
H(X,Y)
\]

Required:

```rust
joint_entropy(x, y)
```

Input sequences must have compatible lengths unless the API explicitly supports another mode.

---

## 7.2 Conditional entropy

\[
H(X|Y)=H(X,Y)-H(Y)
\]

Required forms:

```rust
conditional_entropy(x, y)
conditional_entropy_order(sequence, order)
```

These are different concepts and must not be conflated.

---

## 7.3 Self-information / surprisal

\[
I(x)=-\log_2 P(x)
\]

Provide a low-level probability API:

```rust
self_information(probability)
```

and distribution-aware helper:

```rust
surprisal(symbol, distribution)
```

Potential aggregate sequence quantities:

```text
mean surprisal
total surprisal
maximum surprisal
```

These should be clearly defined rather than hidden inside one ambiguous function.

---

## 7.4 Mutual information

\[
I(X;Y)=
\sum_{x,y}
p(x,y)
\log_2
\frac{p(x,y)}
{p(x)p(y)}
\]

Required:

```rust
mutual_information(x, y)
```

Property:

\[
I(X;Y)\ge0
\]

and:

\[
I(X;X)=H(X)
\]

under the usual empirical construction.

---

## 7.5 Conditional mutual information

\[
I(X;Y|Z)
\]

Target API:

```rust
conditional_mutual_information(x, y, z)
```

May be postponed until the joint-distribution abstraction is mature.

---

## 7.6 Pointwise mutual information

\[
PMI(x,y)=
\log_2
\frac{P(x,y)}
{P(x)P(y)}
\]

Required:

```rust
pointwise_mutual_information(...)
```

Define behavior when probabilities are zero.

---

## 7.7 Total correlation

For multiple variables:

\[
TC(X_1,\ldots,X_n)
=
\sum_i H(X_i)
-
H(X_1,\ldots,X_n)
\]

This is lower priority but belongs conceptually in the information module.

---

# 9. Phase 3 — Divergence and Statistical Distance

Target release: **v0.2 or v0.3**

---

## 8.1 KL divergence

\[
D_{KL}(P||Q)=
\sum_i P(i)\log_2\frac{P(i)}{Q(i)}
\]

Required API:

```rust
kl_divergence(p, q)
```

Critical behavior:

If:

\[
P(i)>0
\]

and:

\[
Q(i)=0
\]

then:

\[
D_{KL}(P||Q)=\infty
\]

Do not silently smooth.

Optional explicit helper:

```rust
kl_divergence_smoothed(p, q, epsilon)
```

---

## 8.2 Jensen-Shannon divergence

\[
M=\frac{1}{2}(P+Q)
\]

\[
JSD(P,Q)=
\frac12D_{KL}(P||M)
+
\frac12D_{KL}(Q||M)
\]

Required:

```rust
jensen_shannon_divergence(p, q)
```

Properties:

\[
JSD(P,Q)=JSD(Q,P)
\]

\[
JSD(P,P)=0
\]

With base-2 logarithms and equal weighting:

\[
0\le JSD(P,Q)\le1
\]

---

## 8.3 Jensen-Shannon distance

\[
d_{JS}(P,Q)=\sqrt{JSD(P,Q)}
\]

Expose separately:

```rust
jensen_shannon_distance(p, q)
```

Do not use the terms "distance" and "divergence" interchangeably.

---

## 8.4 Hellinger distance

\[
H(P,Q)=
\frac{1}{\sqrt2}
\sqrt{
\sum_i
(\sqrt{p_i}-\sqrt{q_i})^2
}
\]

Required:

```rust
hellinger_distance(p, q)
```

---

## 8.5 Total variation distance

\[
TV(P,Q)=
\frac12
\sum_i |p_i-q_i|
\]

Required:

```rust
total_variation_distance(p, q)
```

Range:

\[
0\le TV\le1
\]

---

## 8.6 Bhattacharyya coefficient and distance

\[
BC(P,Q)=
\sum_i\sqrt{p_iq_i}
\]

Potential functions:

```rust
bhattacharyya_coefficient(p, q)
bhattacharyya_distance(p, q)
```

The precise distance definition must be documented.

---

# 10. Phase 4 — Sequential Information

Target release: **v0.3**

This phase moves beyond symbol-frequency histograms and begins modeling sequence structure.

---

## 9.1 Block entropy

For blocks of length \(n\):

\[
H_n =
H(X_1,\ldots,X_n)
\]

Empirically:

1. extract length-\(n\) blocks,
2. estimate block probabilities,
3. compute Shannon entropy.

Required:

```rust
block_entropy(sequence, n)
```

---

## 9.2 Conditional entropy by order

Estimate:

\[
H(X_t \mid X_{t-1})
\]

\[
H(X_t \mid X_{t-2},X_{t-1})
\]

and higher orders.

Required:

```rust
conditional_entropy_order(sequence, order)
```

Document:

- estimator,
- treatment of initial symbols,
- insufficient sample behavior,
- bias for large orders.

---

## 9.3 Entropy rate

For a stationary process:

\[
h(X)=
\lim_{n\to\infty}
\frac{1}{n}
H(X_1,\ldots,X_n)
\]

A finite sequence cannot provide the exact entropy rate of its unknown generating process.

Therefore expose estimator-specific names such as:

```rust
entropy_rate_block_estimate(...)
entropy_rate_markov_estimate(...)
entropy_rate_lz_estimate(...)
```

Do not expose a single function that implies exactness.

---

## 9.4 Entropy profile

Return:

\[
H_1,H_2,\ldots,H_k
\]

for block orders.

Potential API:

```rust
block_entropy_profile(sequence, max_order)
```

This can reveal how rapidly additional context stops adding information.

---

## 9.5 Excess entropy

Conceptually:

\[
E=
\sum_{n=1}^{\infty}
(H_n-hn)
\]

For finite data, expose only an explicitly named estimator:

```rust
excess_entropy_estimate(sequence, max_order)
```

This may become a particularly interesting measure of stored / predictive structure.

---

# 11. Phase 5 — Lempel-Ziv and Sequence Complexity

Target release: **v0.4**

---

## 10.1 LZ76 complexity

Implement the specific 1976 Lempel-Ziv sequence-complexity definition.

Required:

```rust
lz76_complexity(sequence)
```

Do not initially call it simply:

```rust
lz_complexity()
```

because LZ76, LZ77, LZ78 and related constructions are distinct.

---

## 10.2 Normalized LZ76 complexity

Multiple normalizations appear in literature.

Before implementation:

1. identify commonly used normalizations,
2. choose explicit names,
3. document asymptotic assumptions,
4. test behavior on finite short strings.

Possible API:

```rust
lz76_complexity_normalized(sequence, normalization)
```

or separate functions.

Avoid one undocumented normalization.

---

## 10.3 Subword complexity

For finite string \(x\):

\[
p_x(n)=
\text{number of distinct substrings of length }n
\]

Required:

```rust
subword_complexity(sequence, n)
subword_complexity_profile(sequence)
```

This provides direct structural information without relying on probabilities.

---

## 10.4 Linguistic complexity

Conceptually compare observed distinct substrings with the maximum possible number.

Required research before implementation:

- exact normalization,
- finite-length denominator,
- alphabet assumptions,
- literature variants.

Expose only after choosing a precise definition.

---

## 10.5 Topological-entropy estimate

For symbolic systems:

\[
h_{\text{top}}
=
\lim_{n\rightarrow\infty}
\frac{1}{n}\log p(n)
\]

A finite string only permits an approximation.

Potential API:

```rust
topological_entropy_estimate(sequence, range)
```

This should be a later research-oriented feature, not part of the first stable core.

---

# 12. Phase 6 — Compression and Algorithmic Complexity Proxies

Target release: **v0.5**

This should live behind optional features or a separate crate so the mathematical core stays lightweight.

---

## 11.1 Compressor abstraction

Define conceptually:

```rust
trait Compressor {
    fn compressed_len(&self, input: &[u8]) -> usize;
}
```

Potential optional backends:

- DEFLATE,
- zstd,
- LZ4,
- Brotli.

The library must document that real compressors include:

- framing overhead,
- headers,
- dictionaries,
- initialization effects,
- finite-window effects.

This matters especially for short strings.

---

## 11.2 Compressed length

```rust
compressed_length(sequence, compressor)
```

Return raw compressed size with well-defined units.

---

## 11.3 Compression ratio

Possible definition:

\[
R(x)=\frac{C(x)}{|x|}
\]

Required:

```rust
compression_ratio(sequence, compressor)
```

Define behavior for empty input.

---

## 11.4 Compression complexity

Use an explicitly proxy-oriented name.

Examples:

```rust
compression_complexity(sequence, compressor)
normalized_compression_complexity(sequence, compressor)
```

Documentation must state:

> This is not Kolmogorov complexity.

---

## 11.5 Compressor overhead calibration

For short strings, a 10–30 byte compressor overhead can dominate results.

Research possible modes:

```text
raw stream
framed stream
pretrained dictionary
shared dictionary
fixed dictionary
batched reference context
```

Do not silently subtract estimated overhead unless the method is mathematically documented.

---

# 13. Phase 7 — Compression-Based Distance

Target release: **v0.5**

---

## 12.1 Normalized Compression Distance

\[
NCD(x,y)=
\frac{
C(xy)-\min(C(x),C(y))
}{
\max(C(x),C(y))
}
\]

Required:

```rust
normalized_compression_distance(x, y, compressor)
```

---

## 12.2 Concatenation asymmetry

Real compressors may produce:

\[
C(xy)\ne C(yx)
\]

Expose or investigate symmetric variants.

Possible explicit function:

```rust
normalized_compression_distance_symmetric(...)
```

with documented construction.

---

## 12.3 Raw components

For research, allow inspection of:

```text
C(x)
C(y)
C(xy)
C(yx)
```

This is useful for diagnosing compressor behavior.

---

# 14. Phase 8 — Statistical Complexity Measures

Target release: **v0.6+**

Only add measures after their mathematical conventions are fully understood.

---

## 13.1 Approximate entropy

Potential:

```rust
approximate_entropy(sequence, m, r)
```

Originally formulated for time-series-style data.

Need to decide whether:

- it belongs in core,
- it belongs in an optional numeric-sequence module,
- symbolic-string adaptation is mathematically justified.

---

## 13.2 Sample entropy

Potential:

```rust
sample_entropy(sequence, m, r)
```

Same scope decision as approximate entropy.

---

## 13.3 Permutation entropy

Primarily intended for ordered numeric observations.

Likely place:

```text
celandine-numeric
```

rather than the initial discrete-symbol core.

---

## 13.4 Statistical complexity families

There is no single canonical "statistical complexity."

Any implementation must use explicit names, for example:

```text
López-Ruiz–Mancini–Calbet complexity
Jensen-Shannon statistical complexity
```

Never expose an ambiguous:

```rust
statistical_complexity()
```

---

# 15. Transformations Module

Transformations should be reusable across metrics.

---

## 14.1 N-grams

```rust
ngrams(sequence, n)
```

Metrics can operate over n-gram symbols.

This enables:

\[
H_1,H_2,H_3,\ldots
\]

and generalized entropy spectra over block representations.

---

## 14.2 Fixed blocks

Support non-overlapping blocks separately from overlapping n-grams.

These are mathematically different transformations.

---

## 14.3 Sliding windows

Generic abstraction:

```rust
windows(sequence, size, step)
```

Then allow:

```text
windowed Shannon entropy
windowed Rényi entropy
windowed LZ complexity
windowed surprisal
```

Prefer composition over implementing dozens of unrelated window functions.

---

## 14.4 Context transformation

Expose sequence contexts for order-\(k\) analysis.

This should power:

- conditional entropy,
- Markov estimators,
- context distributions.

---

# 16. High-Level Profiles

After individual metrics are stable, offer convenience profiles.

Example conceptual result:

```json
{
  "length": 128,
  "alphabet_size": 37,
  "entropy": {
    "shannon": 4.81,
    "hartley": 5.21,
    "renyi_0_5": 5.02,
    "renyi_2": 4.37,
    "collision": 4.37,
    "min": 3.71
  },
  "sequence": {
    "conditional_order_1": 3.24,
    "conditional_order_2": 2.79,
    "lz76": 42,
    "lz76_normalized": 0.67
  }
}
```

A high-level profile must never become the only API.

Every metric must remain independently callable.

---

# 17. Public API Philosophy

Provide three conceptual levels.

---

## 16.1 Level 1 — Simple

```rust
shannon(b"ABAB")
renyi(b"ABAB", 2.0)
min_entropy(b"ABAB")
```

Optimized for convenience.

---

## 16.2 Level 2 — Reusable distributions

When a caller wants many entropy measures, do not repeatedly count symbols.

Conceptually:

```rust
let d = Distribution::from_bytes(data);

d.shannon()
d.renyi(2.0)
d.min_entropy()
```

This should be efficient and explicit.

---

## 16.3 Level 3 — Research / analysis

Potential:

```rust
let analysis = SequenceAnalysis::new(data)
    .max_block_order(5)
    .max_context_order(3);
```

Then compute profiles without repeated work.

This layer should come only after the low-level APIs are stable.

---

# 18. Edge Cases

Create one project-wide document defining edge-case behavior.

Every metric must explicitly address:

- empty input,
- single-symbol input,
- single-element alphabet,
- invalid probability vectors,
- NaN parameters,
- infinite parameters,
- zero probabilities,
- extremely large sequences,
- very high n-gram order,
- contexts larger than the sequence,
- overflow,
- floating-point precision.

Examples:

### Shannon entropy of empty sequence

Choose one explicit behavior:

- error,
- `None`,
- `NaN`,
- defined as zero.

Do not let different modules behave inconsistently.

### Constant sequence

For:

```text
AAAAAA
```

Shannon:

\[
H=0
\]

Min entropy:

\[
H_\infty=0
\]

These should be exact up to floating representation.

---

# 19. Testing Strategy

Testing is a core feature of this project.

---

## 18.1 Hand-verifiable test vectors

Include:

```text
""
"A"
"AAAA"
"AB"
"ABAB"
"ABCD"
"ABCABCABC"
"0123456789"
```

For:

```text
ABCD
```

each symbol has:

\[
p_i=\frac14
\]

therefore:

\[
H=2
\]

This should be an obvious canonical test.

---

## 18.2 Mathematical property tests

Use property-based testing wherever possible.

### Shannon

\[
H(X)\ge0
\]

\[
H(\text{constant})=0
\]

\[
H(X)\le\log_2 |\operatorname{supp}(X)|
\]

### Rényi

For:

\[
\alpha_1<\alpha_2
\]

verify approximately:

\[
H_{\alpha_1}\ge H_{\alpha_2}
\]

Verify:

\[
\lim_{\alpha\to1}H_\alpha=H
\]

### Jensen-Shannon

\[
JSD(P,Q)=JSD(Q,P)
\]

\[
JSD(P,P)=0
\]

### Total variation

\[
0\le TV(P,Q)\le1
\]

### Mutual information

\[
I(X;Y)\ge0
\]

within numerical tolerance.

---

## 18.3 Cross-library verification

Where possible, compare results against established implementations in:

- SciPy,
- NumPy-based reference calculations,
- R packages,
- published examples,
- textbook exercises.

External libraries should be test references, not definitions of truth.

---

## 18.4 Symbolic / exact verification

For tiny rational distributions, generate expected results from exact rational arithmetic where possible before converting to floating point.

This helps expose numerical mistakes.

---

## 18.5 Fuzzing

Fuzz:

- arbitrary byte arrays,
- invalid distribution inputs,
- huge alpha/q values,
- malformed parameters,
- very large n-gram orders.

The library must never panic on ordinary invalid user input unless the API contract explicitly specifies a panic.

---

# 20. Numerical Analysis

Floating-point correctness deserves a dedicated section.

Topics to investigate:

- `f64` as default,
- stable log calculations,
- behavior near Rényi \(\alpha=1\),
- behavior near Tsallis \(q=1\),
- probability normalization tolerance,
- summation error,
- Kahan summation where useful,
- very small probabilities,
- infinite KL divergence.

Avoid premature `f32` support.

---

# 21. Performance Strategy

The library should aim for both mathematical quality and systems-level performance.

---

## 20.1 Benchmark input sizes

At minimum:

```text
16 B
64 B
256 B
1 KB
4 KB
16 KB
64 KB
1 MB
10 MB
100 MB
```

Small strings matter as much as huge data because many real sequence-analysis applications work on short payloads.

---

## 20.2 Benchmark metrics

Measure:

```text
ns/call
MB/s
allocations
peak memory
cycles/byte where useful
```

---

## 20.3 Byte-specialized algorithms

For byte entropy:

```text
[usize; 256]
```

should often outperform hash maps dramatically.

Use:

```text
generic implementation
+
specialized byte path
```

where beneficial.

---

## 20.4 Allocation discipline

Simple metrics such as byte Shannon entropy should ideally require:

- no heap allocations,
- one pass through input,
- one small fixed frequency table.

---

## 20.5 Parallelism

Do not start with parallelism.

Only introduce it after profiling shows that:

- inputs are sufficiently large,
- merging partial statistics is mathematically valid,
- overhead is worthwhile.

---

## 20.6 SIMD

Treat SIMD as an optimization phase, not an architectural requirement.

Potential candidates:

- byte histogram computation,
- block comparisons,
- numeric sequence distances.

Correct scalar implementations come first.

---

# 22. Streaming APIs

Some metrics can naturally support streaming.

Potential conceptual interface:

```rust
let mut estimator = ShannonAccumulator::new();

estimator.update(chunk1);
estimator.update(chunk2);

let result = estimator.finalize();
```

Suitable candidates:

- symbol counts,
- Shannon entropy from accumulated counts,
- some distribution metrics,
- potentially fixed n-gram statistics with retained boundary state.

Not naturally streamable without additional state:

- arbitrary NCD,
- some LZ measures,
- arbitrary sliding-window metrics.

Do not force streaming onto algorithms that do not naturally support it.

---

# 23. Generic Symbol Support

Do not implement generic symbols before the byte API is mature.

Long-term sequence types may include:

```text
u8
char
u16
u32
enum tokens
integer event IDs
custom Hash + Eq symbols
```

Design goal:

```rust
shannon(&[u32])
```

should eventually be possible.

Performance goal:

```rust
shannon(&[u8])
```

should remain extremely fast.

---

# 24. Documentation Standard

Every metric should use the same structure.

## Template

### Name

Canonical metric name.

### Definition

Plain-language description.

### Formula

Mathematical notation.

### Interpretation

What high and low values mean.

### Domain

Required input type / assumptions.

### Range

Known bounds.

### Exact or estimated

Explicit classification.

### Parameters

Meaning and constraints.

### Edge cases

Empty sequence, zero probabilities, etc.

### Computational complexity

Time and memory.

### Numerical notes

Any stability issues.

### Example

Small manually verifiable example.

### References

Original paper and recommended textbook.

### Related measures

Links to closely related functions.

---

# 25. References and Literature

Maintain a formal bibliography.

Core books:

1. **Thomas M. Cover, Joy A. Thomas — _Elements of Information Theory_**
2. **David J. C. MacKay — _Information Theory, Inference, and Learning Algorithms_**
3. **Ming Li, Paul Vitányi — _An Introduction to Kolmogorov Complexity and Its Applications_**
4. **János Aczél, Zoltán Daróczy — _On Measures of Information and Their Characterizations_**
5. **Khalid Sayood — _Introduction to Data Compression_**
6. **Peter D. Grünwald — _The Minimum Description Length Principle_**
7. **Robert M. Gray — _Entropy and Information Theory_**
8. **Douglas Lind, Brian Marcus — _An Introduction to Symbolic Dynamics and Coding_**
9. **Tom Leinster — _Entropy and Diversity: The Axiomatic Approach_**
10. A rigorous probability reference.

Important supplementary literature:

- Shannon's original information theory papers,
- Rényi's original entropy papers,
- Kolmogorov's algorithmic-information papers,
- Lempel and Ziv's original sequence-complexity papers,
- Cilibrasi and Vitányi on normalized compression distance,
- Csiszár and Shields on information theory and statistics,
- Tsallis's original generalized-entropy work.

---

# 26. Per-Metric Reference Files

Create:

```text
docs/references/
├── shannon.md
├── renyi.md
├── tsallis.md
├── min_entropy.md
├── mutual_information.md
├── kl.md
├── jensen_shannon.md
├── entropy_rate.md
├── lz76.md
├── kolmogorov.md
├── mdl.md
└── ncd.md
```

Each should contain:

```text
original definition
modern reference
formula
implementation convention
known variants
numerical issues
test sources
open questions
```

This makes the repository a knowledge base as well as software.

---

# 27. Metric Metadata

Consider exposing metadata programmatically later.

Conceptually:

```rust
MetricInfo {
    name,
    family,
    exact_or_estimator,
    output_range,
    reference,
    time_complexity,
    space_complexity,
}
```

Potential uses:

- CLI help,
- generated documentation,
- research tooling,
- automated benchmarking,
- discovery by other libraries.

Do not build this before the metrics themselves are stable.

---

# 28. CLI Plan

The CLI should be a separate layer.

Working command:

```text
celandine
```

Examples:

```bash
celandine shannon file.bin
celandine renyi --alpha 2 file.bin
celandine lz76 file.bin
celandine ncd file-a file-b
celandine profile file.bin
```

Raw string mode:

```bash
celandine shannon --text "ABABAB"
```

JSON:

```bash
celandine profile file.bin --json
```

The CLI should be:

- deterministic,
- scriptable,
- quiet by default,
- usable through stdin/stdout,
- suitable for shell pipelines and agents.

---

# 29. Language Bindings

Only after the Rust API is stable.

## Python

High priority for research.

Potential:

```python
import celandine

celandine.shannon(data)
celandine.renyi(data, alpha=2.0)
celandine.lz76(data)
celandine.ncd(a, b)
```

## C ABI

Useful for:

- systems integration,
- nginx modules,
- C/C++ projects,
- embedded systems.

## WASM

Potentially useful for:

- browser-based analysis,
- demonstrations,
- educational tools.

Bindings must expose the same mathematical conventions as Rust.

---

# 30. Release Roadmap

---

## v0.1 — Entropy Foundation

### Required

- byte frequency tables,
- empirical distributions,
- n-gram primitive,
- Shannon entropy,
- Hartley entropy,
- Rényi entropy,
- collision entropy,
- min-entropy,
- Tsallis entropy,
- documentation,
- property tests,
- benchmark suite.

### Definition of done

All functions have:

- formula,
- reference,
- edge-case definition,
- unit tests,
- property tests,
- benchmark.

---

## v0.2 — Core Information Theory

Add:

- joint entropy,
- conditional entropy,
- self-information,
- surprisal,
- mutual information,
- pointwise mutual information,
- KL divergence,
- Jensen-Shannon divergence,
- Jensen-Shannon distance,
- Hellinger distance,
- total variation.

---

## v0.3 — Sequence Information

Add:

- block entropy,
- conditional entropy by sequence order,
- entropy profiles,
- entropy-rate estimators,
- excess-entropy estimator,
- context statistics.

This release should include extensive estimator documentation.

---

## v0.4 — Sequence Complexity

Add:

- LZ76 complexity,
- normalized LZ76 variants,
- subword complexity,
- subword-complexity profile,
- linguistic complexity if mathematically settled.

---

## v0.5 — Compression and Information Distance

Add:

- compressor trait,
- optional compressor backends,
- compressed length,
- compression ratio,
- compression complexity,
- NCD,
- symmetric NCD variant,
- compressor-overhead experiments.

---

## v0.6 — Statistical Complexity

Potential additions:

- approximate entropy,
- sample entropy,
- selected explicitly named statistical-complexity measures.

Do not rush this phase.

---

## v0.7 — Performance and Streaming

Add:

- incremental counters,
- streaming entropy APIs,
- performance optimization,
- optional parallel processing,
- SIMD only where justified.

Publish reproducible benchmarks.

---

## v0.8 — Research API

Add:

- reusable sequence-analysis context,
- metric traits,
- profiles,
- batch APIs,
- distance matrices,
- generic symbol sequences.

---

## v0.9 — Ecosystem

Add:

- CLI,
- Python bindings,
- C ABI,
- optional WASM package.

---

## v1.0 — Stable Mathematical Library

Requirements:

- stable API,
- complete mathematical conventions,
- documented estimators,
- reproducible benchmark suite,
- reference test vectors,
- extensive property testing,
- no ambiguous metric naming,
- cited literature for every public algorithm,
- backward-compatibility policy.

---

# 31. Research Questions Inside the Library Project

Even though the library is domain-independent, several research questions naturally arise.

## 30.1 Short-sequence entropy estimation

How biased are common entropy estimators on sequences of:

```text
10
20
50
100
500
```

symbols?

Possible future implementations:

- Miller-Madow correction,
- Bayesian estimators,
- Grassberger-style estimators,
- other finite-sample corrections.

These should live in an `estimators` module and be explicitly named.

---

## 30.2 Complexity of short strings

Compression proxies are problematic for short sequences because compressor overhead can dominate the data.

Research:

- dictionary effects,
- framing overhead,
- compressor choice,
- context reuse,
- normalized formulations.

This could become an independent paper or technical report.

---

## 30.3 Rényi spectrum

Rather than one \(\alpha\), represent:

\[
\alpha
\mapsto
H_\alpha(X)
\]

for many values.

Potential API:

```rust
renyi_spectrum(sequence, alphas)
```

Research questions:

- numerical stability,
- useful default orders,
- characteristic shapes of sequence families.

---

## 30.4 Block-entropy profile

Study:

\[
H_1,H_2,\ldots,H_n
\]

and derived quantities such as:

\[
H_n-H_{n-1}
\]

These approximate additional information contributed by increasing context.

---

## 30.5 Complexity profile

A sequence may eventually be characterized by:

\[
\Phi(x)=
[
H,
H_{0.5},
H_2,
H_\infty,
H_2^{block},
H_3^{block},
h,
LZ,
p(1),
p(2),
\ldots
]
\]

The library can expose the measurements while leaving interpretation to downstream applications.

---

# 32. Potential Future Metrics

These should be placed in a backlog rather than implemented immediately.

### Entropy / Information

- cross entropy,
- conditional Rényi entropy,
- Rényi divergence,
- Tsallis divergence,
- f-divergences,
- variation of information,
- transfer entropy,
- interaction information,
- dual total correlation.

### Complexity

- grammar-based complexity,
- context-tree complexity,
- finite-state complexity,
- logical-depth approximations,
- sophistication proxies,
- effective complexity.

### Statistical / Sequence

- permutation entropy,
- dispersion entropy,
- fuzzy entropy,
- multiscale entropy,
- recurrence measures,
- correlation dimension.

Every candidate must first answer:

> Does this metric make mathematical sense for the library's supported sequence type?

---

# 33. What Not to Add

Reject scope creep.

Do not add to the core:

```text
HTTP parsing
security classifiers
ML models
embeddings
neural networks
CVE analysis
attack signatures
visual dashboards
database storage
REST APIs
distributed execution
LLMs
application-specific anomaly scores
```

Those should be separate consumers.

---

# 34. Quality Gates for Every New Metric

No metric is merged until all of these exist:

- [ ] Mathematical definition
- [ ] Exact convention identified
- [ ] Original or authoritative reference
- [ ] Public API name is unambiguous
- [ ] Input domain documented
- [ ] Output range documented where known
- [ ] Edge cases defined
- [ ] Time complexity documented
- [ ] Space complexity documented
- [ ] Hand-calculated test
- [ ] Property tests where applicable
- [ ] Cross-check against independent implementation/reference
- [ ] Benchmark
- [ ] API documentation
- [ ] Changelog entry

---

# 35. Initial Issue Backlog

A useful first GitHub milestone could contain:

## Foundation

- [ ] Define mathematical conventions
- [ ] Define empty-sequence policy
- [ ] Define logarithm-base API
- [ ] Implement byte histogram
- [ ] Implement generic count table
- [ ] Implement validated probability distribution
- [ ] Implement n-grams
- [ ] Establish benchmark harness
- [ ] Establish property-test harness

## Entropy

- [ ] Shannon
- [ ] Hartley
- [ ] Rényi
- [ ] Collision
- [ ] Min-entropy
- [ ] Tsallis

## Docs

- [ ] Shannon reference page
- [ ] Rényi reference page
- [ ] Tsallis reference page
- [ ] Numerical behavior guide
- [ ] Mathematical conventions guide

## Validation

- [ ] Hand-verifiable examples
- [ ] Compare against Python/SciPy reference calculations
- [ ] Fuzz public entropy APIs
- [ ] Benchmark byte inputs from 16 B to 100 MB

That is enough for a meaningful v0.1.

---

# 36. Suggested Development Order

Do not develop in the order that sounds most exciting.

Use this dependency order:

```text
1. Mathematical conventions
2. Byte counting
3. Distribution abstraction
4. Shannon
5. Hartley
6. Rényi
7. Collision + min entropy
8. Tsallis
9. n-grams
10. joint distributions
11. joint/conditional entropy
12. mutual information
13. divergences
14. block entropy
15. context statistics
16. entropy-rate estimators
17. LZ76
18. subword complexity
19. compression abstraction
20. NCD
21. statistical complexity
22. streaming
23. generic symbols
24. bindings / CLI
```

---

# 37. First Six-Milestone Plan

## Milestone A — Mathematical Constitution

Before production code:

- establish definitions,
- choose conventions,
- resolve empty-input behavior,
- choose logarithm defaults,
- create bibliography,
- create test vectors.

Deliverable:

```text
docs/mathematical-conventions.md
```

---

## Milestone B — Distribution Engine

Implement and benchmark:

```text
byte histogram
probability distribution
support size
n-grams
joint distributions
```

Deliverable:

```text
src/distribution/
```

---

## Milestone C — Entropy Core

Implement:

```text
Shannon
Hartley
Rényi
collision
min entropy
Tsallis
```

Deliverable:

```text
v0.1
```

---

## Milestone D — Information and Divergence

Implement:

```text
joint entropy
conditional entropy
surprisal
mutual information
KL
JSD
JS distance
Hellinger
TV
```

Deliverable:

```text
v0.2
```

---

## Milestone E — Finite Sequence Structure

Implement:

```text
block entropy
conditional entropy by order
entropy-rate estimators
LZ76
subword complexity
```

Deliverable:

```text
v0.3-v0.4
```

---

## Milestone F — Algorithmic Proxies

Implement:

```text
compressor abstraction
compression complexity
NCD
```

Deliverable:

```text
v0.5
```

At this point the project has enough breadth to be genuinely distinctive.

---

# 38. Long-Term Identity of the Project

The project should not present itself as:

> "A Rust entropy crate."

It should present itself as:

> **A rigorous, high-performance toolkit for information-theoretic and complexity analysis of finite discrete sequences.**

A more academic description:

> **A computational library of information-theoretic, statistical, symbolic, and algorithmic complexity measures for finite sequences.**

A shorter README tagline:

> **Entropy, information, complexity, and distance for finite sequences.**

---

# 39. Success Criteria

The project is successful when a researcher can:

```rust
let x = b"ABABABXYZXYZ";

let h = shannon(x);
let r = renyi(x, 2.0);
let m = min_entropy(x);
let lz = lz76_complexity(x);
```

and trust that:

1. each result follows a documented mathematical convention,
2. every metric is linked to an authoritative reference,
3. edge cases are predictable,
4. performance is competitive,
5. the same library can be used on arbitrary finite symbolic data,
6. no domain-specific assumptions contaminate the mathematics.

The long-term ambition should be that when someone asks:

> "How much information, randomness, structure, or algorithmic complexity does this finite sequence contain?"

this library is one of the first serious tools they reach for.

---

# 40. Immediate Next Steps

The first work session should produce only these artifacts:

1. `README.md`
2. `docs/mathematical-conventions.md`
3. `docs/references.md`
4. `src/distribution/`
5. `src/entropy/shannon.rs`
6. canonical entropy test vectors
7. initial benchmark harness

Do **not** begin with twenty algorithms.

Make Shannon entropy and the underlying distribution engine exceptionally correct, fast, and documented.

Then grow outward.

---

# 41. Recommended First Research Reading While Building

### Stage 1 — Core information theory

Read primarily:

- Cover & Thomas
- MacKay

Implement:

```text
distribution
Shannon
joint entropy
conditional entropy
mutual information
KL divergence
```

### Stage 2 — Generalized entropy

Read:

- Rényi
- Aczél & Daróczy
- Leinster
- selected Tsallis literature

Implement:

```text
Hartley
Rényi
collision
min entropy
Tsallis
```

### Stage 3 — Sequences

Read:

- Gray
- Lind & Marcus

Implement:

```text
block entropy
entropy rate
subword complexity
excess entropy
```

### Stage 4 — Algorithmic information

Read:

- Li & Vitányi

Implement / research:

```text
algorithmic information concepts
compression proxies
information distance
NCD
```

### Stage 5 — Compression

Read:

- Sayood

Implement:

```text
compressor abstraction
LZ-related metrics
compression complexity
```

### Stage 6 — Description length

Read:

- Grünwald

Explore later:

```text
MDL-oriented utilities
model-description interfaces
research integrations
```

MDL should not be forced into the initial sequence-metric API.

---

# Final Principle

The guiding question for every addition should be:

> **What precisely does this measure say about a finite sequence, under what assumptions, and can we compute or estimate it honestly?**

If that question cannot be answered clearly, the metric does not belong in the library yet.
