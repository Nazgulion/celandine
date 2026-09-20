#![doc = include_str!("../../docs/ngrams.md")]

mod ngrams;

pub use ngrams::{InvalidNgramLength, ngrams};
