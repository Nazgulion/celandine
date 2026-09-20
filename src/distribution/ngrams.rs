use crate::transforms::{InvalidNgramLength, ngrams};
use std::collections::BTreeMap;

/// Exact frequencies of observed overlapping byte blocks of one positive length.
///
/// Construct with [`ngram_counts`]. For `ABABA` at length two, `AB` and `BA`
/// each have count two, and the total is four block occurrences. Keeping exact
/// counts allows probabilities to reuse the same tally.
///
/// Keys borrow the input and compare by byte contents. Only positive counts are
/// stored, in lexicographic byte order. Fields are private; their integer sum
/// always equals `total()`. The table owns `O(k)` tree storage for `k` distinct
/// blocks but does not copy their bytes. It cannot outlive the input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NgramCounts<'a> {
    n: usize,
    total: usize,
    counts: BTreeMap<&'a [u8], usize>,
}

/// Counts every overlapping `n`-byte block, borrowing keys from `data`.
///
/// With `m` block occurrences and `k` distinct blocks, construction takes
/// `O(m * n * log(k + 1))` worst-case time and `O(k)` additional storage.
/// Each count and their sum are bounded by the input length and cannot overflow.
/// Empty input or `n > data.len()` yields an empty table without allocation.
///
/// # Errors
/// Returns [`InvalidNgramLength`] for `n == 0` before inspecting the input.
///
/// # Example
/// ```
/// use celandine::distribution::ngram_counts;
/// let counts = ngram_counts(b"ABABA", 2)?;
/// assert_eq!(counts.total(), 4);
/// assert_eq!(counts.count(b"AB"), 2);
/// assert_eq!(counts.count(b"AA"), 0);
/// # Ok::<(), celandine::transforms::InvalidNgramLength>(())
/// ```
pub fn ngram_counts(data: &[u8], n: usize) -> Result<NgramCounts<'_>, InvalidNgramLength> {
    let blocks = ngrams(data, n)?;
    let total = blocks.len();
    let mut counts = BTreeMap::new();
    for block in blocks {
        *counts.entry(block).or_insert(0) += 1;
    }
    Ok(NgramCounts { n, total, counts })
}

impl<'a> NgramCounts<'a> {
    /// Returns the requested positive block length in bytes, even for no blocks.
    pub fn ngram_len(&self) -> usize {
        self.n
    }

    /// Returns the exact number of block occurrences, including repeated blocks.
    pub fn total(&self) -> usize {
        self.total
    }

    /// Returns whether there are no complete blocks.
    pub fn is_empty(&self) -> bool {
        self.total == 0
    }

    /// Returns the number of distinct observed blocks in `O(1)` time.
    pub fn support_size(&self) -> usize {
        self.counts.len()
    }

    /// Returns a block's count, or zero for absent blocks and wrong lengths.
    ///
    /// Queries compare byte contents in `O(n * log(k + 1))` worst-case time,
    /// where `n` is the block length and `k` the support size, without allocation.
    pub fn count(&self, block: &[u8]) -> usize {
        if block.len() != self.n {
            return 0;
        }
        self.counts.get(block).copied().unwrap_or(0)
    }

    /// Iterates over `(block, count)` pairs in lexicographic byte order.
    ///
    /// Visits only observed blocks, in `O(k)` total time without allocation.
    /// Returned slices borrow the original input, not copied keys.
    pub fn counts(&self) -> impl ExactSizeIterator<Item = (&'a [u8], usize)> + '_ {
        self.counts.iter().map(|(&block, &count)| (block, count))
    }
}

/// An empirical distribution over observed overlapping byte blocks.
///
/// Retains [`NgramCounts`] and derives `p(block) = count(block) / sample_size`
/// lazily as `f64`. Here sample size counts block occurrences, not input bytes.
/// For `ABABA` at length two, both `AB` and `BA` have probability `2/4 = 0.5`.
/// This describes local frequencies without assuming independent observations.
///
/// An empty table is an explicit empty empirical state: queries return zero
/// and iteration yields no entries. It is not a unit-mass probability law.
/// Keys borrow the original input; this distribution cannot outlive it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NgramDistribution<'a> {
    counts: NgramCounts<'a>,
}

/// Counts overlapping byte blocks once and returns their empirical distribution.
///
/// Uses [`ngram_counts`] and [`NgramDistribution::from_counts`], with the same
/// construction costs and errors. Probabilities are dimensionless fractions;
/// no smoothing, boundary markers, or unseen blocks are added.
///
/// # Errors
/// Returns [`InvalidNgramLength`] for `n == 0`, even for empty input.
///
/// # Example
/// ```
/// use celandine::distribution::ngram_probabilities;
/// let d = ngram_probabilities(b"ABABA", 2)?;
/// assert_eq!(d.sample_size(), 4);
/// assert_eq!(d.probability(b"AB"), 0.5);
/// assert_eq!(d.probability(b"AA"), 0.0);
/// # Ok::<(), celandine::transforms::InvalidNgramLength>(())
/// ```
pub fn ngram_probabilities(
    data: &[u8],
    n: usize,
) -> Result<NgramDistribution<'_>, InvalidNgramLength> {
    Ok(NgramDistribution::from_counts(ngram_counts(data, n)?))
}

impl<'a> NgramDistribution<'a> {
    /// Takes ownership of exact counts in `O(1)` time without recounting or allocating.
    ///
    /// ```
    /// use celandine::distribution::{ngram_counts, NgramDistribution};
    /// let d = NgramDistribution::from_counts(ngram_counts(b"AAAA", 2)?);
    /// assert_eq!(d.sample_size(), 3);
    /// assert_eq!(d.probability(b"AA"), 1.0);
    /// # Ok::<(), celandine::transforms::InvalidNgramLength>(())
    /// ```
    pub fn from_counts(counts: NgramCounts<'a>) -> Self {
        Self { counts }
    }

    /// Borrows the underlying exact counts and block total without allocation.
    pub fn counts(&self) -> &NgramCounts<'a> {
        &self.counts
    }

    /// Returns the positive block length in bytes, even for an empty state.
    pub fn ngram_len(&self) -> usize {
        self.counts.ngram_len()
    }

    /// Returns the number of observed block occurrences, not input bytes.
    pub fn sample_size(&self) -> usize {
        self.counts.total()
    }

    /// Returns whether this is an empty empirical state.
    pub fn is_empty(&self) -> bool {
        self.counts.is_empty()
    }

    /// Returns the number of distinct observed blocks in `O(1)` time.
    pub fn support_size(&self) -> usize {
        self.counts.support_size()
    }

    /// Returns `count(block) / sample_size`, or zero when no matching block exists.
    ///
    /// Empty samples and wrong-length queries return positive zero. Lookup has
    /// the same costs as [`NgramCounts::count`]; conversion to `f64` can round.
    pub fn probability(&self, block: &[u8]) -> f64 {
        self.counts.count(block) as f64 / self.sample_size().max(1) as f64
    }

    /// Iterates over observed `(block, probability)` pairs in lexicographic order.
    ///
    /// Empty samples yield no entries. Visits `k` distinct blocks in `O(k)`
    /// time without allocating, deriving each probability from its exact count.
    pub fn probabilities(&self) -> impl ExactSizeIterator<Item = (&'a [u8], f64)> + '_ {
        let total = self.sample_size().max(1) as f64;
        self.counts
            .counts()
            .map(move |(block, count)| (block, count as f64 / total))
    }
}
