/// Compute semantic similarity between a pair of annotated items `a` and `b`.
///
/// The computation is infallible.
pub trait SimilarityMeasure<T> {
    type Sim;

    /// Compute the semantic similarity
    fn compute(&self, a: &[T], b: &[T]) -> Self::Sim;
}

/// Status of an entity that can be present or excluded.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum ObservationStatus {
    /// The entity is present.
    Present,
    /// The entity is excluded.
    Excluded,
}

impl ObservationStatus {
    pub fn is_present(&self) -> bool {
        *self == ObservationStatus::Present
    }

    pub fn is_excluded(&self) -> bool {
        *self == ObservationStatus::Excluded
    }
}

/// Represents status of a feature (e.g. an HPO term)
/// in one or multiple individuals.
pub trait Observed {
    fn status(&self) -> ObservationStatus;

    fn is_present(&self) -> bool {
        self.status().is_present()
    }

    fn is_excluded(&self) -> bool {
        self.status().is_excluded()
    }
}

impl<T> Observed for &T
where
    T: Observed,
{
    fn status(&self) -> ObservationStatus {
        (*self).status()
    }
}

impl<T> Observed for Box<T>
where
    T: Observed,
{
    fn status(&self) -> ObservationStatus {
        (**self).status()
    }
}

/// Implemented by features (e.g. ontology terms) that were assessed
/// and found to be present in `n` of `m` annotated items (e.g. individuals).
///
/// Any aggregated feature can also be used as [`Observed`],
/// since it conveys information about one or more entities.
/// 
/// # Implementation notes
///
/// The implementors must ensure that `n<=m`.
pub trait Aggregated {
    /// Get the numerator - the count of entities `n` that were found to be positive
    /// out of the `m` tested entities.
    fn n(&self) -> u32;

    /// Get the denominator - the count of `m` tested entities.
    fn m(&self) -> u32;

    /// Get the frequency of the feature.
    fn frequency(&self) -> f64 {
        match (self.n(), self.m()) {
            (0, _) => 0.,
            (n, m) => n as f64 / m as f64,
        }
    }
}

/// Any [`Observed`] entity can be used as [`Aggregated`]:
/// - present: `1/1`
/// - excluded: `0/1`
impl<T> Aggregated for T
where
    T: Observed,
{
    fn n(&self) -> u32 {
        match self.status() {
            ObservationStatus::Present => 1,
            ObservationStatus::Excluded => 0,
        }
    }

    fn m(&self) -> u32 {
        1
    }
}

/// Ratio representing a fraction of `n` of `m` items.
///
/// # Examples
///
/// Ratio can be used to represent that 2 out of 10 tested individuals met a condition:
///
/// ```rust
/// use ontolius::sim::{Ratio, Aggregated};
///
/// let ratio = Ratio::try_from((2, 10)).expect("The values are valid");
///
/// assert_eq!(ratio.n(), 2);
/// assert_eq!(ratio.m(), 10);
/// ```
///
/// The frequency of the condition can be obtained:
///
/// ```rust
/// # use ontolius::sim::{Ratio, Aggregated};
/// # let ratio = Ratio::try_from((2, 10)).expect("The values are valid");
/// let frequency = ratio.frequency();
///
/// assert_eq!(frequency, 0.2);
/// ```
///
/// A ratio representing 0 of 0 is a valid value:
///
/// ```rust
/// # use ontolius::sim::{Ratio, Aggregated};
/// let ratio = Ratio::try_from((0, 0)).expect("The values are valid");
///
/// assert_eq!(ratio.n(), 0);
/// assert_eq!(ratio.m(), 0);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ratio {
    n: u32,
    m: u32,
}

impl Aggregated for Ratio {
    fn n(&self) -> u32 {
        self.n
    }

    fn m(&self) -> u32 {
        self.m
    }
}

#[derive(Debug, Clone)]
pub struct InvalidRatioError {
    n: u32,
    m: u32,
}

impl std::fmt::Display for InvalidRatioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid ratio {}/{}", self.n, self.m)
    }
}

impl std::error::Error for InvalidRatioError {}

/// Create `Ratio` from a `(n, m)` tuple.
///
/// A tuple is valid if `n<=m`.
///
/// # Example
///
/// `Ratio` is created from a valid tuple.
///
/// ```rust
/// use ontolius::sim::Ratio;
///
/// let ratio = Ratio::try_from((1, 2));
/// assert!(ratio.is_ok());
/// ```
///
/// However, parsing of an invalid input fails:
/// ```rust
/// # use ontolius::sim::Ratio;
///
/// let ratio = Ratio::try_from((2, 1));
/// assert!(ratio.is_err());
/// ```
impl TryFrom<(u32, u32)> for Ratio {
    type Error = InvalidRatioError;

    fn try_from(value: (u32, u32)) -> Result<Self, Self::Error> {
        if value.0 <= value.1 {
            Ok(Ratio {
                n: value.0,
                m: value.1,
            })
        } else {
            Err(InvalidRatioError {
                n: value.0,
                m: value.1,
            })
        }
    }
}
