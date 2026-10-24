//! Borrowed byte transformations for composing sequence measurements.
//!
//! [`ngrams`] extracts every overlapping block; [`windows`] selects complete
//! blocks at a positive byte stride. [`blocks`] partitions into adjacent complete
//! blocks with an inspectable tail. These operations borrow without allocation.

#![doc = include_str!("../../docs/ngrams.md")]

mod blocks;
mod ngrams;
mod windows;

pub use blocks::{InvalidBlockSize, blocks};
pub use ngrams::{InvalidNgramLength, ngrams};
pub use windows::{InvalidWindowParameters, windows};
