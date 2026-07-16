//! Semantic similarity methods.
mod base;
pub mod feature;
pub mod ic;
pub mod phenomizer;
pub mod setsim;
// #[cfg(test)]
// pub(super) mod test;

pub use base::{Aggregated, ObservationStatus, Observed, Ratio, SimilarityMeasure};
