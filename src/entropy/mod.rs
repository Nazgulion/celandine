//! Empirical entropy of discrete byte distributions.

mod hartley;
mod renyi;
mod shannon;

pub use hartley::{hartley_entropy, hartley_entropy_distribution};
pub use renyi::{InvalidRenyiOrder, renyi_entropy, renyi_entropy_distribution};
pub use shannon::{shannon, shannon_distribution};
