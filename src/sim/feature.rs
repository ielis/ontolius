//! A module with feature definitions.
//!
//! Feature is a concept (e.g. an ontology term) that an entity (e.g. an individual or a cohort) can be annotated to.
//!
//! The features include
//! - [`PresentFeature`] - an observed feature of the tested entity.
//! - [`IndividualFeature`] - a feature tested in an individual that can be either present or excluded.
//! - [`AggregatedFeature`] - a feature observed in `n` of `m` tested individuals.
//!
//! Some features support conversions. For instance, [`PresentFeature`] can be converted into an [`IndividualFeature`],
//! which in turn can be converted into [`AggregatedFeature`]. The backward process is, however, not necessarily possible.
//!
//! The features implement notable traits, such as [`crate::Identified`] or [`crate::sim::Observed`],
//! to make them work with the rest of the crate framework.
use std::marker::PhantomData;

use crate::{sim::Observed, Identified, TermId};

use super::{Aggregated, ObservationStatus};

/// A simple wrapper around [`TermId`] to represent a phenotypic feature
/// that was observed in an individual.
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct PresentFeature<'a> {
    term_id: std::borrow::Cow<'a, TermId>,
}

/// Convert the term id reference into the borrowed variant of [`PresentFeature`].
impl<'a> From<&'a TermId> for PresentFeature<'a> {
    fn from(value: &'a TermId) -> Self {
        Self {
            term_id: std::borrow::Cow::Borrowed(value),
        }
    }
}

/// Convert the term id into the owned variant of [`PresentFeature`].
impl From<TermId> for PresentFeature<'_> {
    fn from(value: TermId) -> Self {
        Self {
            term_id: std::borrow::Cow::Owned(value),
        }
    }
}

/// Convert [`IndividualFeature`] `a` into [`PresentFeature`]
/// if `a` is present.
///
/// Returns `Err(a)` if `a` is excluded.
impl<'a> TryFrom<IndividualFeature<'a>> for PresentFeature<'a> {
    type Error = IndividualFeature<'a>;

    fn try_from(value: IndividualFeature<'a>) -> Result<Self, Self::Error> {
        match value.status {
            ObservationStatus::Present => Ok(Self {
                term_id: value.term_id,
            }),
            ObservationStatus::Excluded => Err(value),
        }
    }
}

impl Identified for PresentFeature<'_> {
    fn identifier(&self) -> &TermId {
        self.term_id.as_ref()
    }
}

/// A present feature has always the [`ObservationStatus::Present`] status.
impl Observed for PresentFeature<'_> {
    fn status(&self) -> ObservationStatus {
        ObservationStatus::Present
    }
}

#[cfg(test)]
mod test_present_feature {
    use super::PresentFeature;
    use crate::{
        common::hpo::test::{ARACHNODACTYLY, SEIZURE},
        Identified,
    };

    /// The features are equal regardless of the Cow variant.
    #[test]
    fn owned_and_borrowed_are_equal() {
        let owned = PresentFeature::from(ARACHNODACTYLY.clone());
        let borrowed = PresentFeature::from(&ARACHNODACTYLY);

        assert_eq!(owned, borrowed);
    }

    /// The Cow variant does not affect sorting.
    #[test]
    fn features_can_be_sorted() {
        let ar_owned = PresentFeature::from(ARACHNODACTYLY.clone());
        let ar_borrowed = PresentFeature::from(&ARACHNODACTYLY);
        let sei_owned = PresentFeature::from(SEIZURE.clone());
        let sei_borrowed = PresentFeature::from(&SEIZURE);

        let mut features = [&sei_borrowed, &ar_owned];
        features.sort();
        assert_eq!(features, [&ar_owned, &sei_borrowed]);

        let mut features = [&sei_owned, &ar_borrowed];
        features.sort();
        assert_eq!(features, [&ar_borrowed, &sei_owned]);
    }

    /// Borrowed variant can be converted to owned
    #[test]
    fn borrowed_can_be_converted_to_cloned() {
        let borrowed = PresentFeature::from(&ARACHNODACTYLY);
        let owned = borrowed.to_owned();

        assert_eq!(&borrowed, &owned);

        // The lifetimes are truly independent.
        drop(borrowed);
        let _ = owned.identifier();
    }
}

/// Represents a feature that can be present or excluded (see [`ObservationStatus`])
/// in an individual.
///
/// See [`PresentFeature`] for an individual feature that cannot be excluded.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IndividualFeature<'a> {
    term_id: std::borrow::Cow<'a, TermId>,
    status: ObservationStatus,
}

impl<'a> IndividualFeature<'a> {
    /// Get a builder for building a feature.
    pub fn builder() -> IndividualFeatureBuilder<'a, Unset, Unset> {
        IndividualFeatureBuilder {
            term_id: None,
            status: None,
            state: PhantomData,
        }
    }
}

impl IndividualFeature<'_> {
    /// Set the state to [`ObservationStatus::Present`].
    pub fn to_present(mut self) -> Self {
        self.status = ObservationStatus::Present;
        self
    }

    /// Set the state to [`ObservationStatus::Excluded`].
    pub fn to_excluded(mut self) -> Self {
        self.status = ObservationStatus::Excluded;
        self
    }
}

impl<'a> From<PresentFeature<'a>> for IndividualFeature<'a> {
    fn from(value: PresentFeature<'a>) -> Self {
        Self {
            term_id: value.term_id,
            status: ObservationStatus::Present,
        }
    }
}

impl<'a> From<&'a PresentFeature<'a>> for IndividualFeature<'a> {
    fn from(value: &'a PresentFeature<'a>) -> Self {
        Self {
            term_id: std::borrow::Cow::Borrowed(&value.term_id),
            status: value.status().clone(),
        }
    }
}

impl Identified for IndividualFeature<'_> {
    fn identifier(&self) -> &TermId {
        self.term_id.as_ref()
    }
}

impl Observed for IndividualFeature<'_> {
    fn status(&self) -> ObservationStatus {
        self.status
    }
}

/// A marker struct to indicate that a required field of [`IndividualFeatureBuilder`] was set.
pub struct Set;

/// A marker struct to indicate that a required field of [`IndividualFeatureBuilder`] was not set.
pub struct Unset;

/// A builder for building an [`IndividualFeature`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct IndividualFeatureBuilder<'a, T, S> {
    term_id: Option<std::borrow::Cow<'a, TermId>>,
    status: Option<ObservationStatus>,
    state: PhantomData<(T, S)>,
}

impl<'a> From<&'a TermId> for IndividualFeatureBuilder<'a, Set, Unset> {
    fn from(value: &'a TermId) -> Self {
        Self {
            term_id: Some(std::borrow::Cow::Borrowed(value)),
            status: None,
            state: PhantomData,
        }
    }
}

impl<'a> From<TermId> for IndividualFeatureBuilder<'a, Set, Unset> {
    fn from(value: TermId) -> Self {
        Self {
            term_id: Some(std::borrow::Cow::Owned(value)),
            status: None,
            state: PhantomData,
        }
    }
}

impl<'a, T, S> IndividualFeatureBuilder<'a, T, S> {
    /// Use a borrowed term id.
    pub fn borrowed(self, term_id: &'a TermId) -> IndividualFeatureBuilder<'a, Set, S> {
        IndividualFeatureBuilder {
            term_id: Some(std::borrow::Cow::Borrowed(term_id)),
            status: self.status,
            state: PhantomData,
        }
    }

    /// Use an owned term id.
    pub fn owned(self, term_id: TermId) -> IndividualFeatureBuilder<'a, Set, S> {
        IndividualFeatureBuilder {
            term_id: Some(std::borrow::Cow::Owned(term_id)),
            status: self.status,
            state: PhantomData,
        }
    }

    /// Set the observation status to *present*.
    pub fn present(self) -> IndividualFeatureBuilder<'a, T, Set> {
        self.with_status(ObservationStatus::Present)
    }

    /// Set the observation status to *excluded*.
    pub fn excluded(self) -> IndividualFeatureBuilder<'a, T, Set> {
        self.with_status(ObservationStatus::Excluded)
    }

    /// Set the observation status to provided `status` value.
    pub fn with_status(self, status: ObservationStatus) -> IndividualFeatureBuilder<'a, T, Set> {
        IndividualFeatureBuilder {
            term_id: self.term_id,
            status: Some(status),
            state: PhantomData,
        }
    }
}

impl<'a> IndividualFeatureBuilder<'a, Set, Set> {
    /// Build the final individual feature.
    pub fn build(self) -> IndividualFeature<'a> {
        IndividualFeature {
            term_id: self
                .term_id
                .expect("build can be called only after term_id is set"),
            status: self
                .status
                .expect("Build can only be called after status is set"),
        }
    }
}

/// A feature that has been found present in `n` out of `m` tested individuals.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AggregatedFeature<'a> {
    term_id: std::borrow::Cow<'a, TermId>,
    n: u32,
    m: u32,
}

impl<'a> From<PresentFeature<'a>> for AggregatedFeature<'a> {
    fn from(value: PresentFeature<'a>) -> Self {
        Self {
            n: value.n(),
            m: value.m(),
            term_id: value.term_id,
        }
    }
}

impl<'a> From<IndividualFeature<'a>> for AggregatedFeature<'a> {
    fn from(value: IndividualFeature<'a>) -> Self {
        Self {
            n: value.n(),
            m: value.m(),
            term_id: value.term_id,
        }
    }
}

impl Identified for AggregatedFeature<'_> {
    fn identifier(&self) -> &TermId {
        self.term_id.as_ref()
    }
}

impl Aggregated for AggregatedFeature<'_> {
    fn n(&self) -> u32 {
        self.n
    }

    fn m(&self) -> u32 {
        self.m
    }
}
