# Joint distributions and entropy: references and verification

C. E. Shannon (1948), *A Mathematical Theory of Communication*, Bell System
Technical Journal 27, 379–423 and 623–656,
[original paper](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf).
Section 6, property 3 defines entropy of a joint event and the upper bound by
the sum of marginal entropies. The library evaluates that definition on aligned
byte observations. Equal-length validation, owned sparse storage, and the empty
convention are implementation choices. See [the distribution guide](../joint-distributions.md)
and [joint entropy](../entropy/joint.md).

`python3 scripts/reference_joint.py --check` regenerates 1,387 independent
fixtures. Python Counter counts tuples from aligned positions; 100-digit Decimal
arithmetic computes `log2(N) - sum(c*log2(c))/N`. Fixtures cover all binary pair
sequences through length five, four worked/diverse cases, twelve seeded samples
and six unequal-length cases. Rust checks exact pair counts and marginals,
probability queries and joint entropy within 1e-12 absolute bits.

Normal tests additionally cover all 65,536 possible pairs (16 bits), a rare
pair in 1,048,576 observations, owned results outliving inputs, exact empty zeros,
error fields, clone behavior and query order. Seeded properties test symmetry,
diagonal identity, marginal bounds, total/support bounds, joint permutation,
coordinate relabeling and count scaling. Byte/reuse results must match in f64
bits; mathematical identities involving different summation orders use tolerance.
The isolated allocator checks empty/error paths and count-to-distribution moves,
queries, marginalization and entropy reuse without new heap allocation.

Review found no new production defect. Two development-test issues were fixed
before validation: an ambiguous floating literal needed an f64 annotation, and
a range assertion was adjusted to satisfy Clippy. These were harness issues,
not mathematical regressions. Independent fuzzing and archived performance data
supplement this evidence without proving correctness for every possible input.
