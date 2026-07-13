use crate::TermId;

pub trait SimilarityMeasure<I> {
    type Sim;

    fn compute(&self, a: &I, b: &I) -> Self::Sim;
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

pub trait Observed {
    fn status(&self) -> ObservationStatus;
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
