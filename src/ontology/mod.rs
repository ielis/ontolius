//! A module with the ontology parts.
pub mod csr;

use std::hash::Hash;

use crate::base::{term::MinimalTerm, Identified, TermId};
use crate::hierarchy::{HierarchyIdx, OntologyHierarchy};

/// The implementors can be used to index the [`super::TermAware`].
pub trait TermIdx: Copy {
    // Convert the index to `usize` for indexing.
    fn index(&self) -> usize;
}

macro_rules! impl_idx {
    ($TYPE:ty) => {
        impl TermIdx for $TYPE {
            fn index(&self) -> usize {
                *self as usize
            }
        }
    }
}

impl_idx!(u8);
impl_idx!(u16);
impl_idx!(u32);
impl_idx!(u64);
impl_idx!(usize);

impl_idx!(i8);
impl_idx!(i16);
impl_idx!(i32);
impl_idx!(i64);
impl_idx!(isize);

/// A trait for types that act as a containers of the ontology terms.
///
/// The container supports iteration over the terms, to retrieve a term
/// by its index or by the primary or obsolete [`TermId`],
/// and several associated convenience methods.
pub trait TermAware {
    type TI: TermIdx;
    type Term: MinimalTerm;
    type TermIter<'a>: Iterator<Item = &'a Self::Term>
    where
        Self: 'a;

    /// Get the iterator over the *primary* ontology terms.
    fn iter_terms(&self) -> Self::TermIter<'_>;

    /// Map index to a [`TermAware::Term`] of the ontology.
    ///
    /// Returns `None` if there is no such term for the input `idx` in the ontology.
    fn idx_to_term(&self, idx: &Self::TI) -> Option<&Self::Term>;

    /// Get the index corresponding to a [`TermAware::Term`] for given ID.
    fn id_to_idx<ID>(&self, id: &ID) -> Option<&Self::TI>
    where
        ID: Identified;

    /// Get [`TermAware::Term`] for given term ID.
    ///
    /// Returns `None`` if the ID does not correspond to a concept from the ontology.
    fn id_to_term<ID>(&self, id: &ID) -> Option<&Self::Term>
    where
        ID: Identified,
    {
        self.id_to_idx(id).and_then(|idx| self.idx_to_term(idx))
    }

    /// Get the primary term ID for a given ID.
    ///
    /// Returns `None` if the term ID does not correspond to a concept from the ontology.
    fn primary_term_id<ID>(&self, term_id: &ID) -> Option<&TermId>
    where
        ID: Identified,
    {
        self.id_to_term(term_id).map(|term| term.identifier())
    }

    /// Get the term ID of a term stored under given `idx`.
    fn idx_to_term_id(&self, idx: &Self::TI) -> Option<&TermId> {
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
    fn iter_term_ids(&self) -> TermIdIter<'_, Self::Term> {
        TermIdIter {
            terms: Box::new(self.iter_terms()),
        }
    }

    /// Iterate over term IDs of *all* terms (primary and obsolete).
    fn iter_all_term_ids(&self) -> AllTermIdsIter<'_, Self::Term> {
        AllTermIdsIter {
            state: State::Primary,
            terms: Box::new(self.iter_terms()),
        }
    }
}

/// Iterator over the *primary* term ids of [`TermAware`].
pub struct TermIdIter<'a, T>
where
    T: MinimalTerm,
{
    terms: Box<dyn Iterator<Item = &'a T> + 'a>,
}

impl<'a, T> Iterator for TermIdIter<'a, T>
where
    T: MinimalTerm,
{
    type Item = &'a TermId;

    fn next(&mut self) -> Option<Self::Item> {
        self.terms.next().map(Identified::identifier)
    }
}

enum State<'a, T>
where
    T: MinimalTerm + 'a,
{
    Primary,
    Alt(T::TermIdIter<'a>),
}

/// Iterator over *all* (primary and obsolete) term ids of [`TermAware`].
pub struct AllTermIdsIter<'a, T>
where
    T: MinimalTerm,
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
pub trait HierarchyAware {
    /// The indexer for the graph nodes.
    type HI: HierarchyIdx;
    /// The hierarchy type.
    type Hierarchy: OntologyHierarchy<HI = Self::HI>;

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

/// The specification of an ontology.
///
/// The ontology has several functionalities. First, it acts as a container
/// of ontology terms and supports iterating over all terms/term IDs
/// and getting a term either by its index or term ID (including obsolete IDs).
/// See [`TermAware`] for more details.
///
/// Next, the ontology has the hierarchy - a directed acyclyc graph of term
/// relations. Currently, only `is_a` relationship is supported.
/// See [`OntologyHierarchy`] for more details.
///
/// Last, ontology includes the metadata such as its release version.
/// See [`MetadataAware`] for more details.
///
pub trait Ontology:
    TermAware<TI = Self::Idx, Term = Self::T> + HierarchyAware<HI = Self::Idx> + MetadataAware
{
    /// The indexer for the terms and ontology graph nodes.
    /// 
    /// Note, `Hash` is not necessarily used for the ontology functionality. 
    /// However, we include the bound to increase user convenience, 
    /// e.g. to support creating hash sets/maps of the vanilla ontology indices.
    type Idx: TermIdx + HierarchyIdx + Hash;
    /// The term type.
    type T: MinimalTerm;

    /// Get the root term.
    fn root_term(&self) -> &Self::T {
        self.idx_to_term(self.hierarchy().root())
            .expect("Ontology should contain a term for term index")
    }

    /// Get the term ID of the root term of the ontology.
    fn root_term_id<'a>(&'a self) -> &'a TermId
    where
        Self::T: 'a,
    {
        self.root_term().identifier()
    }
}
