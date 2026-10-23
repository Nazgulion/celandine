//! Distances between empirical byte distributions.
//!
//! [`total_variation_distance`] compares relative frequencies on the shared
//! byte alphabet. Samples may have different lengths; neither may be empty.
//! Jensen–Shannon divergence compares each law to their equally weighted
//! mixture; its square root is exposed separately as a distance.

mod jensen_shannon;
mod total_variation;

pub use jensen_shannon::{
    jensen_shannon_distance, jensen_shannon_distance_distribution, jensen_shannon_divergence,
    jensen_shannon_divergence_distribution,
};
pub use total_variation::{total_variation_distance, total_variation_distance_distribution};

/// At least one input has no observations and therefore no probability law.
///
/// Distribution comparisons require two nonempty laws, even when both inputs
/// are empty. No probabilities are inferred from missing observations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmptyDistribution;

impl std::fmt::Display for EmptyDistribution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("comparison requires two nonempty distributions")
    }
}

impl std::error::Error for EmptyDistribution {}
