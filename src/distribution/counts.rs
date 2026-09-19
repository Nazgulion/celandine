use std::fmt;

/// Exact frequencies over the 256 byte values, with no heap allocation.
///
/// Counts summarize repeated observations: `AAAB` has counts 3 for `A` and 1
/// for `B`. Keeping these integers lets several metrics reuse one input pass.
/// Order is discarded, so `ABAA` produces the same histogram.
///
/// The private total always equals the sum of the private counts. Observed
/// support is the number of strictly positive counts, not the alphabet size.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ByteHistogram {
    counts: [usize; 256],
    total: usize,
}

/// The sum of an externally supplied count table exceeds `usize::MAX`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CountOverflow;

impl fmt::Display for CountOverflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("byte count total exceeds usize::MAX")
    }
}

impl std::error::Error for CountOverflow {}

impl ByteHistogram {
    /// Counts a byte slice in one pass, using `O(256)` fixed storage.
    ///
    /// Each count is bounded by `data.len()`, so counting cannot overflow.
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut counts = [0; 256];
        for &byte in data {
            counts[usize::from(byte)] += 1;
        }
        Self {
            counts,
            total: data.len(),
        }
    }

    /// Reuses a count table after checking its total in `O(256)` time.
    ///
    /// An all-zero table is valid and represents an empty sample.
    ///
    /// # Errors
    /// Returns [`CountOverflow`] if the sum cannot be represented by `usize`.
    pub fn try_from_counts(counts: [usize; 256]) -> Result<Self, CountOverflow> {
        let total = counts
            .iter()
            .try_fold(0usize, |total, &count| total.checked_add(count))
            .ok_or(CountOverflow)?;
        Ok(Self { counts, total })
    }

    /// Returns the exact number of observations.
    pub fn total(&self) -> usize {
        self.total
    }

    /// Returns whether there are no observations.
    pub fn is_empty(&self) -> bool {
        self.total == 0
    }

    /// Returns the exact count for a byte, including zero for absent bytes.
    pub fn count(&self, symbol: u8) -> usize {
        self.counts[usize::from(symbol)]
    }

    /// Borrows all 256 counts in ascending byte order.
    pub fn counts(&self) -> &[usize; 256] {
        &self.counts
    }

    /// Counts observed symbols in `O(256)` time; returns zero for empty input.
    pub fn support_size(&self) -> usize {
        self.counts.iter().filter(|&&count| count > 0).count()
    }
}
