#![doc = include_str!("../../docs/distribution.md")]

mod counts;
mod ngrams;
mod probability;

pub use counts::{ByteHistogram, CountOverflow};
pub use ngrams::{NgramCounts, NgramDistribution, ngram_counts, ngram_probabilities};
pub use probability::Distribution;
