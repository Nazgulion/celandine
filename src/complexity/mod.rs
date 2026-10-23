//! Exact combinatorial measures of observed finite byte sequences.
//!
//! Subword complexity counts distinct contiguous blocks, preserving local order.
//! See [`subword_complexity`] for its definition, history, and limits.
//! [`lz76_complexity`] counts phrases in an exhaustive left-to-right history.

mod lz76;
pub use lz76::lz76_complexity;

use crate::distribution::{ByteHistogram, ngram_counts};

#[doc = include_str!("../../docs/complexity/subword.md")]
pub fn subword_complexity(data: &[u8], n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    if n > data.len() {
        return 0;
    }
    if n == data.len() {
        return 1;
    }
    if n == 1 {
        return ByteHistogram::from_bytes(data).support_size();
    }
    ngram_counts(data, n)
        .expect("the zero-length case was handled above")
        .support_size()
}

/// Lazily yields `p_x(n)` for block lengths `n = 0..=data.len()`.
///
/// Each value counts distinct contiguous blocks of that byte length; units are
/// blocks, not bits. This shows how observed pattern diversity changes with
/// length. For the definition and historical references, see [`subword_complexity`].
/// The empty-word entry is always one, so empty input yields `[1]`.
/// The finite profile need not increase: `ABCD` yields `[1, 4, 3, 2, 1]`.
///
/// Construction and cloning take `O(1)` time and allocate nothing. The iterator
/// borrows the input and computes each requested length independently; it does
/// not cache results. Skipped lengths are not computed. Use `.take(max_n + 1)`
/// to bound work (ensure the addition fits), or `.enumerate()` to retain lengths.
/// Each evaluated entry has the costs of [`subword_complexity`]. Consuming the
/// full profile has a conservative `O(L^3 log(L + 1))` worst-case time bound and
/// `O(L)` peak temporary tree storage for input length `L`; collecting also owns
/// `L + 1` output integers. Full profiles are intended for short inputs.
///
/// ```
/// use celandine::complexity::subword_complexity_profile;
/// let profile: Vec<_> = subword_complexity_profile(b"ABABA").collect();
/// assert_eq!(profile, [1, 2, 2, 2, 2, 1]);
/// assert_eq!(subword_complexity_profile(b"").collect::<Vec<_>>(), [1]);
/// assert_eq!(subword_complexity_profile(b"ABABA").take(3).sum::<usize>(), 5);
/// ```
pub fn subword_complexity_profile(
    data: &[u8],
) -> impl ExactSizeIterator<Item = usize> + DoubleEndedIterator + std::iter::FusedIterator + Clone + '_
{
    // A valid u8 slice occupies at most isize::MAX bytes, so this endpoint fits.
    Profile {
        data,
        lengths: 0..data.len() + 1,
    }
}

// Map preserves closure side effects while skipping. This iterator skips the
// length range first, so nth/nth_back never build tables for discarded entries.
#[derive(Clone)]
struct Profile<'a> {
    data: &'a [u8],
    lengths: std::ops::Range<usize>,
}

impl Iterator for Profile<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        self.lengths
            .next()
            .map(|n| subword_complexity(self.data, n))
    }

    fn nth(&mut self, n: usize) -> Option<usize> {
        self.lengths
            .nth(n)
            .map(|n| subword_complexity(self.data, n))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lengths.size_hint()
    }
}

impl DoubleEndedIterator for Profile<'_> {
    fn next_back(&mut self) -> Option<usize> {
        self.lengths
            .next_back()
            .map(|n| subword_complexity(self.data, n))
    }

    fn nth_back(&mut self, n: usize) -> Option<usize> {
        self.lengths
            .nth_back(n)
            .map(|n| subword_complexity(self.data, n))
    }
}

impl ExactSizeIterator for Profile<'_> {}
impl std::iter::FusedIterator for Profile<'_> {}
