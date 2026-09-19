# References

- R. V. L. Hartley (1928), *Transmission of Information*, Bell System Technical
  Journal 7(3), 535–563.
  [Original paper DOI](https://doi.org/10.1002/j.1538-7305.1928.tb01236.x).
  Establishes logarithmic counting of distinguishable possibilities.
- Alfréd Rényi (1961), *On Measures of Entropy and Information*, Proceedings of
  the Fourth Berkeley Symposium on Mathematical Statistics and Probability,
  volume 1, 547–561.
  [Paper](https://static.renyi.hu/renyi_cikkek/1961_on_measures_of_entropy_and_information.pdf).
  Defines the entropy family whose limit at order zero gives log support.
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
