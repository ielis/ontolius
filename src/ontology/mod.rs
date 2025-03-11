//! A module with the ontology parts.
#[cfg(feature = "csr")]
pub mod csr;

use std::hash::Hash;

use crate::{
    Identified, TermId,
};
use crate::hierarchy::{HierarchyIdx, OntologyHierarchy};
use crate::term::{AltTermIdAware, MinimalTerm};

/// The implementors can be used to index the [`TermAware`].
#[deprecated(since = "0.5.0")]
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

/// A trait for types that act as containers of ontology terms.
///
/// The container supports iteration over the terms, retrieval of a term
/// by its index or by the primary or obsolete [`TermId`],
/// and several associated convenience methods.
///
/// `I` - Ontology node index.
/// `T` - Ontology term.
#[deprecated(since = "0.5.0", note = "Use `crate::ontology::OntologyTerms` instead")]
pub trait TermAware<I, T> {
    /// Get the iterator over the *primary* ontology terms.
    fn iter_terms<'a>(&'a self) -> impl Iterator<Item = &'a T>
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
    fn iter_all_term_ids<'a>(&'a self) -> AllTermIdsIter<'a, T>
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

/// A container of ontology terms.
pub trait OntologyTerms<T> {
    /// Get the iterator over the *primary* ontology terms.
    fn iter_terms<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a;

    /// Get term `T` for given term ID.
    ///
    /// Returns `None` if the ID does not correspond to a concept from the ontology.
    fn term_by_id<ID>(&self, id: &ID) -> Option<&T>
    where
        ID: Identified;

    /// Get the primary term ID for a given ID.
    ///
    /// Returns `None` if the term ID does not correspond to a concept from the ontology.
    fn primary_term_id<'a, ID>(&'a self, term_id: &ID) -> Option<&'a TermId>
    where
        ID: Identified,
        T: 'a + Identified,
    {
        self.term_by_id(term_id).map(|term| term.identifier())
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
    fn iter_term_ids(&self) -> TermIdIter<'_, T> {
        TermIdIter {
            terms: Box::new(self.iter_terms()),
        }
    }

    /// Iterate over term IDs of *all* terms (primary and obsolete).
    fn iter_all_term_ids(&self) -> AllTermIdsIter<'_, T>
    where
        T: AltTermIdAware,
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

/// Traversals in the ontology index space.
pub trait HierarchyTraversals<I> {
    /// Get the index of the `query` term or `None` if the term is unknown.
    fn term_index<Q>(&self, query: &Q) -> Option<I>
    where
        Q: Identified;

    /// Get an iterator with all children nodes of the `query`.
    fn iter_child_idxs(&self, query: I) -> impl Iterator<Item = I>;

    /// Get an iterator of all descendant nodes of the `query`.
    fn iter_descendant_idxs(&self, query: I) -> impl Iterator<Item = I>;

    /// Get an iterator with all parent nodes of the `query`.
    fn iter_parent_idxs(&self, query: I) -> impl Iterator<Item = I>;

    /// Get an iterator with all ancestor nodes of the `query`.
    fn iter_ancestor_idxs(&self, query: I) -> impl Iterator<Item = I>;

    // TODO: maybe a bit more convenience?
}

/// Ontology hierarchy walks provide iterators over parent, children,
/// ancestor, or descendant term ids of the `query` node.
///
/// The child-parent relationship is established solely via the `is_a` relationship.
pub trait HierarchyWalks {
    /// Returns an iterator of all nodes which are parents of `query`.
    fn iter_parent_ids<'a, I>(&'a self, query: &I) -> impl Iterator<Item = &'a TermId>
    where
        I: Identified;

    /// Returns an iterator of all nodes which are children of `query`.
    fn iter_child_ids<'a, I>(&'a self, query: &I) -> impl Iterator<Item = &'a TermId>
    where
        I: Identified;

    /// Returns an iterator of all nodes which are ancestors of `query`.
    fn iter_ancestor_ids<'a, I>(&'a self, query: &I) -> impl Iterator<Item = &'a TermId>
    where
        I: Identified;

    /// Returns an iterator of all nodes which are descendants of `query`.
    fn iter_descendant_ids<'a, I>(&'a self, query: &I) -> impl Iterator<Item = &'a TermId>
    where
        I: Identified;
}

/// Tests if an ontology term is a parent, a child, an ancestor, or descendant of another term.
pub trait HierarchyQueries {
    /// Test if `sub` is child of `obj`.
    ///
    /// Returns `false` if `sub` corresponds to the same ontology node as `obj`.
    fn is_child_of<S, O>(&self, sub: &S, obj: &O) -> bool
    where
        S: Identified,
        O: Identified;

    /// Test if `sub` is either equal to `obj` or its child.
    fn is_equal_or_child_of<S, O>(&self, sub: &S, obj: &O) -> bool
    where
        S: Identified,
        O: Identified,
    {
        sub.identifier() == obj.identifier() || self.is_child_of(sub, obj)
    }

    /// Test if `sub` is descendant of `obj`.
    ///
    /// Returns `false` if `sub` corresponds to the same ontology node as `obj`.
    fn is_descendant_of<S, O>(&self, sub: &S, obj: &O) -> bool
    where
        S: Identified,
        O: Identified;

    /// Test if `sub` is either equal to `obj` or its descendant.
    fn is_equal_or_descendant_of<S, O>(&self, sub: &S, obj: &O) -> bool
    where
        S: Identified,
        O: Identified,
    {
        sub.identifier() == obj.identifier() || self.is_descendant_of(sub, obj)
    }

    /// Test if `sub` is parent of `obj`.
    ///
    /// Returns `false` if `sub` corresponds to the same ontology node as `obj`.
    fn is_parent_of<S, O>(&self, sub: &S, obj: &O) -> bool
    where
        S: Identified,
        O: Identified;

    /// Test if `sub` is either equal to `obj` or its parent.
    fn is_equal_or_parent_of<S, O>(&self, sub: &S, obj: &O) -> bool
    where
        S: Identified,
        O: Identified,
    {
        sub.identifier() == obj.identifier() || self.is_parent_of(sub, obj)
    }

    /// Test if `sub` is ancestor of `obj`.
    ///
    /// Returns `false` if `sub` corresponds to the same ontology node as `obj`.
    fn is_ancestor_of<S, O>(&self, sub: &S, obj: &O) -> bool
    where
        S: Identified,
        O: Identified;

    /// Test if `sub` is either equal to `obj` or its parent.
    fn is_equal_or_ancestor_of<S, O>(&self, sub: &S, obj: &O) -> bool
    where
        S: Identified,
        O: Identified,
    {
        sub.identifier() == obj.identifier() || self.is_ancestor_of(sub, obj)
    }
}

/// The implementors know about the [`OntologyHierarchy`].
///
/// * `I` - ontology node indexer.
#[deprecated(
    since = "0.5.0",
    note = "Use `crate::ontology::HierarchyQueries` or `crate::ontology::HierarchyWalks` instead"
)]
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
#[deprecated(since = "0.5.0")]
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
#[deprecated(
    since = "0.5.0",
    note = "It is unclear what shared functionality should be covered by the Ontology trait"
)]
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
