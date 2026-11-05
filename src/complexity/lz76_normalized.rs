use super::lz76_complexity;
use crate::distribution::ByteHistogram;
use std::fmt;

/// An invalid declared alphabet for asymptotically normalized LZ76 complexity.
///
/// Range is checked before observed support, even for empty input. For example,
/// `ABC` with `K=2` yields `AlphabetTooSmall`; `K=1` yields `InvalidAlphabetSize`.
/// Labels are arbitrary bytes, so `AB` fits `K=2`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lz76NormalizationError {
    /// The declared alphabet cardinality is outside `2..=256`.
    InvalidAlphabetSize,
    /// The number of observed byte values exceeds the declared cardinality.
    AlphabetTooSmall,
}
impl fmt::Display for Lz76NormalizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidAlphabetSize => "alphabet size must be between 2 and 256",
            Self::AlphabetTooSmall => "observed support exceeds the declared alphabet size",
        })
    }
}
impl std::error::Error for Lz76NormalizationError {}

/// Reusable LZ76 phrase count, input length and observed alphabet cardinality.
///
/// Parses once and owns three integers, without retaining the input. This lets
/// callers compare normalization conventions without repeating the quadratic
/// parser. Construction takes O(N² + 256) worst-case time, fixed auxiliary
/// storage and no heap allocation; all queries take O(1) time and no allocation.
/// See [`lz76_complexity_asymptotic`] for formulas, history and finite-size limits.
///
/// ```
/// use celandine::complexity::Lz76Summary;
/// let s = Lz76Summary::from_bytes(b"ABAB"); // A | B | AB
/// assert_eq!((s.len(), s.phrase_count(), s.support_size()), (4, 3, 2));
/// assert_eq!(s.phrase_density(), 0.75);
/// assert_eq!(s.asymptotic(2), Ok(1.5));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lz76Summary {
    len: usize,
    phrase_count: usize,
    support_size: usize,
}
impl Lz76Summary {
    /// Counts exhaustive phrases and distinct byte labels in one input.
    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            len: data.len(),
            phrase_count: lz76_complexity(data),
            support_size: ByteHistogram::from_bytes(data).support_size(),
        }
    }
    /// Input length in byte symbols.
    pub fn len(&self) -> usize {
        self.len
    }
    /// Whether there were no input symbols.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Exact exhaustive phrase count, including a final reproducible suffix.
    pub fn phrase_count(&self) -> usize {
        self.phrase_count
    }
    /// Number of distinct observed byte labels, not an inferred source alphabet.
    pub fn support_size(&self) -> usize {
        self.support_size
    }
    /// `c/N`, or zero for empty input; see [`lz76_phrase_density`].
    pub fn phrase_density(&self) -> f64 {
        density(self.phrase_count, self.len)
    }
    /// `c*log_K(N)/N`, with the domain and errors of [`lz76_complexity_asymptotic`].
    ///
    /// Uses cached support for validation; does not reparse or retain the input.
    pub fn asymptotic(&self, alphabet_size: usize) -> Result<f64, Lz76NormalizationError> {
        validate_alphabet(alphabet_size)?;
        validate_support(self.support_size, alphabet_size)?;
        Ok(asymptotic(self.phrase_count, self.len, alphabet_size))
    }
}

/// Fraction of input symbols that begin an exhaustive LZ76 phrase: `c/N`.
///
/// Here `c` is [`lz76_complexity`] and `N` is the byte length. This dimensionless
/// library rescaling lies in `[0,1]`, with empty input defined as zero and a
/// singleton as one. It is useful for describing phrase density, not estimating
/// entropy rate. `ABAB` parses as `A | B | AB`, giving `3/4 = 0.75`.
/// It inherits the raw parser's O(N²) worst-case time and allocation-free O(1)
/// auxiliary storage. See [`lz76_complexity_asymptotic`] for history and the
/// distinct asymptotic convention. No alphabet parameter is needed for `c/N`.
///
/// ```
/// use celandine::complexity::lz76_phrase_density;
/// assert_eq!(lz76_phrase_density(b"ABAB"), 0.75);
/// assert_eq!(lz76_phrase_density(b""), 0.0);
/// ```
pub fn lz76_phrase_density(data: &[u8]) -> f64 {
    density(lz76_complexity(data), data.len())
}

#[doc = include_str!("../../docs/complexity/lz76_normalized.md")]
pub fn lz76_complexity_asymptotic(
    data: &[u8],
    alphabet_size: usize,
) -> Result<f64, Lz76NormalizationError> {
    // Reject invalid domains before the potentially quadratic parse.
    validate_alphabet(alphabet_size)?;
    validate_support(
        ByteHistogram::from_bytes(data).support_size(),
        alphabet_size,
    )?;
    Ok(asymptotic(lz76_complexity(data), data.len(), alphabet_size))
}
fn density(count: usize, len: usize) -> f64 {
    if len == 0 {
        0.0
    } else {
        count as f64 / len as f64
    }
}
fn asymptotic(count: usize, len: usize, alphabet: usize) -> f64 {
    if len <= 1 {
        0.0
    } else {
        density(count, len) * ((len as f64).log2() / (alphabet as f64).log2())
    }
}
fn validate_alphabet(size: usize) -> Result<(), Lz76NormalizationError> {
    if (2..=256).contains(&size) {
        Ok(())
    } else {
        Err(Lz76NormalizationError::InvalidAlphabetSize)
    }
}
fn validate_support(support: usize, alphabet: usize) -> Result<(), Lz76NormalizationError> {
    if support <= alphabet {
        Ok(())
    } else {
        Err(Lz76NormalizationError::AlphabetTooSmall)
    }
}
