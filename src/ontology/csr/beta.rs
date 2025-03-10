use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
};

use graph_builder::{index::Idx, DirectedCsrGraph, DirectedNeighbors};

use crate::{
    base::TermId,
    ontology::{HierarchyQueries, HierarchyTraversals, HierarchyWalks, OntologyTerms},
    prelude::{Identified, MetadataAware},
};

/// An ontology backed by a term array and a CSR adjacency matrix.
pub struct CsrOntology<I, T>
where
    I: Idx,
{
    adjacency_matrix: DirectedCsrGraph<I>,
    terms: Vec<T>,
    term_id_to_idx: HashMap<TermId, I>,
    metadata: HashMap<String, String>,
}

impl<I, T> OntologyTerms<T> for CsrOntology<I, T>
where
    I: Idx,
{
    fn iter_terms<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.terms.iter()
    }

    fn term_by_id<ID>(&self, id: &ID) -> Option<&T>
    where
        ID: Identified,
    {
        self.term_id_to_idx
            .get(id.identifier())
            .and_then(|&idx| self.terms.get(Idx::index(idx)))
    }
}

impl<I, T> HierarchyTraversals<I> for CsrOntology<I, T>
where
    I: Idx + Hash,
{
    fn iter_child_idxs<'a>(&'a self, query: I) -> impl Iterator<Item = &'a I>
    where
        I: 'a,
    {
        self.adjacency_matrix.in_neighbors(query)
    }

    fn iter_descendant_idxs<'a>(&'a self, query: I) -> impl Iterator<Item = &'a I>
    where
        I: 'a,
    {
        DfsIter {
            source: |&x| self.adjacency_matrix.in_neighbors(x),
            seen: HashSet::new(),
            queue: VecDeque::from_iter(self.adjacency_matrix.in_neighbors(query)),
        }
    }

    fn iter_parent_idxs<'a>(&'a self, query: I) -> impl Iterator<Item = &'a I>
    where
        I: 'a,
    {
        self.adjacency_matrix.out_neighbors(query)
    }

    fn iter_ancestor_idxs<'a>(&'a self, query: I) -> impl Iterator<Item = &'a I>
    where
        I: 'a,
    {
        DfsIter {
            source: |&x| self.adjacency_matrix.out_neighbors(x),
            seen: HashSet::new(),
            queue: VecDeque::from_iter(self.adjacency_matrix.out_neighbors(query)),
        }
    }
}

impl<J, T> HierarchyWalks for CsrOntology<J, T>
where
    T: Identified,
    J: Idx + Hash,
{
    fn iter_parent_ids<'a, I>(&'a self, query: &I) -> impl Iterator<Item = &'a TermId>
    where
        I: Identified,
    {
        if let Some(&idx) = self.term_id_to_idx.get(query.identifier()) {
            WalkingIter::Known {
                terms: &self.terms,
                iterator: self.iter_parent_idxs(idx),
            }
        } else {
            WalkingIter::UnknownQuery
        }
    }

    fn iter_child_ids<'a, I>(&'a self, query: &I) -> impl Iterator<Item = &'a TermId>
    where
        I: Identified,
    {
        if let Some(&idx) = self.term_id_to_idx.get(query.identifier()) {
            WalkingIter::Known {
                terms: &self.terms,
                iterator: self.iter_child_idxs(idx),
            }
        } else {
            WalkingIter::UnknownQuery
        }
    }

    fn iter_ancestor_ids<'a, I>(&'a self, query: &I) -> impl Iterator<Item = &'a TermId>
    where
        I: Identified,
    {
        if let Some(&idx) = self.term_id_to_idx.get(query.identifier()) {
            WalkingIter::Known {
                terms: &self.terms,
                iterator: self.iter_ancestor_idxs(idx),
            }
        } else {
            WalkingIter::UnknownQuery
        }
    }

    fn iter_descendant_ids<'a, I>(&'a self, query: &I) -> impl Iterator<Item = &'a TermId>
    where
        I: Identified,
    {
        if let Some(&idx) = self.term_id_to_idx.get(query.identifier()) {
            WalkingIter::Known {
                terms: &self.terms,
                iterator: self.iter_descendant_idxs(idx),
            }
        } else {
            WalkingIter::UnknownQuery
        }
    }
}

impl<I, T> HierarchyQueries for CsrOntology<I, T>
where
    I: Idx + Hash,
{
    fn is_child_of<S, O>(&self, sub: &S, obj: &O) -> bool
    where
        S: Identified,
        O: Identified,
    {
        match (
            self.term_id_to_idx.get(sub.identifier()),
            self.term_id_to_idx.get(obj.identifier()),
        ) {
            (Some(sub), Some(obj)) => self.iter_child_idxs(*obj).any(|child| child == sub),
            _ => false,
        }
    }

    fn is_descendant_of<S, O>(&self, sub: &S, obj: &O) -> bool
    where
        S: Identified,
        O: Identified,
    {
        match (
            self.term_id_to_idx.get(sub.identifier()),
            self.term_id_to_idx.get(obj.identifier()),
        ) {
            (Some(sub), Some(obj)) => self.iter_ancestor_idxs(*sub).any(|anc| anc == obj),
            _ => false,
        }
    }

    fn is_parent_of<S, O>(&self, sub: &S, obj: &O) -> bool
    where
        S: Identified,
        O: Identified,
    {
        match (
            self.term_id_to_idx.get(sub.identifier()),
            self.term_id_to_idx.get(obj.identifier()),
        ) {
            (Some(sub), Some(obj)) => self.iter_parent_idxs(*obj).any(|parent| parent == sub),
            _ => false,
        }
    }

    fn is_ancestor_of<S, O>(&self, sub: &S, obj: &O) -> bool
    where
        S: Identified,
        O: Identified,
    {
        match (
            self.term_id_to_idx.get(sub.identifier()),
            self.term_id_to_idx.get(obj.identifier()),
        ) {
            (Some(sub), Some(obj)) => self.iter_ancestor_idxs(*obj).any(|anc| anc == sub),
            _ => false,
        }
    }
}

impl<I, T> MetadataAware for CsrOntology<I, T>
where
    I: Idx,
{
    fn version(&self) -> &str {
        self.metadata
            .get("version")
            .map(|a| a.as_str())
            .expect("Ontology should have a version")
    }
}

/// An iterator for traversing the source elements in a depth-first fashion.
///
/// `F`: a function for supplying elements.
/// `I`: element type.
struct DfsIter<F, T> {
    source: F,
    seen: HashSet<T>,
    queue: VecDeque<T>,
}

/// Implement iterator if `F` is a supplier of items `I` that are supplied from `F`.
///
/// An example `F` can include a function that provides e.g.
impl<F, T, I> Iterator for DfsIter<F, T>
where
    F: Fn(T) -> I,
    T: Eq + Hash + Copy,
    I: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(i) = self.queue.pop_front() {
            if self.seen.insert(i) {
                // newly inserted
                self.queue.extend((self.source)(i));
                return Some(i);
            }
        }
        None
    }
}

/// An iterator for traversing the source elements in a breadth-first fashion.
///
/// `F`: a function for supplying elements.
/// `I`: element type.
#[allow(dead_code)] // This is dead for now ...
struct BfsIter<F, T> {
    source: F,
    seen: HashSet<T>,
    stack: Vec<T>,
}

/// Implement iterator if `F` is a supplier of items `I` that are supplied from `F`.
///
/// An example `F` can include a function that provides e.g. parents nodes of an ontology graph.
impl<F, T, I> Iterator for BfsIter<F, T>
where
    F: Fn(T) -> I,
    T: Eq + Hash + Copy,
    I: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(i) = self.stack.pop() {
            if self.seen.insert(i) {
                // newly inserted
                self.stack.extend((self.source)(i));
                return Some(i);
            }
        }
        None
    }
}

/// Iterator over [`TermId`]s that correspond to parents, ancestors, children, or descendants of the
enum WalkingIter<'a, T, I> {
    UnknownQuery,
    Known { terms: &'a [T], iterator: I },
}

impl<'a, T, I, J> Iterator for WalkingIter<'a, T, I>
where
    T: Identified,
    I: Iterator<Item = &'a J>,
    J: Idx,
{
    type Item = &'a TermId;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            WalkingIter::UnknownQuery => None,
            WalkingIter::Known { terms, iterator } => match iterator.next() {
                Some(&j) => terms.get(Idx::index(j)).map(Identified::identifier),
                None => None,
            },
        }
    }
}
