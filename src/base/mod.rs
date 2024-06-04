//! The base building blocks for working with ontology data.
pub use term_id::TermId;

pub mod term;
mod term_id;
#[cfg(feature="pyo3")]
pub(crate) mod py;


/// `Identified` is implemented by entities that have a [`TermId`] as an identifier.
///
/// ## Examples
///
/// [`crate::base::term::simple::SimpleMinimalTerm`] implements `Identified`.
/// ```
/// use ontolius::prelude::*;
/// use ontolius::base::term::simple::SimpleMinimalTerm;
///
/// let term_id = TermId::from(("HP", "1234567"));
/// let term = SimpleMinimalTerm::new(term_id, "Sample term", vec![], false);
///
/// assert_eq!(term.identifier().to_string(), "HP:1234567")
/// ```
pub trait Identified {
    fn identifier(&self) -> &TermId;
}
