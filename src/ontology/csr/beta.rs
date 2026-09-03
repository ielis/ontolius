use anyhow::Error;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
    iter::once,
};

use graph_builder::{
    index::Idx, CsrLayout, DirectedCsrGraph, DirectedNeighbors, Graph, GraphBuilder,
};

use crate::{
    io::{GraphEdge, OntologyData, Relationship},
    ontology::{api::TaxonomyTraversal, MetadataAware, OntologyTerms, TaxonomyQuery, TaxonomyWalk},
    term::{AltTermIdAware, MinimalTerm},
    Identified, TermId,
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

impl<I, T> std::fmt::Debug for CsrOntology<I, T>
where
    I: Idx,
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CsrOntology {{ n_terms: {0:?}, adjacency_matrix: {{ n_nodes: {1:?}, n_edges: {2:?} }}, metadata: {3:?} }}",
            self.terms.len(),
            self.adjacency_matrix.node_count(),
            self.adjacency_matrix.edge_count(),
            self.metadata,
        )
    }
}

impl<I, T> TryFrom<OntologyData<I, T>> for CsrOntology<I, T>
where
    I: Idx,
    T: Identified + AltTermIdAware,
{
    type Error = Error;

    fn try_from(value: OntologyData<I, T>) -> Result<Self, Self::Error> {
        let adjacency_matrix = GraphBuilder::new()
            // No performance difference was observed for `CsrLayout::Sorted`
            // in IO and traversal benches.
            .csr_layout(CsrLayout::Unsorted)
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

impl<I, T> OntologyTerms for CsrOntology<I, T>
where
    I: Idx,
    T: MinimalTerm,
{
    type Term = T;
    fn iter_terms<'a>(&'a self) -> impl Iterator<Item = &'a Self::Term>
    where
        Self::Term: 'a,
    {
        self.terms.iter()
    }

    fn term_by_id<ID>(&self, id: &ID) -> Option<&Self::Term>
    where
        ID: Identified,
    {
        self.term_id_to_idx
            .get(id.identifier())
            .and_then(|&idx| self.terms.get(Idx::index(idx)))
    }
}

macro_rules! impl_ontology_terms {
    ($t:ty) => {
        impl<I, T> OntologyTerms for $t
        where
            I: Idx,
            T: MinimalTerm,
        {
            type Term = T;
            fn iter_terms<'a>(&'a self) -> impl Iterator<Item = &'a Self::Term>
            where
                T: 'a,
            {
                (**self).iter_terms()
            }

            fn term_by_id<ID>(&self, id: &ID) -> Option<&Self::Term>
            where
                ID: Identified,
            {
                (**self).term_by_id(id)
            }
        }
    };
}
impl_ontology_terms!(&CsrOntology<I, T>);
impl_ontology_terms!(Box<CsrOntology<I, T>>);

impl<I, T> TaxonomyTraversal for CsrOntology<I, T>
where
    I: Idx + Hash,
    T: Identified,
{
    type Idx = I;
    fn term_index<Q>(&self, query: &Q) -> Option<Self::Idx>
    where
        Q: Identified,
    {
        self.term_id_to_idx.get(query.identifier()).copied()
    }

    fn idx_to_term_id(&self, query: Self::Idx) -> Option<&TermId> {
        self.terms.get(query.index()).map(|t| t.identifier())
    }

    fn iter_child_idxs(&self, query: Self::Idx) -> impl Iterator<Item = Self::Idx> {
        self.adjacency_matrix.in_neighbors(query).copied()
    }

    fn iter_descendant_idxs(&self, query: Self::Idx) -> impl Iterator<Item = Self::Idx> {
        DfsIter {
            source: |x| self.adjacency_matrix.in_neighbors(x).copied(),
            seen: HashSet::new(),
            queue: VecDeque::from_iter(self.adjacency_matrix.in_neighbors(query).copied()),
        }
    }

    fn iter_parent_idxs(&self, query: Self::Idx) -> impl Iterator<Item = Self::Idx> {
        self.adjacency_matrix.out_neighbors(query).copied()
    }

    fn iter_ancestor_idxs(&self, query: Self::Idx) -> impl Iterator<Item = Self::Idx> {
        DfsIter {
            source: |x| self.adjacency_matrix.out_neighbors(x).copied(),
            seen: HashSet::new(),
            queue: VecDeque::from_iter(self.adjacency_matrix.out_neighbors(query).copied()),
        }
    }
}
macro_rules! impl_taxonomy_traversal {
    ($t:ty) => {
        impl<I, T> TaxonomyTraversal for $t
        where
            I: Idx + Hash,
            T: Identified,
        {
            type Idx = I;
            fn term_index<Q>(&self, query: &Q) -> Option<Self::Idx>
            where
                Q: Identified,
            {
                (**self).term_index(query)
            }

            fn idx_to_term_id(&self, query: Self::Idx) -> Option<&TermId> {
                (**self).idx_to_term_id(query)
            }

            fn iter_child_idxs(&self, query: Self::Idx) -> impl Iterator<Item = Self::Idx> {
                (**self).iter_child_idxs(query)
            }

            fn iter_descendant_idxs(&self, query: Self::Idx) -> impl Iterator<Item = Self::Idx> {
                (**self).iter_descendant_idxs(query)
            }

            fn iter_parent_idxs(&self, query: Self::Idx) -> impl Iterator<Item = Self::Idx> {
                (**self).iter_descendant_idxs(query)
            }

            fn iter_ancestor_idxs(&self, query: Self::Idx) -> impl Iterator<Item = Self::Idx> {
                (**self).iter_ancestor_idxs(query)
            }
        }
    };
}
impl_taxonomy_traversal!(&CsrOntology<I, T>);
impl_taxonomy_traversal!(Box<CsrOntology<I, T>>);

impl<I, T> TaxonomyWalk for CsrOntology<I, T>
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
                iterator: TaxonomyTraversal::iter_parent_idxs(self, idx),
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

macro_rules! impl_taxonomy_walk {
    ($t:ty) => {
        impl<I, T> TaxonomyWalk for $t
        where
            I: Idx + Hash,
            T: Identified,
        {
            fn iter_parent_ids<'a, ID>(&'a self, query: &ID) -> impl Iterator<Item = &'a TermId>
            where
                ID: Identified,
            {
                (**self).iter_parent_ids(query)
            }

            fn iter_child_ids<'a, ID>(&'a self, query: &ID) -> impl Iterator<Item = &'a TermId>
            where
                ID: Identified,
            {
                (**self).iter_child_ids(query)
            }

            fn iter_ancestor_ids<'a, ID>(&'a self, query: &ID) -> impl Iterator<Item = &'a TermId>
            where
                ID: Identified,
            {
                (**self).iter_ancestor_ids(query)
            }

            fn iter_descendant_ids<'a, ID>(&'a self, query: &ID) -> impl Iterator<Item = &'a TermId>
            where
                ID: Identified,
            {
                (**self).iter_descendant_ids(query)
            }
        }
    };
}
impl_taxonomy_walk!(&CsrOntology<I, T>);
impl_taxonomy_walk!(Box<CsrOntology<I, T>>);

impl<I, T> TaxonomyQuery for CsrOntology<I, T>
where
    I: Idx + Hash,
    T: Identified,
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

macro_rules! impl_taxonomy_query {
    ($t:ty) => {
        impl<I, T> TaxonomyQuery for $t
        where
            I: Idx + Hash,
            T: Identified,
        {
            fn is_child_of<S, O>(&self, sub: &S, obj: &O) -> bool
            where
                S: Identified,
                O: Identified,
            {
                (**self).is_child_of(sub, obj)
            }

            fn is_descendant_of<S, O>(&self, sub: &S, obj: &O) -> bool
            where
                S: Identified,
                O: Identified,
            {
                (**self).is_descendant_of(sub, obj)
            }

            fn is_parent_of<S, O>(&self, sub: &S, obj: &O) -> bool
            where
                S: Identified,
                O: Identified,
            {
                (**self).is_parent_of(sub, obj)
            }

            fn is_ancestor_of<S, O>(&self, sub: &S, obj: &O) -> bool
            where
                S: Identified,
                O: Identified,
            {
                (**self).is_ancestor_of(sub, obj)
            }
        }
    };
}
impl_taxonomy_query!(&CsrOntology<I, T>);
impl_taxonomy_query!(Box<CsrOntology<I, T>>);

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

#[cfg(test)]
mod test_csr_ontology {
    use std::{collections::HashMap, fmt::Write};

    use crate::{io::OntologyData, ontology::csr::CsrOntology, term::simple::SimpleMinimalTerm};

    fn make_ontology_data<I, T>() -> OntologyData<I, T> {
        OntologyData {
            terms: vec![],
            edges: vec![],
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_debug() {
        let toy: CsrOntology<u8, SimpleMinimalTerm> = make_ontology_data()
            .try_into()
            .expect("Parsing should not fail");

        let mut val = String::new();
        write!(&mut val, "{0:?}", toy).expect("Expecting no formatting issues");

        assert_eq!(&val, "CsrOntology { n_terms: 0, adjacency_matrix: { n_nodes: 1, n_edges: 0 }, metadata: {} }");
    }

    mod hierarchy_traversals {
        use crate::{
            common::hpo::PHENOTYPIC_ABNORMALITY,
            ontology::{TaxonomyTraversal, TaxonomyWalk},
            test::hpo,
        };

        #[test]
        fn term_id_to_idx_roundtrip() {
            let hpo = hpo();

            let root = &PHENOTYPIC_ABNORMALITY;

            for term_id in hpo.iter_term_and_child_ids(root) {
                let idx = hpo.term_index(term_id).expect("Index must be present");
                let other = hpo.idx_to_term_id(idx).expect("Term id must be present");
                assert_eq!(term_id, other);
            }
        }
    }
}
