mod base;
pub mod ic;
pub mod phenomizer;
pub mod setsim;
// #[cfg(test)]
// pub(super) mod test;

pub use base::{
    IndividualFeature, IndividualFeatureBuilder, ObservationStatus, Observed, PresentFeature,
    SimilarityMeasure,
};
