use crate::term::{AltTermIdAware, MinimalTerm};
use crate::{Identified, TermId};

/// A container of ontology terms.
/// 
/// `T` is the ontology term type.
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

/// Trait for describing ontology metadata.
///
/// Only the version is supported at the moment but the info will likely grow
/// in future.
pub trait MetadataAware {
    /// Get the version of the ontology.
    fn version(&self) -> &str;
}
