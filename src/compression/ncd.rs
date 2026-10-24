//! Ordered normalized compression distance and its measured components.
use super::Compressor;
use std::{collections::TryReserveError, error::Error, fmt};

/// Complete compressed byte sizes used by ordered NCD.
///
/// Retain these to inspect framing and concatenation effects without recompressing.
/// All three sizes must use the same fresh-stream compressor and configuration.
/// For a literal format with one header byte, `x = AB`, `y = CD` gives
/// `(3, 3, 5)` and distance `(5 - 3) / 3 = 2/3`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompressionDistanceComponents {
    /// `C(x)`: complete compressed size of the left input, in bytes.
    pub x: usize,
    /// `C(y)`: complete compressed size of the right input, in bytes.
    pub y: usize,
    /// `C(xy)`: complete compressed size of left then right, without a separator.
    pub xy: usize,
}

/// Both individual compressed sizes are zero, so NCD's denominator is zero.
///
/// For example, components `(0, 0, 1)` cannot be normalized. This concerns encoded
/// sizes, not empty inputs: a framed compressor can encode empty input nontrivially.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZeroCompressedLength;

impl fmt::Display for ZeroCompressedLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NCD requires at least one nonzero individual compressed length")
    }
}
impl Error for ZeroCompressedLength {}

impl CompressionDistanceComponents {
    /// Compute `(xy - min(x, y)) / max(x, y)` without further compression.
    ///
    /// Subtracts integer sizes before conversion to preserve small differences
    /// above 2^53. Negative results are retained. O(1) time and space, no allocation.
    ///
    /// # Errors
    /// Returns [`ZeroCompressedLength`] if both individual compressed sizes are zero.
    pub fn normalized_distance(self) -> Result<f64, ZeroCompressedLength> {
        let denominator = self.x.max(self.y);
        if denominator == 0 {
            return Err(ZeroCompressedLength);
        }
        let minimum = self.x.min(self.y);
        let numerator = if self.xy >= minimum {
            (self.xy - minimum) as f64
        } else {
            -((minimum - self.xy) as f64)
        };
        Ok(numerator / denominator as f64)
    }
}

/// Failure to measure ordered compression distance.
///
/// Backend errors preserve their source; concatenation errors distinguish size
/// overflow from failed buffer reservation. No partial components are returned.
/// `ZeroDenominator` occurs only when normalizing two zero encoded sizes.
#[derive(Debug)]
pub enum CompressionDistanceError<E> {
    /// The compressor failed on an individual or concatenated input.
    Compressor(E),
    /// The sum of input lengths cannot be represented by usize.
    LengthOverflow,
    /// A contiguous concatenation buffer could not be reserved.
    Allocation(TryReserveError),
    /// Both individual compressed sizes are zero.
    ZeroDenominator,
}

impl<E: fmt::Display> fmt::Display for CompressionDistanceError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compressor(error) => write!(f, "compression failed: {error}"),
            Self::LengthOverflow => f.write_str("concatenated input length exceeds usize"),
            Self::Allocation(error) => write!(f, "cannot reserve concatenation buffer: {error}"),
            Self::ZeroDenominator => ZeroCompressedLength.fmt(f),
        }
    }
}
impl<E: Error + 'static> Error for CompressionDistanceError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Compressor(error) => Some(error),
            Self::Allocation(error) => Some(error),
            _ => None,
        }
    }
}

/// Measure `C(x)`, `C(y)`, and `C(xy)` with three fresh compressor calls.
///
/// Calls in that order, including empty inputs. Concatenation is raw bytes with
/// no delimiter or length prefix. If both inputs are nonempty, temporarily copies
/// them into a fallibly reserved buffer of `x.len() + y.len()` bytes. Otherwise
/// borrows the nonempty input (or empty slice). No compression state is shared.
///
/// Time: three backend calls plus O(|x| + |y|) copying. Peak auxiliary space:
/// O(|x| + |y|) plus backend workspace. Does not compute `C(yx)`; swap arguments
/// explicitly to investigate order dependence. See [`normalized_compression_distance`].
///
/// # Errors
/// Propagates backend errors; reports length overflow or buffer reservation
/// failure. Returns components even when both individual compressed sizes are zero.
pub fn compression_distance_components<C: Compressor + ?Sized>(
    x: &[u8],
    y: &[u8],
    compressor: &C,
) -> Result<CompressionDistanceComponents, CompressionDistanceError<C::Error>> {
    let length = x
        .len()
        .checked_add(y.len())
        .ok_or(CompressionDistanceError::LengthOverflow)?;
    let cx = compressor
        .compressed_len(x)
        .map_err(CompressionDistanceError::Compressor)?;
    let cy = compressor
        .compressed_len(y)
        .map_err(CompressionDistanceError::Compressor)?;
    let mut joined = Vec::new();
    let xy = if x.is_empty() {
        y
    } else if y.is_empty() {
        x
    } else {
        joined
            .try_reserve_exact(length)
            .map_err(CompressionDistanceError::Allocation)?;
        joined.extend_from_slice(x);
        joined.extend_from_slice(y);
        &joined
    };
    let cxy = compressor
        .compressed_len(xy)
        .map_err(CompressionDistanceError::Compressor)?;
    Ok(CompressionDistanceComponents {
        x: cx,
        y: cy,
        xy: cxy,
    })
}

#[doc = include_str!("../../docs/compression/ncd.md")]
pub fn normalized_compression_distance<C: Compressor + ?Sized>(
    x: &[u8],
    y: &[u8],
    compressor: &C,
) -> Result<f64, CompressionDistanceError<C::Error>> {
    compression_distance_components(x, y, compressor)?
        .normalized_distance()
        .map_err(|_| CompressionDistanceError::ZeroDenominator)
}
