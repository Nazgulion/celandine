use super::{ByteHistogram, Distribution};
use std::{collections::BTreeMap, fmt};

/// Paired byte observations have different lengths.
///
/// No truncation, padding or implicit alignment is performed. For example,
/// pairing `AB` with `X` returns lengths 2 and 1 before counting any observations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LengthMismatch {
    /// Number of observations in the first input.
    pub x_len: usize,
    /// Number of observations in the second input.
    pub y_len: usize,
}

impl fmt::Display for LengthMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "paired inputs must have equal lengths ({} != {})",
            self.x_len, self.y_len
        )
    }
}
impl std::error::Error for LengthMismatch {}

#[doc = include_str!("../../docs/joint-distributions.md")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointCounts {
    total: usize,
    counts: BTreeMap<(u8, u8), usize>,
}

/// Counts position-aligned `(x[i], y[i])` observations, owning the pair keys.
///
/// Takes `O(N log(s+1))` time and `O(s)` heap storage for N pairs and s distinct
/// pairs (at most 65,536). Empty inputs allocate nothing. See [`JointCounts`]
/// for the definition, marginal operations, examples and limits.
///
/// # Errors
/// Returns [`LengthMismatch`] before counting if lengths differ, even if one is empty.
///
/// ```
/// use celandine::distribution::joint_counts;
/// let c = joint_counts(b"AABB", b"XYXY")?;
/// assert_eq!(c.total(), 4);
/// assert_eq!(c.count(b'A', b'X'), 1);
/// # Ok::<(), celandine::distribution::LengthMismatch>(())
/// ```
pub fn joint_counts(x: &[u8], y: &[u8]) -> Result<JointCounts, LengthMismatch> {
    if x.len() != y.len() {
        return Err(LengthMismatch {
            x_len: x.len(),
            y_len: y.len(),
        });
    }
    let mut counts = BTreeMap::new();
    for (&a, &b) in x.iter().zip(y) {
        *counts.entry((a, b)).or_insert(0) += 1;
    }
    Ok(JointCounts {
        total: x.len(),
        counts,
    })
}

impl JointCounts {
    /// Number of paired observations, not the sum of the two input lengths.
    pub fn total(&self) -> usize {
        self.total
    }
    /// Whether no observations were supplied.
    pub fn is_empty(&self) -> bool {
        self.total == 0
    }
    /// Number of observed pairs, in O(1) time; zero for an empty sample.
    pub fn support_size(&self) -> usize {
        self.counts.len()
    }
    /// Exact pair count, or zero if absent, in `O(log(s+1))` time without allocation.
    pub fn count(&self, x: u8, y: u8) -> usize {
        self.counts.get(&(x, y)).copied().unwrap_or(0)
    }
    /// Observed `((x,y), count)` entries in lexicographic pair order.
    ///
    /// Visits s positive entries in O(s) time without allocation.
    pub fn counts(&self) -> impl ExactSizeIterator<Item = ((u8, u8), usize)> + '_ {
        self.counts.iter().map(|(&pair, &count)| (pair, count))
    }
    /// Sums over y to recover the first input's byte histogram.
    ///
    /// `c_x[a] = sum_b c[a,b]`; takes `O(s+256)` time and fixed 256-count
    /// storage without heap allocation. Both marginals have the same total.
    pub fn marginal_x(&self) -> ByteHistogram {
        self.marginal(false)
    }
    /// Sums over x to recover the second input's byte histogram.
    ///
    /// `c_y[b] = sum_a c[a,b]`; same costs as [`Self::marginal_x`].
    pub fn marginal_y(&self) -> ByteHistogram {
        self.marginal(true)
    }
    fn marginal(&self, second: bool) -> ByteHistogram {
        let mut counts = [0usize; 256];
        for ((a, b), count) in self.counts() {
            counts[usize::from(if second { b } else { a })] += count;
        }
        // Private positive counts sum to a slice length, so no subset can overflow.
        ByteHistogram::try_from_counts(counts).expect("joint counts have a representable total")
    }
}

/// Empirical probabilities of aligned byte pairs, retaining exact [`JointCounts`].
///
/// `p(a,b) = count(a,b)/N`, where N counts pairs. For `AABB` with `XYXY`,
/// four pairs each have probability 1/4. Probabilities are derived lazily as f64;
/// nothing is smoothed or normalized. An empty state returns zero for queries
/// and no iterator entries; it is not a unit-mass probability law.
/// Owns its table independently of the input slices. See [`JointCounts`] for
/// the worked example, statistical interpretation and storage costs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointDistribution {
    counts: JointCounts,
}

/// Counts aligned observations once and exposes their empirical probabilities.
///
/// Has the same costs and conventions as [`joint_counts`]; moving counts into
/// the distribution does not allocate or recount.
///
/// # Errors
/// Returns [`LengthMismatch`] for unequal input lengths, including one empty input.
///
/// ```
/// use celandine::distribution::joint_probabilities;
/// let d = joint_probabilities(b"AABB", b"XYXY")?;
/// assert_eq!(d.probability(b'A', b'X'), 0.25);
/// assert_eq!(d.marginal_x().probability(b'A'), 0.5);
/// # Ok::<(), celandine::distribution::LengthMismatch>(())
/// ```
pub fn joint_probabilities(x: &[u8], y: &[u8]) -> Result<JointDistribution, LengthMismatch> {
    Ok(JointDistribution::from_counts(joint_counts(x, y)?))
}
impl JointDistribution {
    /// Moves an existing exact table without recounting or allocating, in O(1) time.
    pub fn from_counts(counts: JointCounts) -> Self {
        Self { counts }
    }
    /// Borrows the exact counts and total without allocation.
    pub fn counts(&self) -> &JointCounts {
        &self.counts
    }
    /// Number of paired observations.
    pub fn sample_size(&self) -> usize {
        self.counts.total()
    }
    /// Whether no pairs were observed.
    pub fn is_empty(&self) -> bool {
        self.counts.is_empty()
    }
    /// Number of observed pairs, in O(1) time.
    pub fn support_size(&self) -> usize {
        self.counts.support_size()
    }
    /// Pair count divided by sample size; positive zero for absent pairs or empty input.
    ///
    /// Takes `O(log(s+1))` time without allocation. Floating-point conversion can round.
    pub fn probability(&self, x: u8, y: u8) -> f64 {
        self.counts.count(x, y) as f64 / self.sample_size().max(1) as f64
    }
    /// Observed `((x,y), probability)` entries in lexicographic pair order.
    ///
    /// Visits s pairs in O(s) time without allocation. Empty states yield no entries.
    pub fn probabilities(&self) -> impl ExactSizeIterator<Item = ((u8, u8), f64)> + '_ {
        let total = self.sample_size().max(1) as f64;
        self.counts
            .counts()
            .map(move |(pair, count)| (pair, count as f64 / total))
    }
    /// First-coordinate law `p_x[a] = sum_b p[a,b]`, derived via exact integer sums.
    ///
    /// Takes `O(s+256)` time without heap allocation; for `AABB`/`XYXY`, A has probability 1/2.
    pub fn marginal_x(&self) -> Distribution {
        Distribution::from_counts(self.counts.marginal_x())
    }
    /// Second-coordinate law `p_y[b] = sum_a p[a,b]`, derived via exact integer sums.
    ///
    /// Same costs as [`Self::marginal_x`]; for `AABB`/`XYXY`, X has probability 1/2.
    pub fn marginal_y(&self) -> Distribution {
        Distribution::from_counts(self.counts.marginal_y())
    }
}
