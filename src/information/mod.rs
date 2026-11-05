//! Shared information in aligned empirical byte observations.
//!
//! [`mutual_information`] measures dependence in bits per pair. Its reusable
//! distribution form avoids recounting inputs. Neither establishes causation
//! nor estimates a source's dependence without finite-sample limitations.
mod mutual;
pub use mutual::{mutual_information, mutual_information_distribution};
