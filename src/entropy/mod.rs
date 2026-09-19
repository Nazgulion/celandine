//! Empirical entropy of discrete byte distributions.

mod hartley;
mod shannon;

pub use hartley::{hartley_entropy, hartley_entropy_distribution};
pub use shannon::{shannon, shannon_distribution};
