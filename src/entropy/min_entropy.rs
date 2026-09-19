use super::renyi_entropy_distribution;
use crate::distribution::Distribution;

#[doc = include_str!("../../docs/entropy/min_entropy.md")]
pub fn min_entropy(data: &[u8]) -> f64 {
    min_entropy_distribution(&Distribution::from_bytes(data))
}

/// Min-entropy in bits per symbol, reusing validated empirical counts.
///
/// Returns `-log2(max(p_i))`, the surprise of the most likely observed symbol.
/// Empty and constant inputs return positive zero. Uses `O(256)` time and
/// constant auxiliary storage without allocating or recounting the sequence.
/// See [`min_entropy`] for interpretation, numerical behavior, and conventions.
///
/// ```
/// use celandine::distribution::Distribution;
/// use celandine::entropy::min_entropy_distribution;
/// let d = Distribution::from_bytes(b"AAAB");
/// assert!((min_entropy_distribution(&d) - 0.4150374992788438).abs() < 1e-12);
/// ```
pub fn min_entropy_distribution(distribution: &Distribution) -> f64 {
    // Positive infinity is always a valid order; share the stable count-ratio log.
    renyi_entropy_distribution(distribution, f64::INFINITY)
        .expect("positive infinity is always a valid Rényi order")
}
