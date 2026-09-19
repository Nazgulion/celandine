use crate::distribution::Distribution;

#[doc = include_str!("../../docs/entropy/hartley.md")]
pub fn hartley_entropy(data: &[u8]) -> f64 {
    hartley_entropy_distribution(&Distribution::from_bytes(data))
}

/// Computes Hartley entropy in bits from the observed support of a distribution.
///
/// Returns `log2(k)` for `k > 0` positive integer counts, and positive zero for
/// an empty empirical state by convention. Frequencies do not otherwise affect
/// the result. Support is determined from exact counts, not rounded probabilities.
///
/// Takes `O(256)` time and `O(1)` additional space, with no heap allocation or
/// recounting. See [`hartley_entropy`] for interpretation and references.
///
/// ```
/// use celandine::distribution::Distribution;
/// use celandine::entropy::{hartley_entropy_distribution, shannon_distribution};
///
/// let d = Distribution::from_bytes(b"AAAB");
/// assert_eq!(hartley_entropy_distribution(&d), 1.0);
/// assert!(shannon_distribution(&d) < hartley_entropy_distribution(&d));
/// ```
pub fn hartley_entropy_distribution(distribution: &Distribution) -> f64 {
    let support = distribution.support_size();
    if support <= 1 {
        0.0
    } else {
        (support as f64).log2()
    }
}
