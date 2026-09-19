use super::renyi_entropy_distribution;
use crate::distribution::Distribution;

#[doc = include_str!("../../docs/entropy/collision.md")]
pub fn collision_entropy(data: &[u8]) -> f64 {
    collision_entropy_distribution(&Distribution::from_bytes(data))
}

/// Collision entropy in bits per symbol, reusing validated empirical counts.
///
/// Computes `-log2(sum(p_i^2))`, the negative log probability of an equal-symbol
/// pair from independent draws with replacement. Empty and constant inputs return
/// positive zero. Uses `O(256)` time, constant auxiliary storage, and no allocation.
/// See [`collision_entropy`] for the definition, interpretation, and conventions.
///
/// ```
/// use celandine::distribution::Distribution;
/// use celandine::entropy::collision_entropy_distribution;
/// let d = Distribution::from_bytes(b"AAAB");
/// assert!((collision_entropy_distribution(&d) - 0.6780719051126377).abs() < 1e-12);
/// ```
pub fn collision_entropy_distribution(distribution: &Distribution) -> f64 {
    // Two is always a valid Rényi order; invalid data cannot enter Distribution.
    // Reuse its stable calculation instead of duplicating numerical branches.
    renyi_entropy_distribution(distribution, 2.0).expect("Rényi order two is always valid")
}
