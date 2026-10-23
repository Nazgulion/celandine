use crate::distribution::Distribution;
use std::fmt;

/// At least one input has no observations and therefore no probability law.
///
/// Total variation requires two nonempty distributions, including when both
/// inputs are empty. No probabilities are inferred from an empty sample.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmptyDistribution;

impl fmt::Display for EmptyDistribution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("total variation requires two nonempty distributions")
    }
}

impl std::error::Error for EmptyDistribution {}

#[doc = include_str!("../../docs/divergence/total_variation.md")]
pub fn total_variation_distance(x: &[u8], y: &[u8]) -> Result<f64, EmptyDistribution> {
    if x.is_empty() || y.is_empty() {
        return Err(EmptyDistribution);
    }
    total_variation_distance_distribution(
        &Distribution::from_bytes(x),
        &Distribution::from_bytes(y),
    )
}

/// Dimensionless total variation distance, reusing two validated count tables.
///
/// Computes `0.5 * sum_i |p_i - q_i|` on the shared byte alphabet, in `[0, 1]`.
/// Counts are compared exactly before the final floating-point conversion.
/// Equal laws return positive zero; disjoint supports return one. Sample sizes
/// may differ. Uses `O(256)` time, constant auxiliary storage, and no allocation.
/// See [`total_variation_distance`] for interpretation, history, and limits.
///
/// # Errors
/// Returns [`EmptyDistribution`] if either distribution is empty, including both.
///
/// ```
/// use celandine::distribution::Distribution;
/// use celandine::divergence::total_variation_distance_distribution;
/// let p = Distribution::from_bytes(b"AAAB");
/// let q = Distribution::from_bytes(b"AB");
/// assert_eq!(total_variation_distance_distribution(&p, &q), Ok(0.25));
/// ```
pub fn total_variation_distance_distribution(
    p: &Distribution,
    q: &Distribution,
) -> Result<f64, EmptyDistribution> {
    if p.is_empty() || q.is_empty() {
        return Err(EmptyDistribution);
    }
    let n = p.sample_size() as u128;
    let m = q.sample_size() as u128;
    let mut positive = 0u128;
    for (&a, &b) in p.histogram().counts().iter().zip(q.histogram().counts()) {
        // TV is the sum of positive probability differences. For <=64-bit
        // usize, each product fits u128. Their positive differences sum to at
        // most n*m, so the accumulator fits too (the full L1 numerator may not).
        // Subtracting before conversion preserves differences below f64's
        // resolution near one, and makes proportional count tables exact zero.
        positive += ((a as u128) * m).saturating_sub((b as u128) * n);
    }
    Ok(positive as f64 / (n * m) as f64)
}
