//! Information-theoretic measures for finite discrete byte sequences.
//!
//! Provides allocation-free byte counting, reusable empirical distributions,
//! and Shannon, Hartley, Rényi, collision, and min-entropy in bits per symbol.
//! Empty input returns zero by convention. No source model is assumed.
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

pub mod distribution;
pub mod entropy;
