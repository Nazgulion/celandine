use super::ByteHistogram;

/// A reusable empirical byte distribution backed by exact counts.
///
/// "Empirical" means derived from the observed sample. Relative frequencies
/// turn counts into comparable fractions: `AAAB` gives probabilities 3/4 and
/// 1/4. Shannon can use these fractions and Hartley can use the exact support
/// from the same histogram, without recounting the input.
///
/// For a nonempty sample, `p[b] = count[b] / sample_size`. Probabilities are
/// computed lazily as `f64`, without allocation or a second stored table.
/// Only validated integer counts can enter; arbitrary probability vectors
/// are not accepted, normalized, or smoothed.
///
/// An empty sample is represented as an explicit empty empirical state:
/// all probability queries return zero. It is not a unit-mass probability law.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Distribution {
    histogram: ByteHistogram,
}

impl Distribution {
    /// Counts a byte slice once, with fixed storage and no heap allocation.
    pub fn from_bytes(data: &[u8]) -> Self {
        Self::from_counts(ByteHistogram::from_bytes(data))
    }

    /// Takes ownership of a validated histogram without recounting the input.
    pub fn from_counts(histogram: ByteHistogram) -> Self {
        Self { histogram }
    }

    /// Borrows the underlying exact counts and total.
    pub fn histogram(&self) -> &ByteHistogram {
        &self.histogram
    }

    /// Returns the sample size, distinct from observed support size.
    pub fn sample_size(&self) -> usize {
        self.histogram.total()
    }

    /// Returns whether this is the empty empirical state.
    pub fn is_empty(&self) -> bool {
        self.histogram.is_empty()
    }

    /// Returns the number of positive counts in `O(256)` time.
    pub fn support_size(&self) -> usize {
        self.histogram.support_size()
    }

    /// Returns `count / sample_size`, or zero for an empty sample.
    pub fn probability(&self, symbol: u8) -> f64 {
        self.histogram.count(symbol) as f64 / self.sample_size().max(1) as f64
    }

    /// Iterates over all 256 probabilities in ascending byte order.
    ///
    /// Includes zero probabilities. Empty samples yield 256 zeros. The iterator
    /// borrows this distribution and performs no allocation.
    pub fn probabilities(&self) -> impl ExactSizeIterator<Item = f64> + '_ {
        let total = self.sample_size().max(1) as f64;
        self.histogram
            .counts()
            .iter()
            .map(move |&count| count as f64 / total)
    }
}
