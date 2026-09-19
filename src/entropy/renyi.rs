use super::{hartley_entropy_distribution, shannon_distribution};
use crate::distribution::Distribution;
use std::f64::consts::LN_2;
use std::fmt;

/// A Rényi order was negative or NaN; nonnegative orders and positive infinity are valid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidRenyiOrder;

impl fmt::Display for InvalidRenyiOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Rényi order must be nonnegative and not NaN")
    }
}

impl std::error::Error for InvalidRenyiOrder {}

#[doc = include_str!("../../docs/entropy/renyi.md")]
pub fn renyi_entropy(data: &[u8], alpha: f64) -> Result<f64, InvalidRenyiOrder> {
    validate_order(alpha)?;
    Ok(evaluate(&Distribution::from_bytes(data), alpha))
}

/// Empirical Rényi entropy in bits per symbol, reusing existing counts.
///
/// Accepts nonnegative orders, including positive infinity. Orders zero and one
/// return Hartley and Shannon respectively. Empty and constant distributions
/// return positive zero for valid orders. Does not allocate or recount bytes.
///
/// # Errors
///
/// Returns [`InvalidRenyiOrder`] for negative or NaN orders, even for empty data.
/// See [`renyi_entropy`] for the definition, conventions, and numerical notes.
///
/// ```
/// use celandine::distribution::Distribution;
/// use celandine::entropy::renyi_entropy_distribution;
/// let d = Distribution::from_bytes(b"AAAB");
/// assert!((renyi_entropy_distribution(&d, 2.0).unwrap() - 0.6780719051126377).abs() < 1e-12);
/// ```
pub fn renyi_entropy_distribution(
    distribution: &Distribution,
    alpha: f64,
) -> Result<f64, InvalidRenyiOrder> {
    validate_order(alpha)?;
    Ok(evaluate(distribution, alpha))
}

fn validate_order(alpha: f64) -> Result<(), InvalidRenyiOrder> {
    if alpha.is_nan() || alpha < 0.0 {
        Err(InvalidRenyiOrder)
    } else {
        Ok(())
    }
}

// Natural logarithm of an exact count ratio, with 0 < numerator <= denominator.
// Subtract before conversion to retain a tiny complement near probability one.
fn log_ratio(numerator: usize, denominator: usize) -> f64 {
    if numerator > denominator / 2 {
        (-((denominator - numerator) as f64 / denominator as f64)).ln_1p()
    } else {
        (numerator as f64 / denominator as f64).ln()
    }
}

// Kahan summation in deterministic byte order, without an intermediate vector.
fn compensated_sum(terms: impl Iterator<Item = f64>) -> f64 {
    let mut sum = 0.0;
    let mut correction = 0.0;
    for term in terms {
        let adjusted = term - correction;
        let next = sum + adjusted;
        correction = (next - sum) - adjusted;
        sum = next;
    }
    sum
}

fn evaluate(d: &Distribution, alpha: f64) -> f64 {
    if alpha == 0.0 {
        return hartley_entropy_distribution(d);
    }
    if alpha == 1.0 {
        return shannon_distribution(d);
    }
    let counts = d.histogram().counts();
    // The fixed table is always nonempty; choose one maximum deterministically.
    let (maximum_index, &maximum) = counts.iter().enumerate().max_by_key(|(_, c)| *c).unwrap();
    let total = d.sample_size();
    if maximum == total {
        return 0.0; // Includes both empty and singleton support.
    }
    let log_maximum = log_ratio(maximum, total);
    let minimum_entropy = -log_maximum / LN_2;
    if alpha == f64::INFINITY {
        return minimum_entropy;
    }

    let t = alpha - 1.0;
    if t.abs() <= 0.25 {
        // sum(p^alpha) = 1 + sum(p * expm1(t * ln(p))).
        // No subtraction of nearly equal numbers when alpha is near one.
        let delta = compensated_sum(
            counts
                .iter()
                .copied()
                .filter(|&c| c > 0)
                .map(|c| (c as f64 / total as f64) * (t * log_ratio(c, total)).exp_m1()),
        );
        return -delta.ln_1p() / t / LN_2;
    }

    // Factor out p_max^alpha; the omitted maximum contributes exactly one.
    let tail = compensated_sum(
        counts
            .iter()
            .enumerate()
            .filter(|&(i, &c)| c > 0 && i != maximum_index)
            .map(|(_, &c)| (alpha * log_ratio(c, maximum)).exp()),
    );
    let log_scaled_sum = tail.ln_1p();
    if alpha > 1.0 {
        // Avoid alpha * log(p_max), which could overflow for a finite order.
        minimum_entropy + (minimum_entropy - log_scaled_sum / LN_2) / t
    } else {
        (alpha * log_maximum + log_scaled_sum) / (1.0 - alpha) / LN_2
    }
}
