#![doc = include_str!("../../docs/compression.md")]

use std::{error::Error, fmt};

#[cfg(feature = "compression-deflate")]
mod deflate;
#[cfg(feature = "compression-deflate")]
pub use deflate::Zlib;

/// A lossless compressor's complete encoded size, in bytes.
///
/// Implementations must start a fresh stream for each call, include finalization
/// and framing, and document their format, settings, dictionaries, and failures.
/// Fixed inputs and settings should give repeatable lengths. This lets callers
/// compare compressibility without prescribing one codec or retaining output.
/// See the module's worked example for a minimal custom implementation.
pub trait Compressor {
    /// The backend's failure type; use `Infallible` for an infallible compressor.
    type Error;

    /// Return the complete stream size in bytes, including for empty input.
    ///
    /// # Errors
    /// Return the backend error rather than a partial or truncated size.
    fn compressed_len(&self, input: &[u8]) -> Result<usize, Self::Error>;
}

/// Measure `C(x)`, the complete compressed size of bytes `x`.
///
/// Framing and finalization are included; no overhead is subtracted. Empty input
/// is passed to the compressor. Cost and possible errors depend on the backend.
/// See the module's worked example.
///
/// # Errors
/// Propagates the compressor's error unchanged.
pub fn compressed_length<C: Compressor + ?Sized>(
    input: &[u8],
    compressor: &C,
) -> Result<usize, C::Error> {
    compressor.compressed_len(input)
}

/// Why a dimensionless compression ratio could not be measured.
///
/// Separates a zero denominator from an actual compressor failure. For example,
/// `compression_ratio(b"", &codec)` returns `EmptyInput` without invoking `codec`.
#[derive(Debug, PartialEq, Eq)]
pub enum CompressionRatioError<E> {
    /// The input has zero bytes, so `C(x) / |x|` is undefined.
    EmptyInput,
    /// The compressor could not produce a complete stream length.
    Compressor(E),
}

impl<E: fmt::Display> fmt::Display for CompressionRatioError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => f.write_str("compression ratio requires nonempty input"),
            Self::Compressor(error) => write!(f, "compression failed: {error}"),
        }
    }
}

impl<E: Error + 'static> Error for CompressionRatioError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EmptyInput => None,
            Self::Compressor(error) => Some(error),
        }
    }
}

/// Measure `R(x) = C(x) / |x|`, in compressed bytes per input byte.
///
/// This dimensionless ratio can exceed one, especially for short inputs. It
/// measures one compressor's performance, not entropy or Kolmogorov complexity.
/// Integer sizes are converted to `f64` before division; sizes above 2^53 may
/// round. Performs one compression, with its time and space costs.
///
/// # Errors
/// Returns [`CompressionRatioError::EmptyInput`] without calling the compressor
/// for empty input, or wraps the backend error. See the module's worked example.
pub fn compression_ratio<C: Compressor + ?Sized>(
    input: &[u8],
    compressor: &C,
) -> Result<f64, CompressionRatioError<C::Error>> {
    if input.is_empty() {
        return Err(CompressionRatioError::EmptyInput);
    }
    let length = compressed_length(input, compressor).map_err(CompressionRatioError::Compressor)?;
    Ok(length as f64 / input.len() as f64)
}
