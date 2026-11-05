use crate::distribution::{JointDistribution, LengthMismatch, joint_probabilities};
use std::f64::consts::LN_2;

#[doc = include_str!("../../docs/information/mutual.md")]
///
/// # Errors
/// Returns [`LengthMismatch`] for unequal lengths, including one empty input.
///
/// ```
/// use celandine::information::mutual_information;
/// assert_eq!(mutual_information(b"AABB", b"XYXY")?, 0.0);
/// assert!((mutual_information(b"AABB", b"XXYY")? - 1.0).abs() < 1e-12);
/// # Ok::<(), celandine::distribution::LengthMismatch>(())
/// ```
pub fn mutual_information(x: &[u8], y: &[u8]) -> Result<f64, LengthMismatch> {
    Ok(mutual_information_distribution(&joint_probabilities(x, y)?))
}

/// Shared information in an existing joint byte law, in bits per pair.
///
/// Computes `sum p(a,b)*log2(p(a,b)/(p_x(a)*p_y(b)))`, using exact integer
/// marginals and a stable nonnegative divergence decomposition. Four equally
/// frequent pairs AX, AY, BX, BY give zero; equally frequent AX and BY give one
/// bit. Empty laws and laws with a constant coordinate return positive zero.
///
/// Takes `O(s+256)` time for s observed pairs, two fixed 256-count histograms,
/// and no new heap allocation. See [`mutual_information`] for numerical details,
/// bounds, interpretation and finite-sample limitations.
///
/// ```
/// use celandine::{distribution::joint_probabilities, information::mutual_information_distribution};
/// let d = joint_probabilities(b"AABB", b"XXYY")?;
/// assert!((mutual_information_distribution(&d) - 1.0).abs() < 1e-12);
/// # Ok::<(), celandine::distribution::LengthMismatch>(())
/// ```
pub fn mutual_information_distribution(distribution: &JointDistribution) -> f64 {
    if distribution.support_size() <= 1 {
        return 0.0;
    }
    let counts = distribution.counts();
    let mx = counts.marginal_x();
    let my = counts.marginal_y();
    let n = counts.total() as u128;
    let n_squared = n * n;
    let mut observed_product = 0u128;
    let mut sum = 0.0;
    let mut correction = 0.0;
    for ((a, b), c) in counts.counts() {
        // usize is at most 64 bits on supported targets; products fit u128.
        let joint = c as u128 * n;
        let product = mx.count(a) as u128 * my.count(b) as u128;
        observed_product += product;
        let term = divergence_term(joint, product) - correction;
        let next = sum + term;
        correction = (next - sum) - term;
        sum = next;
    }
    // Absent joint cells contribute q. Their total product mass is exact, and
    // summing it avoids visiting all 65,536 possible pairs or querying the tree.
    let missing = (n_squared - observed_product) as f64;
    let result = (sum + (missing - correction)) / n_squared as f64 / LN_2;
    if result == 0.0 { 0.0 } else { result }
}

// a*ln(a/b)-a+b, for strictly positive a,b. It is nonnegative and zero at a=b.
fn divergence_term(a: u128, b: u128) -> f64 {
    if a == b {
        return 0.0;
    }
    let delta = if a > b {
        (a - b) as f64 / b as f64
    } else {
        -((b - a) as f64 / b as f64)
    };
    if delta.abs() <= 0.25 {
        // (1+d)*ln(1+d)-d = sum_{k>=2} (-d)^k/[k*(k-1)].
        // The exact integer difference retains departures below f64 resolution
        // of the large products. Avoid cancellation in the quadratic result.
        let mut term = delta * delta / 2.0;
        let mut sum = term;
        for k in 3..=32 {
            term *= -delta * (k - 2) as f64 / k as f64;
            sum += term;
            if term.abs() <= f64::EPSILON * sum {
                break;
            }
        }
        b as f64 * sum
    } else {
        let difference = if a > b {
            (a - b) as f64
        } else {
            -((b - a) as f64)
        };
        a as f64 * (a as f64 / b as f64).ln() - difference
    }
}

#[cfg(test)]
mod tests {
    use super::divergence_term;
    #[test]
    fn integer_product_gaps_survive_float_rounding() {
        let base = 1u128 << 120;
        for (a, b) in [(base, base - 1), (base - 1, base)] {
            assert_eq!(a as f64, b as f64);
            // With |a-b|=1, the leading term is 1/(2b); the relative
            // remainder is below 1/b, far below f64 rounding at this scale.
            let expected = 0.5 / b as f64;
            let actual = divergence_term(a, b);
            assert!(actual > 0.0 && (actual - expected).abs() <= expected * 1e-15);
        }
    }
}
