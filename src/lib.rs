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
//! [`distribution::JointDistribution`] retains aligned byte pairs and both marginals.
//! [`entropy::joint_entropy`] measures their combined uncertainty in bits per pair.
//! [`entropy::conditional_entropy`] measures first-coordinate uncertainty given
//! the second aligned byte, with allocation-free joint-law reuse.
//! [`information::mutual_information`] measures their shared information in bits per pair.
//! [`entropy::block_entropy`] measures their Shannon entropy in bits per block.
//! [`entropy::block_entropy_profile`] lazily evaluates a bounded range of block lengths.
//! [`entropy::conditional_entropy_order`] measures next-byte uncertainty given
//! preceding context, using matched observations and bits per predicted byte.
//! [`transforms::windows`] borrows complete windows at a positive byte width and
//! stride, with exact length, reverse traversal and constant-time skips.
//! [`transforms::blocks`] yields adjacent complete blocks with an inspectable tail.
//! [`complexity::subword_complexity`] counts distinct contiguous blocks exactly;
//! [`complexity::subword_complexity_profile`] lazily covers lengths zero through L.
//! [`complexity::lz76_complexity`] counts exhaustive LZ76 production phrases
//! with overlapping copies and a counted final suffix, without heap allocation.
//! [`complexity::Lz76Summary`] reuses one parse for phrase density and explicit
//! asymptotic normalization; the latter can exceed one on finite inputs.
//! [`complexity::normalized_jensen_shannon_complexity`] combines normalized byte
//! entropy and departure from uniformity on a caller-declared alphabet.
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
pub mod information;
pub mod transforms;
