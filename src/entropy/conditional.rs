use crate::distribution::{NgramDistribution, ngram_probabilities};

#[doc = include_str!("../../docs/entropy/conditional_order.md")]
///
/// # Example
/// ```
/// use celandine::entropy::conditional_entropy_order;
/// assert_eq!(conditional_entropy_order(b"ABABAB", 0), 1.0);
/// assert_eq!(conditional_entropy_order(b"ABABAB", 1), 0.0);
/// assert_eq!(conditional_entropy_order(b"AAB", 1), 1.0);
/// ```
pub fn conditional_entropy_order(data: &[u8], order: usize) -> f64 {
    if order == 0 {
        return super::shannon(data);
    }
    // Zero or one complete context/next-byte observation has zero entropy.
    // Check before adding one: every usize order, including MAX, is accepted.
    if order >= data.len().saturating_sub(1) {
        return 0.0;
    }
    let distribution = ngram_probabilities(data, order + 1)
        .expect("order below input length gives a positive, representable block length");
    conditional_entropy_order_distribution(&distribution)
}

/// Conditional entropy of the last byte given the preceding bytes in each n-gram.
///
/// The context order is `distribution.ngram_len() - 1`: pass pairs for order one,
/// triples for order two, or single-byte blocks for order zero. Counts and their
/// prefix marginals use exactly the same observations. Returns bits per predicted
/// byte, with positive zero for empty or deterministic empirical transitions.
/// See [`conditional_entropy_order`] for the formula, history and limitations.
///
/// With `s` distinct n-grams of length `n`, this takes `O(s*n)` worst-case time
/// and a fixed 256-count scratch array, without any new heap allocation. Adjacent
/// lexicographic keys with the same prefix form a context group. No table is
/// cloned or recounted. Order-zero reuse can differ from `shannon` in rounding.
///
/// ```
/// use celandine::{distribution::ngram_probabilities, entropy::conditional_entropy_order_distribution};
/// let pairs = ngram_probabilities(b"AAB", 2)?;
/// assert_eq!(conditional_entropy_order_distribution(&pairs), 1.0);
/// # Ok::<(), celandine::transforms::InvalidNgramLength>(())
/// ```
pub fn conditional_entropy_order_distribution(distribution: &NgramDistribution<'_>) -> f64 {
    if distribution.support_size() <= 1 {
        return 0.0;
    }
    let order = distribution.ngram_len() - 1;
    let total = distribution.sample_size() as f64;
    let mut entries = distribution.counts().counts().peekable();
    let mut counts = [0usize; 256];
    let mut sum = 0.0;
    let mut correction = 0.0;
    while let Some((block, count)) = entries.next() {
        let context = &block[..order];
        counts[0] = count;
        let mut used = 1;
        let mut context_total = count;
        while let Some(&(next, count)) = entries.peek() {
            if &next[..order] != context {
                break;
            }
            // A context has at most 256 distinct next bytes. All counts are
            // positive and their sum is bounded by the distribution sample size.
            counts[used] = count;
            used += 1;
            context_total += count;
            entries.next();
        }
        if used == 1 {
            continue;
        }
        for &count in &counts[..used] {
            let log = if count > context_total / 2 {
                (-((context_total - count) as f64 / context_total as f64)).ln_1p()
                    / std::f64::consts::LN_2
            } else {
                (count as f64 / context_total as f64).log2()
            };
            let term = -(count as f64 / total) * log - correction;
            let next = sum + term;
            correction = (next - sum) - term;
            sum = next;
        }
    }
    sum
}
