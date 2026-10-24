//! Information-theoretic measures for finite discrete byte sequences.
//!
//! Provides allocation-free byte counting, reusable empirical distributions,
//! and Shannon, Hartley, Rényi, collision, and min-entropy in bits per symbol.
//! Tsallis entropy uses a fixed scale agreeing with Shannon at order one.
//! Shannon also accepts explicit logarithm bases greater than one through
//! [`entropy::shannon_with_base`] and [`entropy::shannon_distribution_with_base`].
//! Entropy of empty input returns zero by convention. No source model is assumed.
//! [`divergence::total_variation_distance`] compares byte frequencies in two
//! nonempty samples, returning a dimensionless distance in `[0, 1]`.
//! Equal-weight [`divergence::jensen_shannon_divergence`] compares them through
//! a mixture in bits; [`divergence::jensen_shannon_distance`] returns its square root.
//! Overlapping byte n-grams are available through [`transforms::ngrams`], with
//! count-backed n-gram distributions in [`distribution`].
//! [`transforms::windows`] borrows complete windows at a positive byte width and
//! stride, with exact length, reverse traversal and constant-time skips.
//! [`complexity::subword_complexity`] counts distinct contiguous blocks exactly;
//! [`complexity::subword_complexity_profile`] lazily covers lengths zero through L.
//! [`complexity::lz76_complexity`] counts exhaustive LZ76 production phrases
//! with overlapping copies and a counted final suffix, without heap allocation.
//! [`compression`] measures complete encoded byte lengths and ratios through a
//! dependency-free compressor interface, including ordered normalized compression
//! distance with inspectable component sizes; `compression-deflate` enables fixed zlib.
//! Extraction borrows
//! the input without allocation; n-gram count tables allocate tree storage.
//!
//! ```
//! use celandine::distribution::Distribution;
//! use celandine::entropy::{shannon, shannon_distribution};
//!
//! assert_eq!(shannon(b"ABCD"), 2.0);
//! let distribution = Distribution::from_bytes(b"ABAB");
//! assert_eq!(distribution.sample_size(), 4);
//! assert_eq!(distribution.support_size(), 2);
//! assert_eq!(distribution.probability(b'A'), 0.5);
//! assert_eq!(shannon_distribution(&distribution), 1.0);
//! ```

#![forbid(unsafe_code)]

pub mod complexity;
pub mod compression;
pub mod distribution;
pub mod divergence;
pub mod entropy;
pub mod transforms;
