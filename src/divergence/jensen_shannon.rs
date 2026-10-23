use super::EmptyDistribution;
use crate::distribution::Distribution;
use std::f64::consts::LN_2;

#[doc = include_str!("../../docs/divergence/jensen_shannon.md")]
pub fn jensen_shannon_divergence(x: &[u8], y: &[u8]) -> Result<f64, EmptyDistribution> {
    if x.is_empty() || y.is_empty() {
        return Err(EmptyDistribution);
    }
    jensen_shannon_divergence_distribution(
        &Distribution::from_bytes(x),
        &Distribution::from_bytes(y),
    )
}

/// Equal-weight Jensen–Shannon divergence in bits, reusing empirical counts.
///
/// Computes `J(P,Q) = H((P+Q)/2) - (H(P)+H(Q))/2` without subtracting rounded
/// entropies. The mathematical range is `[0,1]`; equal laws return positive
/// zero and disjoint supports return one. Different sample sizes are allowed.
/// Uses `O(256)` time, constant auxiliary storage, and no heap allocation.
/// See [`jensen_shannon_divergence`] for definitions, history, and numerical limits.
///
/// # Errors
/// Returns [`EmptyDistribution`] if either law is empty, including both.
///
/// ```
/// use celandine::distribution::Distribution;
/// use celandine::divergence::jensen_shannon_divergence_distribution;
/// let p = Distribution::from_bytes(b"A");
/// let q = Distribution::from_bytes(b"AB");
/// let j = jensen_shannon_divergence_distribution(&p, &q).unwrap();
/// assert!((j - 0.31127812445913283).abs() < 1e-14);
/// ```
pub fn jensen_shannon_divergence_distribution(
    p: &Distribution,
    q: &Distribution,
) -> Result<f64, EmptyDistribution> {
    if p.is_empty() || q.is_empty() {
        return Err(EmptyDistribution);
    }
    let n = p.sample_size() as u128;
    let m = q.sample_size() as u128;
    let denominator = (n * m) as f64;
    let mut sum = 0.0;
    let mut correction = 0.0;
    let mut overlap = false;
    for (&a, &b) in p.histogram().counts().iter().zip(q.histogram().counts()) {
        overlap |= a > 0 && b > 0;
        // Products fit u128 on <=64-bit usize targets. Their SUM may not, so
        // add converted products; subtract in integers to retain tiny gaps.
        let left = (a as u128) * m;
        let right = (b as u128) * n;
        if left == right {
            continue;
        }
        let mass = left as f64 + right as f64;
        let delta = left.abs_diff(right) as f64 / mass;
        let term = (0.5 * mass / denominator) * binary_gap(delta);
        // Compensated summation of nonnegative per-symbol contributions.
        let adjusted = term - correction;
        let next = sum + adjusted;
        correction = (next - sum) - adjusted;
        sum = next;
    }
    Ok(if overlap { sum } else { 1.0 })
}

// 1 - binary entropy((1+d)/2), evaluated without entropy cancellation.
fn binary_gap(d: f64) -> f64 {
    if d <= 0.5 {
        // g(d) = sum_{k>=1} d^(2k)/(2k*(2k-1))/ln(2).
        // At d<=1/2, truncation after k=32 is below 1e-21 relative;
        // early termination only skips terms below floating addition resolution.
        let square = d * d;
        let mut power = square;
        let mut sum = square / 2.0;
        for k in 2..=32 {
            power *= square;
            let next = sum + power / f64::from(2 * k * (2 * k - 1));
            if next == sum {
                break;
            }
            sum = next;
        }
        sum / LN_2
    } else if d >= 1.0 {
        // Includes rounding to one; do not evaluate 0*ln(0).
        1.0
    } else {
        ((1.0 + d) * d.ln_1p() + (1.0 - d) * (-d).ln_1p()) / (2.0 * LN_2)
    }
}

#[doc = include_str!("../../docs/divergence/jensen_shannon_distance.md")]
pub fn jensen_shannon_distance(x: &[u8], y: &[u8]) -> Result<f64, EmptyDistribution> {
    jensen_shannon_divergence(x, y).map(f64::sqrt)
}

/// Square root of equal-weight, base-2 Jensen–Shannon divergence, reusing counts.
///
/// `d(P,Q) = sqrt(J(P,Q))` is a metric on probability laws, in square-root-bit
/// units with mathematical range `[0,1]`. Ordering of sample bytes is ignored.
/// Uses `O(256)` time, constant auxiliary storage, and no heap allocation.
/// See [`jensen_shannon_distance`] for the interpretation and conventions.
///
/// # Errors
/// Returns [`EmptyDistribution`] if either law is empty, including both.
///
/// ```
/// use celandine::distribution::Distribution;
/// use celandine::divergence::jensen_shannon_distance_distribution;
/// let p = Distribution::from_bytes(b"AA");
/// let q = Distribution::from_bytes(b"BB");
/// assert_eq!(jensen_shannon_distance_distribution(&p, &q), Ok(1.0));
/// ```
pub fn jensen_shannon_distance_distribution(
    p: &Distribution,
    q: &Distribution,
) -> Result<f64, EmptyDistribution> {
    jensen_shannon_divergence_distribution(p, q).map(f64::sqrt)
}
