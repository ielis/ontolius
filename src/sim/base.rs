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

/// A representation of an individual annotated to ontology features.
pub trait Individual {
    /// The ontology feature type.
    type Feature;

    /// Get the features.
    fn features(&self) -> &[Self::Feature];
}

/// Compute semantic similarity between a pair of annotated items `a` and `b`.
///
/// The computation is infallible.
pub trait SimilarityMeasure<T> {
    type Sim;

    /// Compute the semantic similarity
    fn compute(&self, a: &[T], b: &[T]) -> Self::Sim;
}

/// Represents output of application of a similarity function
/// on a collection of individuals.
///
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
enum SimilarityMatrixFormat {
    /// Upper triangle of the similarity matrix, including the diagonal.
    UpperTriangle,
    /// Full matrix in the flat format.
    Full,
}

impl SimilarityMatrixFormat {
    fn compute_capacity(&self, n: usize) -> usize {
        match self {
            SimilarityMatrixFormat::UpperTriangle => n * (n + 1) / 2,
            SimilarityMatrixFormat::Full => n * n,
        }
    }

    // Returns `None` if `row>=n` or `col>=n` (out of bounds).
    fn compute_index(&self, n: usize, row: usize, col: usize) -> Option<usize> {
        if row < n && col < n {
            match self {
                SimilarityMatrixFormat::UpperTriangle => {
                    // `row<=col` in UpperTriangle format
                    let (row, col) = if row <= col { (row, col) } else { (col, row) };
                    Some(n * row + col - (row * (row + 1)) / 2)
                }
                SimilarityMatrixFormat::Full => Some(row * n + col),
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod similarity_matrix_format {
    use crate::sim::base::SimilarityMatrixFormat;

    #[test]
    fn compute_capacity_upper_triangle() {
        let format = SimilarityMatrixFormat::UpperTriangle;

        assert_eq!(0, format.compute_capacity(0));
        assert_eq!(1, format.compute_capacity(1));
        assert_eq!(3, format.compute_capacity(2));
        assert_eq!(6, format.compute_capacity(3));
        assert_eq!(10, format.compute_capacity(4));
    }

    #[test]
    fn compute_index_upper_triangle() {
        let format = SimilarityMatrixFormat::UpperTriangle;

        assert_eq!(format.compute_index(1, 0, 0), Some(0));
        assert_eq!(format.compute_index(1, 1, 0), None);
        assert_eq!(format.compute_index(1, 0, 1), None);

        assert_eq!(format.compute_index(2, 0, 0), Some(0));
        assert_eq!(format.compute_index(2, 1, 0), Some(1));
        assert_eq!(format.compute_index(2, 1, 1), Some(2));
        assert_eq!(format.compute_index(2, 2, 1), None);

        assert_eq!(format.compute_index(3, 0, 0), Some(0));
        assert_eq!(format.compute_index(3, 1, 2), Some(4));
        assert_eq!(format.compute_index(3, 2, 1), Some(4));
        assert_eq!(format.compute_index(3, 2, 2), Some(5));
        assert_eq!(format.compute_index(3, 3, 0), None);
    }

    #[test]
    fn compute_capacity_full() {
        let format = SimilarityMatrixFormat::Full;

        assert_eq!(0, format.compute_capacity(0));
        assert_eq!(1, format.compute_capacity(1));
        assert_eq!(4, format.compute_capacity(2));
        assert_eq!(9, format.compute_capacity(3));
        assert_eq!(16, format.compute_capacity(4));
    }

    #[test]
    fn compute_index_full() {
        let format = SimilarityMatrixFormat::Full;

        assert_eq!(format.compute_index(1, 0, 0), Some(0));
        assert_eq!(format.compute_index(1, 1, 0), None);
        assert_eq!(format.compute_index(1, 0, 1), None);

        assert_eq!(format.compute_index(2, 0, 0), Some(0));
        assert_eq!(format.compute_index(2, 1, 0), Some(2));
        assert_eq!(format.compute_index(2, 1, 1), Some(3));
        assert_eq!(format.compute_index(2, 2, 1), None);

        assert_eq!(format.compute_index(3, 0, 0), Some(0));
        assert_eq!(format.compute_index(3, 1, 2), Some(5));
        assert_eq!(format.compute_index(3, 2, 1), Some(7));
        assert_eq!(format.compute_index(3, 2, 2), Some(8));
        assert_eq!(format.compute_index(3, 3, 0), None);
    }
}

/// Similarity matrix has results of calculation of semantic similarity
/// between an sequence of individuals/entities annotated to ontology terms.
///
/// `T` - the similarity datatype.
#[allow(clippy::len_without_is_empty)]
pub struct SimilarityMatrix<T> {
    format: SimilarityMatrixFormat,
    values: Vec<T>,
    n: usize,
}

impl<T> SimilarityMatrix<T> {
    /// Get the number of individuals represented in the result.
    pub fn len(&self) -> usize {
        self.n
    }
}

impl<T> SimilarityMatrix<T>
where
    T: Clone,
{
    /// Get the similarity between i-th and j-th individuals.
    pub fn get_sim(&self, i: usize, j: usize) -> Option<T> {
        if let Some(idx) = self.format.compute_index(self.n, i, j) {
            Some(Clone::clone(&self.values[idx]))
        } else {
            None
        }
    }
}

/// Applies a similarity method on a collection of individuals.
pub trait SimilarityMatrixCreator<T> {
    type Sim;
    type Error;

    /// Compute the similarity matrix for the `individuals`.
    ///
    /// The computation can fail from the input being invalid or for operational reasons.
    fn compute(&self, individuals: &[T]) -> Result<SimilarityMatrix<Self::Sim>, Self::Error>;
}

#[allow(dead_code)]
mod simple {
    use crate::sim::{base::Individual, SimilarityMeasure};

    use super::{SimilarityMatrix, SimilarityMatrixCreator};

    struct SimpleSimilarityMatrixCreator<SM> {
        measure: SM,
    }

    impl<SM, I> SimilarityMatrixCreator<I> for SimpleSimilarityMatrixCreator<SM>
    where
        SM: SimilarityMeasure<I::Feature>,
        I: Individual,
    {
        type Sim = f64;
        type Error = ();

        fn compute(&self, individuals: &[I]) -> Result<SimilarityMatrix<Self::Sim>, ()> {
            // TODO: test if the `measure` is symmetric and only compute a triangle, if yes.
            // Manage a thread pool or an executor of some kind and submit the tasks to the threads.
            // Collect the results into the output.
            let first = individuals.first().unwrap();
            let second = individuals.get(1).unwrap();

            let _sim = self.measure.compute(first.features(), second.features());

            // TODO: continue here and implement the kernel
            Err(())
        }
    }
}
