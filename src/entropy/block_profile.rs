use super::block_entropy;

#[doc = include_str!("../../docs/entropy/block_profile.md")]
///
/// # Example
/// ```
/// use celandine::entropy::block_entropy_profile;
/// let h: Vec<_> = block_entropy_profile(b"AABB", 4).collect();
/// assert_eq!(h[0], 1.0);
/// assert!((h[1] - 3.0_f64.log2()).abs() < 1e-12);
/// assert_eq!(&h[2..], &[1.0, 0.0]);
/// assert_eq!(block_entropy_profile(b"AABB", usize::MAX).len(), 4);
/// assert_eq!(block_entropy_profile(b"", 8).count(), 0);
/// ```
pub fn block_entropy_profile(
    data: &[u8],
    max_order: usize,
) -> impl ExactSizeIterator<Item = f64> + DoubleEndedIterator + std::iter::FusedIterator + Clone + '_
{
    // A byte slice has length at most isize::MAX, so the exclusive endpoint fits.
    Profile {
        data,
        lengths: 1..data.len().min(max_order) + 1,
    }
}

#[derive(Clone)]
struct Profile<'a> {
    data: &'a [u8],
    lengths: std::ops::Range<usize>,
}

impl Profile<'_> {
    fn entropy(&self, n: usize) -> f64 {
        block_entropy(self.data, n).expect("profile lengths are strictly positive")
    }
}

impl Iterator for Profile<'_> {
    type Item = f64;

    fn next(&mut self) -> Option<f64> {
        self.lengths.next().map(|n| self.entropy(n))
    }

    fn nth(&mut self, n: usize) -> Option<f64> {
        // Skip lengths before evaluating entropy; Map would evaluate skips.
        self.lengths.nth(n).map(|n| self.entropy(n))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lengths.size_hint()
    }

    fn count(self) -> usize {
        self.lengths.len()
    }

    fn last(mut self) -> Option<f64> {
        self.lengths.next_back().map(|n| self.entropy(n))
    }
}

impl DoubleEndedIterator for Profile<'_> {
    fn next_back(&mut self) -> Option<f64> {
        self.lengths.next_back().map(|n| self.entropy(n))
    }

    fn nth_back(&mut self, n: usize) -> Option<f64> {
        self.lengths.nth_back(n).map(|n| self.entropy(n))
    }
}

impl ExactSizeIterator for Profile<'_> {}
impl std::iter::FusedIterator for Profile<'_> {}
