//! A module with an example implementation of [`Ontology`].
use std::collections::HashSet;
use std::hash::Hash;
use std::{collections::HashMap, iter::once};

use anyhow::{bail, Result};
use graph_builder::index::Idx as CsrIdx;

use crate::hierarchy::HierarchyIdx;
use crate::io::{GraphEdge, Relationship};
use crate::io::OntologyData;
use crate::ontology::{
    HierarchyAware, HierarchyQueries, MetadataAware, Ontology, OntologyIdx, TermAware, TermIdx,
};
use crate::prelude::{AltTermIdAware, AncestorNodes, ChildNodes, ParentNodes};
use crate::{Identified, TermId};
use anyhow::Error;

use super::hierarchy::CsrOntologyHierarchy;

/// An example implementation of [`Ontology`]
/// backed by a ontology graph implemented
/// with a CSR adjacency matrix.
pub struct CsrOntology<I, T>
where
    I: CsrIdx,
{
    terms: Box<[T]>,
    term_id_to_idx: HashMap<TermId, I>,
    hierarchy: CsrOntologyHierarchy<I>,
    metadata: HashMap<String, String>,
}

/// `CsrOntology` can be built from [`OntologyData`].
impl<I, T> TryFrom<OntologyData<I, T>> for CsrOntology<I, T>
where
    I: HierarchyIdx + CsrIdx + Hash,
    T: Identified + AltTermIdAware + Default + PartialEq,
{
    type Error = Error;

    fn try_from(value: OntologyData<I, T>) -> Result<Self, Self::Error> {
        // TODO: I am not sure this is the most efficient way to build the ontology.
        let OntologyData {
            mut terms,
            mut edges,
            metadata,
        } = value;

        let candidate_roots = find_candidate_root_indices(&edges);

        let root = match candidate_roots.len() {
            0 => bail!("Ontology must have at least one candidate root term"),
            1 => **candidate_roots.first().unwrap(),
            _ => {
                let uber_root = T::default();

                let root_idx = HierarchyIdx::new(terms.len());

                if terms.iter().any(|term| term == &uber_root) {
                    bail!("The root already exists in terms");
                }

                terms.push(uber_root);

                let new_edges: Vec<_> = candidate_roots
                    .into_iter()
                    .map(|&subroot| GraphEdge::from((subroot, Relationship::Child, root_idx)))
                    .collect();
                edges.extend(new_edges);
                root_idx
            }
        };

        // Only keep the primary terms.
        let terms: Box<[_]> = terms.into_iter().collect::<Vec<_>>().into_boxed_slice();

        let term_id_to_idx = terms
            .iter()
            .enumerate()
            .flat_map(|(idx, term)| {
                once((term.identifier().clone(), HierarchyIdx::new(idx))).chain(
                    term.iter_alt_term_ids()
                        .map(move |alt| (alt.clone(), HierarchyIdx::new(idx))),
                )
            })
            .collect();

        let hierarchy = CsrOntologyHierarchy::from((root, &*edges));

        Ok(Self {
            terms,
            term_id_to_idx,
            hierarchy,
            metadata,
        })
    }
}

fn find_candidate_root_indices<I>(graph_edges: &[GraphEdge<I>]) -> Vec<&I>
where
    I: Hash + Eq,
{
    let mut root_candidate_set = HashSet::new();
    let mut remove_mark_set = HashSet::new();

    for edge in graph_edges.iter() {
        match edge.pred {
            Relationship::Child => {
                root_candidate_set.insert(&edge.obj);
                remove_mark_set.insert(&edge.sub);
            }
            Relationship::Parent => {
                root_candidate_set.insert(&edge.obj);
                remove_mark_set.insert(&edge.sub);
            }
            _ => {}
        }
    }

    root_candidate_set
        .difference(&remove_mark_set)
        .copied()
        .collect()
}

impl<I, T> HierarchyAware<I> for CsrOntology<I, T>
where
    I: HierarchyIdx + CsrIdx + Hash,
{
    type Hierarchy = CsrOntologyHierarchy<I>;

    fn hierarchy(&self) -> &Self::Hierarchy {
        &self.hierarchy
    }
}

impl<I, T> HierarchyQueries for CsrOntology<I, T>
where
    I: CsrIdx + Hash,
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
            (Some(sub_idx), Some(obj_idx)) => self
                .hierarchy
                .iter_children_of(obj_idx)
                .any(|child| child == sub_idx),
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
            (Some(sub_idx), Some(obj_idx)) => self
                .hierarchy
                .iter_ancestors_of(sub_idx)
                .any(|anc| anc == obj_idx),
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
            (Some(sub_idx), Some(obj_idx)) => self
                .hierarchy
                .iter_parents_of(obj_idx)
                .any(|child| child == sub_idx),
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
            (Some(sub_idx), Some(obj_idx)) => self
                .hierarchy
                .iter_ancestors_of(obj_idx)
                .any(|anc| anc == sub_idx),
            _ => false,
        }
    }
}

impl<I, T> TermAware<I, T> for CsrOntology<I, T>
where
    I: CsrIdx + TermIdx,
{
    fn iter_terms<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.terms.iter()
    }

    fn idx_to_term(&self, idx: &I) -> Option<&T> {
        self.terms.get(TermIdx::index(idx))
    }

    fn id_to_idx<ID>(&self, id: &ID) -> Option<&I>
    where
        ID: Identified,
    {
        self.term_id_to_idx.get(id.identifier())
    }

    fn len(&self) -> usize {
        self.terms.len()
    }

    fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }
}

impl<I, T> MetadataAware for CsrOntology<I, T>
where
    I: CsrIdx,
{
    fn version(&self) -> &str {
        self.metadata
            .get("version")
            .map(|a| a.as_str())
            .unwrap_or("Whoa, a missing version!")
    }
}

impl<I, T> Ontology<I, T> for CsrOntology<I, T> where I: OntologyIdx + CsrIdx {}

#[cfg(test)]
mod test {

    use std::{collections::HashSet, str::FromStr};

    use super::*;
    use crate::{
        ontology::{AllTermIdsIter, State, TermIdIter},
        term::simple::SimpleMinimalTerm,
    };

    #[test]
    fn test_all_term_ids_iter() {
        let terms = get_terms();
        let all_term_id_iter = AllTermIdsIter {
            terms: Box::new(terms.iter()),
            state: State::Primary,
        };

        let all_term_ids: HashSet<_> = all_term_id_iter.map(|t| t.to_string()).collect();

        let curies = [
            "HP:1", "HP:11", "HP:12", "HP:13", "HP:3", "HP:4", "HP:2", "HP:21", "HP:22", "HP:23",
        ];
        assert_eq!(all_term_ids.len(), curies.len());
        curies
            .iter()
            .for_each(|&curie| assert!(all_term_ids.contains(curie)))
    }

    #[test]
    fn test_term_ids_iter() {
        let terms = get_terms();
        let term_id_iter = TermIdIter {
            terms: Box::new(terms.iter()),
        };

        let term_ids: HashSet<_> = term_id_iter.map(|t| t.to_string()).collect();

        let curies = ["HP:1", "HP:3", "HP:4", "HP:2"];
        assert_eq!(term_ids.len(), curies.len());
        curies
            .iter()
            .for_each(|&curie| assert!(term_ids.contains(curie)))
    }

    fn get_terms() -> Vec<SimpleMinimalTerm> {
        vec![
            SimpleMinimalTerm::new(
                TermId::from_str("HP:1").unwrap(),
                "First",
                vec![
                    TermId::from_str("HP:11").unwrap(),
                    TermId::from_str("HP:12").unwrap(),
                    TermId::from_str("HP:13").unwrap(),
                ],
                false,
            ),
            SimpleMinimalTerm::new(TermId::from_str("HP:3").unwrap(), "Third", vec![], false),
            SimpleMinimalTerm::new(TermId::from_str("HP:4").unwrap(), "Fourth", vec![], false),
            SimpleMinimalTerm::new(
                TermId::from_str("HP:2").unwrap(),
                "Second",
                vec![
                    TermId::from_str("HP:21").unwrap(),
                    TermId::from_str("HP:22").unwrap(),
                    TermId::from_str("HP:23").unwrap(),
                ],
                false,
            ),
        ]
    }
}
