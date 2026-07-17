//! Semantic similarity methods.
mod base;
pub mod feature;
pub mod ic;
pub mod phenomizer;
pub mod setsim;
#[cfg(test)]
pub(super) mod test;

pub use base::{
    Aggregated, Individual, ObservationStatus, Observed, Ratio, SimilarityMatrix,
    SimilarityMatrixCreator, SimilarityMeasure,
};
