//! A module with the ontology parts.
pub mod csr;
mod simple;

use std::hash::Hash;

use crate::base::{term::MinimalTerm, Identified, TermId};
use crate::hierarchy::{HierarchyIdx, OntologyHierarchy};
use crate::prelude::AltTermIdAware;

/// The implementors can be used to index the [`TermAware`].
pub trait TermIdx: Copy {
    // Convert the index to `usize` for indexing.
    fn index(&self) -> usize;
}

macro_rules! impl_term_idx {
    ($TYPE:ty) => {
        impl TermIdx for $TYPE {
            fn index(&self) -> usize {
                *self as usize
            }
        }
    };
}

impl_term_idx!(u8);
impl_term_idx!(u16);
impl_term_idx!(u32);
impl_term_idx!(u64);
impl_term_idx!(usize);

impl_term_idx!(i8);
impl_term_idx!(i16);
impl_term_idx!(i32);
impl_term_idx!(i64);
impl_term_idx!(isize);

/// A trait for types that act as containers of ontology terms.
///
/// The container supports iteration over the terms, retrieval of a term
/// by its index or by the primary or obsolete [`TermId`],
/// and several associated convenience methods.
///
/// `I` - Ontology node index.
/// `T` - Ontology term.
pub trait TermAware<I, T> {
    /// Get the iterator over the *primary* ontology terms.
    fn iter_terms<'a>(&'a self) -> impl Iterator<Item = &T>
    where
        T: 'a;

    /// Map index to a term `T` of the ontology.
    ///
    /// Returns `None` if there is no such term for the input `idx` in the ontology.
    fn idx_to_term(&self, idx: &I) -> Option<&T>;

    /// Get the ontology node index corresponding to the provided `id`.
    fn id_to_idx<ID>(&self, id: &ID) -> Option<&I>
    where
        ID: Identified;

    /// Get term `T` for given term ID.
    ///
    /// Returns `None` if the ID does not correspond to a concept from the ontology.
    fn id_to_term<ID>(&self, id: &ID) -> Option<&T>
    where
        ID: Identified,
    {
        self.id_to_idx(id).and_then(|idx| self.idx_to_term(idx))
    }

    /// Get the primary term ID for a given ID.
    ///
    /// Returns `None` if the term ID does not correspond to a concept from the ontology.
    fn primary_term_id<'a, ID>(&'a self, term_id: &ID) -> Option<&'a TermId>
    where
        ID: Identified,
        T: 'a + Identified,
    {
        self.id_to_term(term_id).map(|term| term.identifier())
    }

    /// Get the term ID of a term stored under given `idx`.
    fn idx_to_term_id<'a>(&'a self, idx: &I) -> Option<&'a TermId>
    where
        T: 'a + Identified,
    {
        match self.idx_to_term(idx) {
            Some(term) => Some(term.identifier()),
            None => None,
        }
    }

    /// Get the count of ontology terms.
    fn len(&self) -> usize {
        self.iter_terms().count()
    }

    /// Test if the ontology includes no terms.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over term IDs of the *primary* terms.
    fn iter_term_ids<'a>(&'a self) -> TermIdIter<'a, T>
    where
        I: 'a,
    {
        TermIdIter {
            terms: Box::new(self.iter_terms()),
        }
    }

    /// Iterate over term IDs of *all* terms (primary and obsolete).
    fn iter_all_term_ids<'a>(&'a self) -> AllTermIdsIter<'_, T>
    where
        T: AltTermIdAware,
        I: 'a,
    {
        AllTermIdsIter {
            state: State::Primary,
            terms: Box::new(self.iter_terms()),
        }
    }
}

/// Iterator over the *primary* term ids of [`TermAware`].
pub struct TermIdIter<'a, T> {
    terms: Box<dyn Iterator<Item = &'a T> + 'a>,
}

impl<'a, T> Iterator for TermIdIter<'a, T>
where
    T: Identified,
{
    type Item = &'a TermId;

    fn next(&mut self) -> Option<Self::Item> {
        self.terms.next().map(Identified::identifier)
    }
}

enum State<'a, T>
where
    T: AltTermIdAware + 'a,
{
    Primary,
    Alt(T::TermIdIter<'a>),
}

/// Iterator over *all* (primary and obsolete) term ids of [`TermAware`].
pub struct AllTermIdsIter<'a, T>
where
    T: AltTermIdAware,
{
    state: State<'a, T>,
    terms: Box<dyn Iterator<Item = &'a T> + 'a>,
}

impl<'a, T> Iterator for AllTermIdsIter<'a, T>
where
    T: MinimalTerm,
{
    type Item = &'a TermId;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.state {
            State::Primary => {
                if let Some(term) = self.terms.next() {
                    self.state = State::Alt(term.iter_alt_term_ids());
                    Some(term.identifier())
                } else {
                    self.state = State::Primary;
                    None
                }
            }
            State::Alt(alts) => {
                if let Some(t) = alts.next() {
                    Some(t)
                } else if let Some(term) = self.terms.next() {
                    self.state = State::Alt(term.iter_alt_term_ids());
                    Some(term.identifier())
                } else {
                    self.state = State::Primary;
                    None
                }
            }
        }
    }
}

/// The implementors know about the [`OntologyHierarchy`].
///
/// * `I` - ontology node indexer.
pub trait HierarchyAware<I> {
    /// The hierarchy type.
    type Hierarchy: OntologyHierarchy<I>;

    /// Get the hierarchy.
    fn hierarchy(&self) -> &Self::Hierarchy;
}

/// Trait for describing ontology metadata.
///
/// Only the version is supported at the moment but the info will likely grow
/// in future.
pub trait MetadataAware {
    /// Get the version of the ontology.
    fn version(&self) -> &str;
}

/// Requirements for the index datatype for indexing the ontology nodes.
///
/// Note, `Hash` is not necessarily used in the ontology functionality.
/// However, we require the `Hash` implementation to increase user convenience,
/// e.g. to support creating hash sets/maps of the vanilla ontology indices.
pub trait OntologyIdx: TermIdx + HierarchyIdx + Hash {}

macro_rules! impl_ontology_idx {
    ($TYPE:ty) => {
        impl OntologyIdx for $TYPE {}
    };
}

impl_ontology_idx!(u8);
impl_ontology_idx!(u16);
impl_ontology_idx!(u32);
impl_ontology_idx!(u64);
impl_ontology_idx!(usize);

impl_ontology_idx!(i8);
impl_ontology_idx!(i16);
impl_ontology_idx!(i32);
impl_ontology_idx!(i64);
impl_ontology_idx!(isize);

/// The specification of an ontology.
///
/// An ontology acts as a container of ontology terms and supports iterating over all terms/term IDs
/// and getting a term either by its index or term ID (including obsolete IDs).
/// See [`TermAware`] for more details.
///
/// An ontology also has a hierarchy - a directed acyclic graph of relationships between the terms.
/// Currently, only `is_a` relationship is supported.
/// See [`OntologyHierarchy`] for more details.
///
/// Last, an ontology includes the metadata such as its release version.
/// See [`MetadataAware`] for more details.
///
/// * `I` - The indexer for the terms and ontology graph nodes.
/// * `T` - The ontology term type.
pub trait Ontology<I, T>: TermAware<I, T> + HierarchyAware<I> + MetadataAware {
    /// Get the root term.
    fn root_term(&self) -> &T {
        self.idx_to_term(self.hierarchy().root())
            .expect("Ontology should contain a term for term index")
    }

    /// Get the term ID of the root term of the ontology.
    fn root_term_id<'a>(&'a self) -> &'a TermId
    where
        T: Identified + 'a,
    {
        self.root_term().identifier()
    }
}
