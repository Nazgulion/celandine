# Compression measurement references

[Metric definition and examples](../compression.md) describe C(x) in bytes and
R(x) = C(x)/|x|, with full framing and explicit empty-ratio errors.

- Deutsch and Gailly, [RFC 1950](https://www.rfc-editor.org/rfc/rfc1950), May 1996,
  specifies zlib headers, dictionary flag, Adler-32, and stream termination.
- Deutsch, [RFC 1951](https://www.rfc-editor.org/rfc/rfc1951), May 1996, specifies
  stored/fixed/dynamic blocks, LZ77 distances and Huffman-coded symbols.
- [Pinned backend API](https://docs.rs/miniz_oxide/0.9.1/miniz_oxide/deflate/core/index.html).
- [Python zlib](https://docs.python.org/3/library/zlib.html) supplies the independent
  C-zlib decoder for `scripts/reference_compression.py --check`.

The format does not determine an encoder's output size. We verify actual counted
streams with a separate decoder, hand-check the empty and single-A encodings,
and compare streaming counts with materialized miniz output over generated inputs.
The latter is a buffer/finalization check, not an independent codec implementation.

There is one fixed backend/configuration. Raw DEFLATE, gzip, alternative levels,
other backends, dictionaries, overhead calibration, and NCD remain deferred.
The measurement is an encoder-specific complexity proxy, not Kolmogorov complexity.
