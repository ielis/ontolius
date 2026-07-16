use std::marker::PhantomData;

use crate::{Identified, TermId};

/// Compute semantic similarity between a pair of annotated items `a` and `b`.
///
/// The computation is infallible.
pub trait SimilarityMeasure<T> {
    type Sim;

    /// Compute the semantic similarity
    fn compute(&self, a: &[T], b: &[T]) -> Self::Sim;
}

/// Implemented by features (e.g. ontology terms) that were assessed
/// and found to be present in `n` of `m` annotated items (e.g. individuals).
pub trait RatioAware {
    fn n(&self) -> u32;
    fn m(&self) -> u32;
    fn frequency(&self) -> f64 {
        self.n() as f64 / self.m() as f64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum ObservationStatus {
    Present,
    Excluded,
}

impl ObservationStatus {
    pub fn is_present(&self) -> bool {
        return *self == ObservationStatus::Present;
    }

    pub fn is_excluded(&self) -> bool {
        return *self == ObservationStatus::Excluded;
    }
}

/// Automatically implemented for all [`RatioAware`] types.
/// No need to implement manually.
pub trait Observed {
    fn status(&self) -> ObservationStatus;
}

impl<T> Observed for T
where
    T: RatioAware,
{
    fn status(&self) -> ObservationStatus {
        if self.n() > 0 {
            ObservationStatus::Present
        } else {
            ObservationStatus::Excluded
        }
    }
}

/// A simple wrapper around [`TermId`] to represent a phenotypic feature
/// that was observed in an individual.
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct PresentFeature<'a> {
    term_id: std::borrow::Cow<'a, TermId>,
}

impl<'a> From<&'a TermId> for PresentFeature<'a> {
    fn from(value: &'a TermId) -> Self {
        Self {
            term_id: std::borrow::Cow::Borrowed(value),
        }
    }
}

impl From<TermId> for PresentFeature<'_> {
    fn from(value: TermId) -> Self {
        Self {
            term_id: std::borrow::Cow::Owned(value),
        }
    }
}

/// Convert [`IndividualFeature`] $a$ into [`PresentFeature`]
/// if $a$ is present.
/// 
/// Returns `Err(a)` if $a$ is excluded.
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

impl<'a> Identified for &'a PresentFeature<'_> {
    fn identifier(&self) -> &TermId {
        (*self).identifier()
    }
}

impl RatioAware for PresentFeature<'_> {
    fn n(&self) -> u32 {
        1
    }

    fn m(&self) -> u32 {
        1
    }
}

impl<'a> RatioAware for &'a PresentFeature<'_> {
    fn n(&self) -> u32 {
        (*self).n()
    }

    fn m(&self) -> u32 {
        (*self).m()
    }
}

#[cfg(test)]
mod test_present_feature {
    use crate::{
        common::hpo::test::{ARACHNODACTYLY, SEIZURE},
        sim::base::PresentFeature,
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

impl Identified for IndividualFeature<'_> {
    fn identifier(&self) -> &TermId {
        self.term_id.as_ref()
    }
}

impl<'a> Identified for &'a IndividualFeature<'_> {
    fn identifier(&self) -> &TermId {
        (*self).identifier()
    }
}
impl RatioAware for IndividualFeature<'_> {
    fn n(&self) -> u32 {
        match self.status {
            ObservationStatus::Present => 1,
            ObservationStatus::Excluded => 0,
        }
    }

    fn m(&self) -> u32 {
        1
    }
}

impl<'a> RatioAware for &'a IndividualFeature<'_> {
    fn n(&self) -> u32 {
        (*self).n()
    }

    fn m(&self) -> u32 {
        (*self).m()
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

// pub struct Ratio {
//     n: u32,
//     m: u32,
// }

// #[derive(Debug, Clone)]
// pub struct RatioParseError {
//     n: u32,
//     m: u32,
// }

// impl std::fmt::Display for RatioParseError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Invalid ratio {}/{}", self.n, self.m)
//     }
// }

// impl std::error::Error for RatioParseError {}

// impl TryFrom<(u32, u32)> for Ratio {
//     type Error = RatioParseError;

//     fn try_from(value: (u32, u32)) -> Result<Self, Self::Error> {
//         if value.0 <= value.1 {
//             Ok(Ratio {
//                 n: value.0,
//                 m: value.1,
//             })
//         } else {
//             Err(RatioParseError {
//                 n: value.0,
//                 m: value.1,
//             })
//         }
//     }
// }
