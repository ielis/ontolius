use anyhow::Error;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
    iter::once,
};

use graph_builder::{index::Idx, CsrLayout, DirectedCsrGraph, DirectedNeighbors, GraphBuilder};

use crate::{
    base::{term::AltTermIdAware, Identified, TermId},
    hierarchy::{GraphEdge, Relationship},
    io::OntologyData,
    ontology::{
        HierarchyQueries, HierarchyTraversals, HierarchyWalks, MetadataAware, OntologyTerms,
    },
};

/// An ontology backed by a term array and a CSR adjacency matrix.
pub struct CsrOntology<I, T>
where
    I: Idx,
{
    adjacency_matrix: DirectedCsrGraph<I>,
    terms: Box<[T]>,
    term_id_to_idx: HashMap<TermId, I>,
    metadata: HashMap<String, String>,
}

impl<I, T> TryFrom<OntologyData<I, T>> for CsrOntology<I, T>
where
    I: Idx,
    T: Identified + AltTermIdAware,
{
    type Error = Error;

    fn try_from(value: OntologyData<I, T>) -> Result<Self, Self::Error> {
        let adjacency_matrix = GraphBuilder::new()
            .csr_layout(CsrLayout::Sorted)
            .edges(make_edge_iterator(value.edges))
            .build();

        let terms = value.terms.into_boxed_slice();

        let term_id_to_idx = terms
            .iter()
            .enumerate()
            .flat_map(|(idx, term)| {
                once((term.identifier().clone(), I::new(idx))).chain(
                    term.iter_alt_term_ids()
                        .map(move |alt| (alt.clone(), I::new(idx))),
                )
            })
            .collect();

        Ok(Self {
            adjacency_matrix,
            terms,
            term_id_to_idx,
            metadata: value.metadata,
        })
    }
}

fn make_edge_iterator<I>(graph_edges: Vec<GraphEdge<I>>) -> impl Iterator<Item = (I, I)> {
    graph_edges.into_iter().flat_map(|edge| {
        match edge.pred {
            // `sub -> is_a -> obj` is what we want!
            Relationship::Child => Some((edge.sub, edge.obj)),
            Relationship::Parent => Some((edge.obj, edge.sub)),
            _ => None,
        }
    })
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
    fn iter_child_idxs(&self, query: I) -> impl Iterator<Item = I>
    {
        self.adjacency_matrix.in_neighbors(query).copied()
    }

    fn iter_descendant_idxs(&self, query: I) -> impl Iterator<Item = I>
    {
        DfsIter {
            source: |x| self.adjacency_matrix.in_neighbors(x).copied(),
            seen: HashSet::new(),
            queue: VecDeque::from_iter(self.adjacency_matrix.in_neighbors(query).copied()),
        }
    }

    fn iter_parent_idxs(&self, query: I) -> impl Iterator<Item = I>
    {
        self.adjacency_matrix.out_neighbors(query).copied()
    }

    fn iter_ancestor_idxs(&self, query: I) -> impl Iterator<Item = I>
    {
        DfsIter {
            source: |x| self.adjacency_matrix.out_neighbors(x).copied(),
            seen: HashSet::new(),
            queue: VecDeque::from_iter(self.adjacency_matrix.out_neighbors(query).copied()),
        }
    }
}

impl<I, T> HierarchyWalks for CsrOntology<I, T>
where
    I: Idx + Hash,
    T: Identified,
{
    fn iter_parent_ids<'a, ID>(&'a self, query: &ID) -> impl Iterator<Item = &'a TermId>
    where
        ID: Identified,
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

    fn iter_child_ids<'a, ID>(&'a self, query: &ID) -> impl Iterator<Item = &'a TermId>
    where
        ID: Identified,
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

    fn iter_ancestor_ids<'a, ID>(&'a self, query: &ID) -> impl Iterator<Item = &'a TermId>
    where
        ID: Identified,
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

    fn iter_descendant_ids<'a, ID>(&'a self, query: &ID) -> impl Iterator<Item = &'a TermId>
    where
        ID: Identified,
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
            (Some(&sub), Some(&obj)) => self.iter_child_idxs(obj).any(|child| child == sub),
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
            (Some(&sub), Some(&obj)) => self.iter_ancestor_idxs(sub).any(|anc| anc == obj),
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
            (Some(&sub), Some(&obj)) => self.iter_parent_idxs(obj).any(|parent| parent == sub),
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
            (Some(&sub), Some(&obj)) => self.iter_ancestor_idxs(obj).any(|anc| anc == sub),
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
    I: Iterator<Item = J>,
    J: Idx,
{
    type Item = &'a TermId;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            WalkingIter::UnknownQuery => None,
            WalkingIter::Known { terms, iterator } => match iterator.next() {
                Some(j) => terms.get(Idx::index(j)).map(Identified::identifier),
                None => None,
            },
        }
    }
}
