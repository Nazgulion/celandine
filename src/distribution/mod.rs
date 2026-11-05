#![doc = include_str!("../../docs/distribution.md")]

mod counts;
mod joint;
mod ngrams;
mod probability;

pub use counts::{ByteHistogram, CountOverflow};
pub use ngrams::{NgramCounts, NgramDistribution, ngram_counts, ngram_probabilities};
pub use probability::Distribution;

pub use joint::{
    JointCounts, JointDistribution, LengthMismatch, joint_counts, joint_probabilities,
};
