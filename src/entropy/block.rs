use crate::distribution::{NgramDistribution, ngram_probabilities};
use crate::transforms::InvalidNgramLength;

#[doc = include_str!("../../docs/entropy/block.md")]
///
/// # Errors
/// Returns [`InvalidNgramLength`] for `n == 0`, including on empty input.
///
/// # Example
/// ```
/// use celandine::entropy::block_entropy;
/// assert_eq!(block_entropy(b"ABABA", 2)?, 1.0);
/// assert_eq!(block_entropy(b"ABABA", 5)?, 0.0);
/// # Ok::<(), celandine::transforms::InvalidNgramLength>(())
/// ```
pub fn block_entropy(data: &[u8], n: usize) -> Result<f64, InvalidNgramLength> {
    if n == 0 {
        return Err(InvalidNgramLength);
    }
    if n >= data.len() {
        return Ok(0.0);
    }
    if n == 1 {
        return Ok(super::shannon(data));
    }
    Ok(block_entropy_distribution(&ngram_probabilities(data, n)?))
}

/// Shannon entropy of an existing overlapping n-gram distribution, in bits per block.
///
/// Computes `-sum(p * log2(p))` over observed blocks, where `p` is the block's
/// count divided by the number of overlapping observations. Reusing the table
/// avoids recounting: `O(k)` time for `k` distinct blocks, no new heap allocation,
/// and `O(1)` auxiliary storage. Empty and single-block-support laws return
/// positive zero. See [`block_entropy`] for conventions, history and limitations.
///
/// ```
/// use celandine::{distribution::ngram_probabilities, entropy::block_entropy_distribution};
/// let distribution = ngram_probabilities(b"ABABA", 2)?;
/// assert_eq!(block_entropy_distribution(&distribution), 1.0);
/// # Ok::<(), celandine::transforms::InvalidNgramLength>(())
/// ```
pub fn block_entropy_distribution(distribution: &NgramDistribution<'_>) -> f64 {
    if distribution.support_size() <= 1 {
        return 0.0;
    }
    let total = distribution.sample_size();
    let mut sum = 0.0;
    let mut correction = 0.0;
    for (_, count) in distribution.counts().counts() {
        let p = count as f64 / total as f64;
        // Preserve the majority contribution even when p rounds to one.
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
