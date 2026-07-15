use crate::TermId;

/// Compute semantic similarity between a pair of items `a` and `b`.
///
/// The computation is infallible.
pub trait SimilarityMeasure<I> {
    type Sim;

    /// Compute the semantic similarity
    fn compute(&self, a: &I, b: &I) -> Self::Sim;
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

pub trait PresentFeatures {
    fn present_features(&self) -> impl Iterator<Item = &TermId>;
}

impl<T> PresentFeatures for T
where
    T: AsRef<[TermId]>,
{
    fn present_features(&self) -> impl Iterator<Item = &TermId> {
        self.as_ref().iter()
    }
}

pub mod concrete {
    use std::borrow::Cow;

    use crate::{Identified, TermId};

    use super::RatioAware;

    pub struct IndividualTermId<'a> {
        term_id: std::borrow::Cow<'a, TermId>,
        is_present: bool,
    }

    impl<'a> IndividualTermId<'a> {
        pub fn present(term_id: TermId) -> Self {
            Self {
                term_id: Cow::Owned(term_id),
                is_present: true,
            }
        }
        pub fn excluded(term_id: TermId) -> Self {
            Self {
                term_id: Cow::Owned(term_id),
                is_present: false,
            }
        }
    }

    impl Identified for IndividualTermId<'_> {
        fn identifier(&self) -> &TermId {
            self.term_id.as_ref()
        }
    }

    impl<'a> Identified for &'a IndividualTermId<'_> {
        fn identifier(&self) -> &TermId {
            (*self).identifier()
        }
    }
    impl RatioAware for IndividualTermId<'_> {
        fn n(&self) -> u32 {
            if self.is_present {
                1
            } else {
                0
            }
        }

        fn m(&self) -> u32 {
            1
        }
    }

    impl<'a> RatioAware for &'a IndividualTermId<'_> {
        fn n(&self) -> u32 {
            (*self).n()
        }

        fn m(&self) -> u32 {
            (*self).m()
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
