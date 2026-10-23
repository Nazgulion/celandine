//! Distances between empirical byte distributions.
//!
//! [`total_variation_distance`] compares relative frequencies on the shared
//! byte alphabet. Samples may have different lengths; neither may be empty.

mod total_variation;

pub use total_variation::{
    EmptyDistribution, total_variation_distance, total_variation_distance_distribution,
};
