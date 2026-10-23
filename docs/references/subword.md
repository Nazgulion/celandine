# Subword complexity references and independent verification

Morse and Hedlund's *Symbolic Dynamics* (1938), American Journal of Mathematics
60(4), 815–866, [DOI 10.2307/2371264](https://doi.org/10.2307/2371264), is the
historical source for block growth in symbolic dynamics. Cassaigne and Nicolas's
2010 chapter [Factor complexity](https://doi.org/10.1017/CBO9780511777653.005),
pp. 163–247 in *Combinatorics, Automata and Number Theory*, explains the
terminology and infinite-word theory. Celandine counts factors of a finite
sample; it makes no inference about an unknown infinite continuation.

The [metric guide](../complexity/subword.md) defines the finite extension,
including the single empty word and zero counts beyond the sample length.
For `ABABA`, the two length-two factors are `AB` and `BA`; occurrence counts
are irrelevant once a factor is present. Results are integer counts, without
approximation, smoothing, or logarithms.

```sh
python3 scripts/reference_subword.py --check
cargo test --locked --test subword
cargo run --locked --example subword
```

The Python reference enumerates all start/end pairs once into a set, then groups
by length. It generates **264 complete profiles**, including the first oversized
length: all binary strings of lengths zero through seven (255), plus nine worked,
UTF-8 byte, arbitrary binary, repeated, and all-256-byte samples. It does not call
Rust or reuse the library's per-length tree tally. Every entry requires exact
integer equality.

Rust additionally exhausts all 1,023 binary strings through length nine using
explicit positional comparisons: a block counts only if no earlier position
matches every byte. Fixed-seed properties check this independent definition,
reversal, bijective XOR renaming, length bounds, byte support, and extension
monotonicity. Iterator tests cover lazy prefix selection, cloning, reverse
consumption, exact remaining length, extreme skipping, and fused exhaustion.
The isolated allocation test checks scalar fast paths and skipping of unrequested
lengths; the memory probe checks temporary tree allocations and reclamation.

The [bounded fuzz target](../fuzzing.md) uses sorted explicit slices and deduplication
as a third counting approach, checking whole profiles on up to 64 bytes. These
finite tests do not establish asymptotic theorems or exhaustive correctness for
arbitrary inputs; the implementation also relies on the reviewed n-gram counter.

## Retained skipping regression

The first profile implementation used a mapped range. On Rust 1.90, `nth`
executed the mapping closure for skipped entries, constructing unwanted count
tables. A focused allocation regression on 16 distinct bytes expected zero
allocations when selecting only the full-length entry, but observed 22 before
the fix. The private profile iterator now advances the range before evaluating
only the selected length. The same regression passes with zero allocations;
reverse and oversized skipping are also covered. Ordinary debug/release tests
retain this contract in `tests/allocations.rs`.
