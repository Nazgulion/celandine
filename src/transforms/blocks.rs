use std::{fmt, slice::ChunksExact};

/// A zero block size, which cannot partition bytes into nonempty fixed blocks.
///
/// Size counts bytes and must be positive, even for empty input. For example,
/// `blocks(b"AB", 0)` returns this error instead of panicking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidBlockSize;

impl fmt::Display for InvalidBlockSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("block size must be greater than zero")
    }
}

impl std::error::Error for InvalidBlockSize {}

#[doc = include_str!("../../docs/transforms/blocks.md")]
pub fn blocks(data: &[u8], size: usize) -> Result<ChunksExact<'_, u8>, InvalidBlockSize> {
    if size == 0 {
        return Err(InvalidBlockSize);
    }
    Ok(data.chunks_exact(size))
}
