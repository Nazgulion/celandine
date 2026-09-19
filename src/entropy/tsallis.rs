use super::shannon_distribution;
use crate::distribution::Distribution;
use std::f64::consts::LN_2;
use std::fmt;

/// A Tsallis order was negative or nonfinite; finite nonnegative orders are valid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidTsallisOrder;

impl fmt::Display for InvalidTsallisOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Tsallis order must be finite and nonnegative")
    }
}

impl std::error::Error for InvalidTsallisOrder {}

#[doc = include_str!("../../docs/entropy/tsallis.md")]
pub fn tsallis_entropy(data: &[u8], q: f64) -> Result<f64, InvalidTsallisOrder> {
    validate_order(q)?;
    Ok(evaluate(&Distribution::from_bytes(data), q))
}

/// Empirical Tsallis entropy with fixed `1/ln(2)` scaling, reusing exact counts.
///
/// Accepts finite `q>=0`. Order one returns Shannon entropy in bits per symbol;
/// other orders use `(1-sum(p_i^q))/(q-1)/ln(2)` and can exceed eight.
/// Empty and constant inputs return positive zero for valid orders.
/// Takes `O(256)` time and constant auxiliary storage without allocation.
/// See [`tsallis_entropy`] for the definition, scaling, and numerical behavior.
///
/// # Errors
/// Returns [`InvalidTsallisOrder`] for negative or nonfinite orders, even for
/// empty or constant input. Positive infinity is not accepted.
///
/// ```
/// use celandine::distribution::Distribution;
/// use celandine::entropy::tsallis_entropy_distribution;
/// let d = Distribution::from_bytes(b"AB");
/// assert!((tsallis_entropy_distribution(&d, 2.0).unwrap() - 0.5 / std::f64::consts::LN_2).abs() < 1e-12);
/// ```
pub fn tsallis_entropy_distribution(
    distribution: &Distribution,
    q: f64,
) -> Result<f64, InvalidTsallisOrder> {
    validate_order(q)?;
    Ok(evaluate(distribution, q))
}

fn validate_order(q: f64) -> Result<(), InvalidTsallisOrder> {
    if !q.is_finite() || q < 0.0 {
        Err(InvalidTsallisOrder)
    } else {
        Ok(())
    }
}

fn evaluate(d: &Distribution, q: f64) -> f64 {
    if q == 1.0 {
        return shannon_distribution(d);
    }
    let support = d.support_size();
    if support <= 1 {
        return 0.0;
    }
    if q == 0.0 {
        return (support - 1) as f64 / LN_2;
    }
    let total = d.sample_size();
    let t = q - 1.0;
    let mut sum = 0.0;
    let mut correction = 0.0;
    for &count in d.histogram().counts().iter().filter(|&&c| c > 0) {
        let p = count as f64 / total as f64;
        // Subtract exact counts first to preserve a small complement near one.
        let log_p = if count > total / 2 {
            (-((total - count) as f64 / total as f64)).ln_1p()
        } else {
            p.ln()
        };
        let term = if q < 0.5 && count <= total / 2 {
            // Near zero, p * exp((q-1)*ln(p)) amplifies log/exp roundoff for
            // rare symbols. Direct p^q is accurate here; p<=1/2 and q<1/2
            // keep p^q-p away from cancellation relative to p^q.
            p - (q * log_p).exp()
        } else {
            // Keep expm1 near one and for a dominant p whose ratio rounds to 1.
            -p * (t * log_p).exp_m1()
        };
        let adjusted = term - correction;
        let next = sum + adjusted;
        correction = (next - sum) - adjusted;
        sum = next;
    }
    // Scale after summation to retain contributions before large-order division.
    sum / t / LN_2
}
