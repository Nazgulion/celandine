use crate::distribution::{JointDistribution, LengthMismatch, joint_probabilities};
use std::f64::consts::LN_2;

#[doc = include_str!("../../docs/entropy/conditional.md")]
///
/// # Errors
/// Returns [`LengthMismatch`] before counting if lengths differ, including one empty input.
///
/// ```
/// use celandine::entropy::conditional_entropy;
/// assert_eq!(conditional_entropy(b"AABB", b"XYXY")?, 1.0);
/// assert_eq!(conditional_entropy(b"ABCD", b"XXYY")?, 1.0);
/// assert_eq!(conditional_entropy(b"XXYY", b"ABCD")?, 0.0);
/// # Ok::<(), celandine::distribution::LengthMismatch>(())
/// ```
pub fn conditional_entropy(x: &[u8], y: &[u8]) -> Result<f64, LengthMismatch> {
    Ok(conditional_entropy_distribution(&joint_probabilities(
        x, y,
    )?))
}

/// Uncertainty of the first coordinate given the second in a reusable joint law.
///
/// Evaluates `-sum (c(a,b)/N)*log2(c(a,b)/c_y(b))`, in bits per first-coordinate
/// observation. Here c counts aligned pairs, N is the total number of pairs and
/// c_y is the second marginal. For `ABCD` paired with `XXYY`, each observed Y
/// leaves two equally likely X values, giving one bit. Reversing the inputs gives
/// zero. Empty laws and deterministic X-given-Y laws return positive zero.
///
/// Uses O(s+256) time for s observed pairs and fixed 256-count storage, without
/// heap allocation. The direction is determined by construction of the joint
/// law; swapping coordinates requires a law built in the reverse order. See
/// [`conditional_entropy`] for history, numerical behavior and source limits.
///
/// ```
/// use celandine::{distribution::joint_probabilities, entropy::conditional_entropy_distribution};
/// let d = joint_probabilities(b"ABCD", b"XXYY")?;
/// assert_eq!(conditional_entropy_distribution(&d), 1.0);
/// # Ok::<(), celandine::distribution::LengthMismatch>(())
/// ```
pub fn conditional_entropy_distribution(distribution: &JointDistribution) -> f64 {
    if distribution.support_size() <= 1 {
        return 0.0;
    }
    let counts = distribution.counts();
    let marginal = counts.marginal_y();
    let total = counts.total();
    let mut sum = 0.0;
    let mut correction = 0.0;
    for ((_, b), count) in counts.counts() {
        let given_total = marginal.count(b);
        if count == given_total {
            // X is certain for this Y, even when other Y groups are uncertain.
            continue;
        }
        let term = conditional_term(count, given_total, total) - correction;
        let next = sum + term;
        correction = (next - sum) - term;
        sum = next;
    }
    sum
}

// Private positive counts obey count < given_total <= total. Keep the integer
// complement before conversion so a nearly certain conditional event survives.
fn conditional_term(count: usize, given_total: usize, total: usize) -> f64 {
    let log = if count > given_total / 2 {
        (-((given_total - count) as f64 / given_total as f64)).ln_1p() / LN_2
    } else {
        (count as f64 / given_total as f64).log2()
    };
    -(count as f64 / total as f64) * log
}

#[cfg(test)]
mod tests {
    use super::conditional_term;
    #[test]
    fn near_certain_counts_preserve_the_majority_contribution() {
        // Public laws come from slices; exercise unmaterializable valid count
        // scales in the private arithmetic kernel without widening the API.
        let n = usize::MAX;
        let actual = conditional_term(n - 1, n, n);
        // -((n-1)/n)*log2(1-1/n) = 1/(n ln 2) * (1-O(1/n)).
        let leading = 1.0 / n as f64 / std::f64::consts::LN_2;
        assert!(actual > 0.0);
        assert!((actual - leading).abs() <= leading * 1e-9);
        assert_eq!(conditional_term(1, 2, n), 1.0 / n as f64);
    }
}
