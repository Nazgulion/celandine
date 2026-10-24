//! Borrowed byte transformations for composing sequence measurements.
//!
//! [`ngrams`] extracts every overlapping block; [`windows`] selects complete
//! blocks at a positive byte stride. Neither copies bytes or allocates storage.

#![doc = include_str!("../../docs/ngrams.md")]

mod ngrams;
mod windows;

pub use ngrams::{InvalidNgramLength, ngrams};
pub use windows::{InvalidWindowParameters, windows};
