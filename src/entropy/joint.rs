use crate::distribution::{JointDistribution, LengthMismatch, joint_probabilities};

#[doc = include_str!("../../docs/entropy/joint.md")]
///
/// # Errors
/// Returns [`LengthMismatch`] for unequal lengths, even if one input is empty.
///
/// ```
/// use celandine::entropy::joint_entropy;
/// assert_eq!(joint_entropy(b"AABB", b"XYXY")?, 2.0);
/// assert_eq!(joint_entropy(b"AABB", b"XXYY")?, 1.0);
/// assert_eq!(joint_entropy(b"", b"")?, 0.0);
/// # Ok::<(), celandine::distribution::LengthMismatch>(())
/// ```
pub fn joint_entropy(x: &[u8], y: &[u8]) -> Result<f64, LengthMismatch> {
    Ok(joint_entropy_distribution(&joint_probabilities(x, y)?))
}

/// Shannon entropy of an existing joint byte law, in bits per paired observation.
///
/// Computes `-sum p(a,b)*log2(p(a,b))` from exact counts. For four equally
/// frequent pairs the result is 2 bits/pair. Reuses the table in O(s) time,
/// with O(1) auxiliary space and no new heap allocation. Empty and constant-pair
/// laws return positive zero. See [`joint_entropy`] for bounds, history and limits.
///
/// ```
/// use celandine::{distribution::joint_probabilities, entropy::joint_entropy_distribution};
/// let d = joint_probabilities(b"AABB", b"XYXY")?;
/// assert_eq!(joint_entropy_distribution(&d), 2.0);
/// # Ok::<(), celandine::distribution::LengthMismatch>(())
/// ```
pub fn joint_entropy_distribution(distribution: &JointDistribution) -> f64 {
    if distribution.support_size() <= 1 {
        return 0.0;
    }
    let total = distribution.sample_size();
    let mut sum = 0.0;
    let mut correction = 0.0;
    for (_, count) in distribution.counts().counts() {
        let p = count as f64 / total as f64;
        // Use the integer complement to retain a near-certain pair's contribution.
        let log = if count > total / 2 {
            (-((total - count) as f64 / total as f64)).ln_1p() / std::f64::consts::LN_2
        } else {
            p.log2()
        };
        let term = -p * log - correction;
        let next = sum + term;
        correction = (next - sum) - term;
        sum = next;
    }
    sum
}
