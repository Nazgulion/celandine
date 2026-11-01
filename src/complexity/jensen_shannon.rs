use crate::distribution::{ByteHistogram, Distribution};
use crate::divergence::jensen_shannon_divergence_distribution;
use std::{f64::consts::LN_2, fmt};

/// An invalid domain for normalized Jensen–Shannon statistical complexity.
///
/// Validation checks alphabet range first, then empty input, then whether the
/// observed support fits. For example, three observed byte values with alphabet
/// size two return `AlphabetTooSmall`. No alphabet is inferred or enlarged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatisticalComplexityError {
    /// The declared number of possible byte states is outside `2..=256`.
    InvalidAlphabetSize,
    /// There are no observations, hence no empirical probability law.
    EmptyDistribution,
    /// More distinct bytes were observed than the declared alphabet permits.
    AlphabetTooSmall,
}
impl fmt::Display for StatisticalComplexityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidAlphabetSize => "alphabet size must be between 2 and 256",
            Self::EmptyDistribution => "statistical complexity requires a nonempty distribution",
            Self::AlphabetTooSmall => "observed support exceeds the declared alphabet size",
        })
    }
}
impl std::error::Error for StatisticalComplexityError {}

#[doc = include_str!("../../docs/complexity/jensen_shannon.md")]
pub fn normalized_jensen_shannon_complexity(
    data: &[u8],
    alphabet_size: usize,
) -> Result<f64, StatisticalComplexityError> {
    validate_alphabet(alphabet_size)?;
    normalized_jensen_shannon_complexity_distribution(
        &Distribution::from_bytes(data),
        alphabet_size,
    )
}

/// Normalized Jensen–Shannon statistical complexity from reusable byte counts.
///
/// Computes `(H(P)/log2(K)) * (J(P,U_K)/J_max(K))`, with explicit alphabet size
/// `K` including unobserved states. The result is dimensionless. Labels need not
/// lie in `0..K`; the alphabet must contain the observed support. Uses O(256)
/// time and fixed auxiliary storage without heap allocation. See
/// [`normalized_jensen_shannon_complexity`] for history, examples and limits.
///
/// # Errors
/// Checks `2 <= K <= 256`, nonempty input, then support size at most K, in that
/// order. Returns the corresponding [`StatisticalComplexityError`].
///
/// ```
/// use celandine::{distribution::Distribution, complexity::normalized_jensen_shannon_complexity_distribution};
/// let p = Distribution::from_bytes(b"AABB");
/// assert_eq!(normalized_jensen_shannon_complexity_distribution(&p, 2), Ok(0.0));
/// assert!(normalized_jensen_shannon_complexity_distribution(&p, 4).unwrap() > 0.0);
/// ```
pub fn normalized_jensen_shannon_complexity_distribution(
    distribution: &Distribution,
    alphabet_size: usize,
) -> Result<f64, StatisticalComplexityError> {
    validate_alphabet(alphabet_size)?;
    if distribution.is_empty() {
        return Err(StatisticalComplexityError::EmptyDistribution);
    }
    let support = distribution.support_size();
    if support > alphabet_size {
        return Err(StatisticalComplexityError::AlphabetTooSmall);
    }
    if support == 1 {
        return Ok(0.0);
    }
    // Every observed label belongs to U_K. Assign any extra states to absent
    // byte labels: their identities cannot affect a permutation-invariant sum.
    let mut uniform = distribution
        .histogram()
        .counts()
        .map(|c| usize::from(c > 0));
    let mut missing = alphabet_size - support;
    for count in &mut uniform {
        if *count == 0 && missing > 0 {
            *count = 1;
            missing -= 1;
        }
    }
    let uniform = Distribution::from_counts(
        ByteHistogram::try_from_counts(uniform).expect("uniform total is at most 256"),
    );
    let divergence = jensen_shannon_divergence_distribution(distribution, &uniform)
        .expect("both distributions are nonempty");
    let k = alphabet_size as f64;
    // J(delta,U_K), algebraically rearranged to avoid subtracting large logs.
    let maximum = 1.0 - 0.5 * (k.log2() / k + (1.0 + 1.0 / k) * (1.0 / k).ln_1p() / LN_2);
    Ok((stable_entropy(distribution) / k.log2()) * (divergence / maximum))
}

fn validate_alphabet(size: usize) -> Result<(), StatisticalComplexityError> {
    if !(2..=256).contains(&size) {
        Err(StatisticalComplexityError::InvalidAlphabetSize)
    } else {
        Ok(())
    }
}

// Preserve the majority contribution when count/total rounds to one. Keep this
// evaluation local: the standalone Shannon API retains its existing behavior.
fn stable_entropy(distribution: &Distribution) -> f64 {
    let total = distribution.sample_size();
    let mut sum = 0.0;
    let mut correction = 0.0;
    for &count in distribution.histogram().counts() {
        if count == 0 {
            continue;
        }
        let p = count as f64 / total as f64;
        let log = if count > total / 2 {
            (-((total - count) as f64 / total as f64)).ln_1p() / LN_2
        } else {
            p.log2()
        };
        let adjusted = -p * log - correction;
        let next = sum + adjusted;
        correction = (next - sum) - adjusted;
        sum = next;
    }
    sum
}
