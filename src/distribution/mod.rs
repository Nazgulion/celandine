#![doc = include_str!("../../docs/distribution.md")]

mod counts;
mod probability;

pub use counts::{ByteHistogram, CountOverflow};
pub use probability::Distribution;
