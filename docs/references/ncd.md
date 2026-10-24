# Normalized compression distance references

Cilibrasi and Vitányi, *Clustering by Compression*, IEEE Transactions on
Information Theory 51(4), 1523–1545 (2005),
[author's corrected version](https://homepages.cwi.nl/~paulv/papers/cluster.pdf),
§§III and VI, equation VI.1; [2003 preprint](https://arxiv.org/abs/cs/0312044).

The [API definition](../compression/ncd.md) fixes ordered concatenation and
retains complete-stream framing. Metric guarantees in the theory have compressor
assumptions; arbitrary real encoders need not obey them. Symmetric variants and
overhead corrections are not implemented implicitly.

Independent verification uses Python Fraction for signed size differences and
ratios, a hand-verifiable literal format, and C-zlib decoding of counted streams.
Cross-encoder compressed sizes are not expected to agree. No source entropy,
Kolmogorov complexity, or clustering model is estimated by this function.
