use std::{fmt, slice::Windows};

/// A zero n-gram length, which cannot describe an observed nonempty block.
///
/// All three n-gram constructors reject zero, including for empty input.
/// For example, `ngrams(b"AB", 0)` returns this error instead of panicking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidNgramLength;

impl fmt::Display for InvalidNgramLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("n-gram length must be greater than zero")
    }
}

impl std::error::Error for InvalidNgramLength {}

/// Borrows every contiguous `n`-byte block, advancing one byte at a time.
///
/// For input length `L`, yields `L - n + 1` slices if `1 <= n <= L`, otherwise
/// no slices for valid `n`. There is no padding, wrapping, or text decoding.
/// These local blocks retain order information that a byte histogram discards.
///
/// Construction takes `O(1)` time and storage; each iterator step takes `O(1)`
/// time without copying bytes or allocating. The iterator is exact-size,
/// double-ended, and fused. Empty input and `n > L` produce an empty iterator.
///
/// # Errors
/// Returns [`InvalidNgramLength`] for `n == 0`, even for empty input.
///
/// # Example
/// ```
/// use celandine::transforms::ngrams;
/// let blocks: Vec<_> = ngrams(b"ABABA", 2)?.collect();
/// assert_eq!(blocks, vec![b"AB", b"BA", b"AB", b"BA"]);
/// # Ok::<(), celandine::transforms::InvalidNgramLength>(())
/// ```
pub fn ngrams(data: &[u8], n: usize) -> Result<Windows<'_, u8>, InvalidNgramLength> {
    if n == 0 {
        return Err(InvalidNgramLength);
    }
    Ok(data.windows(n))
}
