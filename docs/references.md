# References

- R. V. L. Hartley (1928), *Transmission of Information*, Bell System Technical
  Journal 7(3), 535–563.
  [Original paper DOI](https://doi.org/10.1002/j.1538-7305.1928.tb01236.x).
  Establishes logarithmic counting of distinguishable possibilities.
- Alfréd Rényi (1961), *On Measures of Entropy and Information*, Proceedings of
  the Fourth Berkeley Symposium on Mathematical Statistics and Probability,
  volume 1, 547–561.
  [Paper](https://static.renyi.hu/renyi_cikkek/1961_on_measures_of_entropy_and_information.pdf).
  Defines the implemented Rényi family, including Shannon and log-support limits.
- Claude E. Shannon (1948), *A Mathematical Theory of Communication*, Bell
  System Technical Journal 27, 379–423 and 623–656. Part I, section 6,
  “Choice, Uncertainty and Entropy.”
  [Paper, reprinted with corrections](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf).
  Defines discrete entropy and its basic properties; base 2 gives bits.
- Thomas M. Cover and Joy A. Thomas (2006), *Elements of Information Theory*,
  second edition, Wiley, chapter 2, especially section 2.1. ISBN 978-0-471-24195-9.
  Textbook treatment of entropy, the zero-probability convention, and bounds.
- David J. C. MacKay (2003), *Information Theory, Inference, and Learning
  Algorithms*, Cambridge University Press, chapter 2. ISBN 978-0-521-64298-9.
  Textbook treatment of probability and information measures.

The implementation follows the mathematical definition, not the behavior of a
third-party entropy library. Empty-sequence behavior is a project convention.
See [Shannon's reference notes](references/shannon.md) and
[Hartley's reference notes](references/hartley.md), with their corresponding
[Shannon](entropy/shannon.md) and [Hartley](entropy/hartley.md) metric pages.

See [Rényi reference notes](references/renyi.md) and its
[metric page](entropy/renyi.md) for the definition, limits, and numerical sources.

[Collision reference notes](references/collision.md) explain the order-two
specialization and its exact matching-pair verification.

[Min-entropy reference notes](references/min_entropy.md) derive the infinite-order
limit and explain the maximum-probability interpretation and verification.

- C. Tsallis (1988), *Possible Generalization of Boltzmann–Gibbs Statistics*,
  Journal of Statistical Physics 52, 479–487.
  [Original paper DOI](https://doi.org/10.1007/BF01016429).
  Defines the entropy family with a conventional positive scaling constant.
  [Tsallis reference notes](references/tsallis.md) explain this library's fixed
  Shannon-bit scaling, order-one limit, and independent verification.

- Christopher D. Manning, Prabhakar Raghavan, and Hinrich Schütze (2008),
  *Introduction to Information Retrieval*, Cambridge University Press,
  section 3.2.2, [k-gram indexes](https://nlp.stanford.edu/IR-book/html/htmledition/k-gram-indexes-for-wildcard-queries-1.html).
  Illustrates overlapping contiguous character blocks. The [n-gram guide](ngrams.md)
  explains the byte-based, unpadded convention and independent verification.
- Rust standard library: [slice windows](https://doc.rust-lang.org/std/primitive.slice.html#method.windows)
  and [BTreeMap](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html).
  Storage/iterator references for borrowed extraction and sorted counts;
  mathematical and empty-state conventions are documented separately.

- David A. Levin and Yuval Peres, with contributions by Elizabeth L. Wilmer
  (2017), *Markov Chains and Mixing Times*, second edition, AMS, Chapter 4,
  Proposition 4.2 and Remark 4.3.
  [Author-hosted book](https://pages.uoregon.edu/dlevin/MARKOV/mcmt2e.pdf).
  [Total variation reference notes](references/total_variation.md) document the
  half-L1 convention, exact count derivation, empty errors, and verification.

- Jianhua Lin (1991), *Divergence Measures Based on the Shannon Entropy*, IEEE
  Transactions on Information Theory 37(1), 145–151.
  [DOI](https://doi.org/10.1109/18.61115).
- Dominik M. Endres and Johannes E. Schindelin (2003), *A New Metric for
  Probability Distributions*, IEEE Transactions on Information Theory 49(7),
  1858–1860. [DOI](https://doi.org/10.1109/TIT.2003.813506).
  [Jensen–Shannon reference notes](references/jensen_shannon.md) explain the
  equal-weight convention, square-root metric, and stable count evaluation.

- Marston Morse and Gustav A. Hedlund (1938), *Symbolic Dynamics*, American
  Journal of Mathematics 60(4), 815–866. [DOI](https://doi.org/10.2307/2371264).
- Julien Cassaigne and François Nicolas (2010), *Factor complexity*, in
  *Combinatorics, Automata and Number Theory*, pp. 163–247.
  [Publisher chapter](https://doi.org/10.1017/CBO9780511777653.005).
  [Subword reference notes](references/subword.md) explain the finite-sample
  convention and independent exact verification.

- Abraham Lempel and Jacob Ziv (1976), *On the Complexity of Finite Sequences*,
  IEEE Transactions on Information Theory 22(1), 75–81.
  [DOI](https://doi.org/10.1109/TIT.1976.1055501).
  [LZ76 reference notes](references/lz76.md) identify the exhaustive convention,
  accessible primary expositions, published parses, and independent verification.

## Compression measurements

[Definition and implementation conventions](compression.md). Primary format
references: Deutsch and Gailly, [RFC 1950 (May 1996)](https://www.rfc-editor.org/rfc/rfc1950),
and Deutsch, [RFC 1951 (May 1996)](https://www.rfc-editor.org/rfc/rfc1951).
DEFLATE combines LZ77 and Huffman coding; zlib adds framing and Adler-32.
The chosen encoder is [miniz_oxide 0.9.1](https://docs.rs/miniz_oxide/0.9.1/).
See [verification details](references/compression.md).
