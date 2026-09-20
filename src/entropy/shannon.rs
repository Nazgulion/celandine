use crate::distribution::Distribution;
use std::fmt;

/// A logarithm base that is not finite and strictly greater than one.
///
/// Entropy units require an increasing logarithm. Zero, one, bases below one,
/// NaN, and infinities are rejected, even for empty input. For example,
/// `shannon_with_base(b"AB", 1.0)` returns this error instead of dividing by zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidLogBase;

impl fmt::Display for InvalidLogBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("logarithm base must be finite and greater than one")
    }
}

impl std::error::Error for InvalidLogBase {}

fn validate_base(base: f64) -> Result<(), InvalidLogBase> {
    if !base.is_finite() || base <= 1.0 {
        return Err(InvalidLogBase);
    }
    Ok(())
}

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

#[doc = include_str!("../../docs/entropy/shannon_base.md")]
pub fn shannon_with_base(data: &[u8], base: f64) -> Result<f64, InvalidLogBase> {
    validate_base(base)?;
    Ok(shannon(data) / base.log2())
}

/// Computes Shannon entropy in selected units from an existing distribution.
///
/// Uses `H_base = H_bits / log2(base)`, where `H_bits` is the existing empirical
/// Shannon result. Base 2 gives bits, base `E` gives nats, and base 10 gives
/// decimal information units, all per symbol. This reuses counts without
/// allocation: `O(256)` time and `O(1)` extra space. For example, four equally
/// likely symbols have entropy `log_base(4)`.
///
/// Empty and constant samples return positive zero after validation. Base 2
/// agrees bit-for-bit with [`shannon_distribution`]. Other bases inherit that
/// calculation's rounding error, scaled by `1/log2(base)`.
/// See [`shannon_with_base`] for the definition, history, range, and limits.
///
/// # Errors
/// Returns [`InvalidLogBase`] unless `base` is finite and greater than one.
/// Validation precedes empty/constant handling.
///
/// ```
/// use celandine::distribution::Distribution;
/// use celandine::entropy::shannon_distribution_with_base;
/// let d = Distribution::from_bytes(b"ABCD");
/// assert_eq!(shannon_distribution_with_base(&d, 2.0), Ok(2.0));
/// assert!((shannon_distribution_with_base(&d, 10.0)? - 0.6020599913279624).abs() < 1e-12);
/// # Ok::<(), celandine::entropy::InvalidLogBase>(())
/// ```
pub fn shannon_distribution_with_base(
    distribution: &Distribution,
    base: f64,
) -> Result<f64, InvalidLogBase> {
    validate_base(base)?;
    Ok(shannon_distribution(distribution) / base.log2())
}
