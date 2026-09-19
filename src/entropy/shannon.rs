use crate::distribution::Distribution;

#[doc = include_str!("../../docs/entropy/shannon.md")]
pub fn shannon(data: &[u8]) -> f64 {
    shannon_distribution(&Distribution::from_bytes(data))
}

/// Computes Shannon entropy in bits from a previously counted distribution.
///
/// Uses `H = -sum(p * log2(p))`, skipping zero probabilities. Empty and constant
/// samples return positive zero. The mathematical range for nonempty samples
/// is `0..=log2(support_size)`, at most 8 bits, subject to rounding.
/// No probabilities are normalized, smoothed, or clamped.
///
/// This takes `O(256)` time and `O(1)` additional space without allocating.
/// See [`shannon`] for interpretation, numerical notes, and references.
///
/// ```
/// use celandine::distribution::{ByteHistogram, Distribution};
/// use celandine::entropy::shannon_distribution;
///
/// let counts = ByteHistogram::from_bytes(b"ABCD");
/// let distribution = Distribution::from_counts(counts);
/// assert_eq!(shannon_distribution(&distribution), 2.0);
/// ```
pub fn shannon_distribution(distribution: &Distribution) -> f64 {
    distribution
        .probabilities()
        .filter(|&p| p > 0.0)
        .fold(0.0, |entropy, p| entropy - p * p.log2())
}
