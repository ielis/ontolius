use anyhow::Error;
use graph_builder::{
    index::Idx, CsrLayout, DirectedCsrGraph, DirectedNeighbors, Graph, GraphBuilder,
};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    hash::Hash,
    iter::once,
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
    idx_to_ancs: BTreeMap<I, HashSet<I>>,
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
    I: Idx + Clone + Hash,
    T: Identified + AltTermIdAware,
{
    type Error = Error;

    fn try_from(value: OntologyData<I, T>) -> Result<Self, Self::Error> {
        let adjacency_matrix: DirectedCsrGraph<_> = GraphBuilder::new()
            // No performance difference was observed for `CsrLayout::Sorted`
            // in IO and traversal benches.
            .csr_layout(CsrLayout::Unsorted)
            .edges(make_edge_iterator(&value.edges))
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

        let mut idx_to_ancs = BTreeMap::new();
        for (sub, _obj) in make_edge_iterator(&value.edges) {
            // Only visit each subject once.
            let _entry = idx_to_ancs.entry(sub).or_insert({
                let iter = DfsIter {
                    source: |x| adjacency_matrix.out_neighbors(x).copied(),
                    seen: BTreeSet::new(),
                    queue: VecDeque::from_iter(adjacency_matrix.out_neighbors(sub).copied()),
                };
                iter.collect()
            });
        }

        Ok(Self {
            adjacency_matrix,
            terms,
            term_id_to_idx,
            idx_to_ancs,
            metadata: value.metadata,
        })
    }
}

fn make_edge_iterator<'a, T, I>(graph_edges: T) -> impl Iterator<Item = (I, I)> + use<'a, T, I>
where
    T: IntoIterator<Item = &'a GraphEdge<I>>,
    I: Clone + 'a,
{
    graph_edges.into_iter().flat_map(|edge| {
        match edge.pred {
            // `sub -> is_a -> obj` is what we want!
            Relationship::Child => Some((Clone::clone(&edge.sub), Clone::clone(&edge.obj))),
            Relationship::Parent => Some((Clone::clone(&edge.obj), Clone::clone(&edge.sub))),
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

impl<I, T> TaxonomyTraversal for CsrOntology<I, T>
where
    I: Idx,
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
            seen: BTreeSet::new(),
            queue: VecDeque::from_iter(self.adjacency_matrix.in_neighbors(query).copied()),
        }
    }

    fn iter_parent_idxs(&self, query: Self::Idx) -> impl Iterator<Item = Self::Idx> {
        self.adjacency_matrix.out_neighbors(query).copied()
    }

    fn iter_ancestor_idxs(&self, query: Self::Idx) -> impl Iterator<Item = Self::Idx> {
        OptionalIter {
            inner: self.idx_to_ancs.get(&query).map(|bm| bm.iter().cloned()),
        }
    }
}

impl<I, T> TaxonomyWalk for CsrOntology<I, T>
where
    I: Idx,
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
            (Some(sub), Some(obj)) => self
                .idx_to_ancs
                .get(sub)
                .map(|ancs| ancs.contains(obj))
                .unwrap_or(false),
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
            (Some(sub), Some(obj)) => self
                .idx_to_ancs
                .get(obj)
                .map(|ancs| ancs.contains(sub))
                .unwrap_or(false),
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

/// Iterates if the inner is `Some`, otherwise no iteration
/// happens.
struct OptionalIter<T> {
    inner: Option<T>,
}

impl<T, I> Iterator for OptionalIter<T>
where
    T: Iterator<Item = I>,
{
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.as_mut() {
            Some(i) => i.next(),
            None => None,
        }
    }
}

/// An iterator for traversing the source elements in a depth-first fashion.
///
/// `F`: a function for supplying elements.
/// `I`: element type.
struct DfsIter<F, T> {
    source: F,
    seen: BTreeSet<T>,
    queue: VecDeque<T>,
}

/// Implement iterator if `F` is a supplier of items `I` that are supplied from `F`.
///
/// For instance, `F` can be a function that gets the parents or children of a term `I`.
impl<F, T, I> Iterator for DfsIter<F, T>
where
    F: Fn(T) -> I,
    T: Ord + Copy,
    I: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(t) = self.queue.pop_front() {
            if self.seen.insert(t) {
                // newly inserted
                self.queue.extend((self.source)(t));
                return Some(t);
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

    mod taxonomy_traversals {
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

    mod taxonomy_query {
        use super::super::TaxonomyQuery;
        use crate::{test::hpo, TermId};

        #[test]
        fn test_is_child_of() {
            let hpo = hpo();

            let arachnodactyly: TermId = "HP:0001166".parse().unwrap();
            let long_fingers: TermId = "HP:0100807".parse().unwrap();
            let abn_finger_morph: TermId = "HP:0001167".parse().unwrap();

            assert!(hpo.is_child_of(&arachnodactyly, &long_fingers));
            assert!(!hpo.is_child_of(&arachnodactyly, &abn_finger_morph));
        }

        #[test]
        fn test_is_descendant_of() {
            let hpo = hpo();
            let arachnodactyly: TermId = "HP:0001166".parse().unwrap();
            let long_fingers: TermId = "HP:0100807".parse().unwrap();
            let abn_finger_morph: TermId = "HP:0001167".parse().unwrap();

            assert!(hpo.is_descendant_of(&arachnodactyly, &long_fingers));
            assert!(hpo.is_descendant_of(&arachnodactyly, &abn_finger_morph));
        }

        #[test]
        fn test_is_parent_of() {
            let hpo = hpo();

            let arachnodactyly: TermId = "HP:0001166".parse().unwrap();
            let long_fingers: TermId = "HP:0100807".parse().unwrap();
            let abn_finger_morph: TermId = "HP:0001167".parse().unwrap();

            assert!(hpo.is_parent_of(&long_fingers, &arachnodactyly));
            assert!(!hpo.is_parent_of(&abn_finger_morph, &arachnodactyly));
        }

        #[test]
        fn test_is_ancestor_of() {
            let hpo = hpo();

            let arachnodactyly: TermId = "HP:0001166".parse().unwrap();
            let long_fingers: TermId = "HP:0100807".parse().unwrap();
            let abn_finger_morph: TermId = "HP:0001167".parse().unwrap();

            assert!(hpo.is_ancestor_of(&long_fingers, &arachnodactyly));
            assert!(hpo.is_ancestor_of(&abn_finger_morph, &arachnodactyly));
        }
    }
}
