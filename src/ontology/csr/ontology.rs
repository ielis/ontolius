//! A module with an example implementation of [`Ontology`].
use std::hash::Hash;
use std::{collections::HashMap, iter::once};

use graph_builder::index::Idx as CsrIdx;

use crate::base::{term::MinimalTerm, Identified, TermId};
use crate::error::OntographError;
use crate::hierarchy::HierarchyIdx;
use crate::io::OntologyData;
use crate::ontology::{HierarchyAware, MetadataAware, Ontology, TermAware, TermIdx};

use super::hierarchy::CsrOntologyHierarchy;

/// An example implementation of [`Ontology`]
/// backed by a ontology graph implemented
/// with a CSR adjacency matrix.
pub struct CsrOntology<HI, T>
where
    HI: TermIdx + HierarchyIdx + CsrIdx + Hash,
    T: MinimalTerm,
{
    terms: Box<[T]>,
    term_id_to_idx: HashMap<TermId, HI>,
    hierarchy: CsrOntologyHierarchy<HI>,
    metadata: HashMap<String, String>,
}

/// `CsrOntology` can be built from [`OntologyData`].
impl<HI, T> TryFrom<OntologyData<HI, T>> for CsrOntology<HI, T>
where
    HI: TermIdx + HierarchyIdx + CsrIdx + Hash,
    T: MinimalTerm,
{
    type Error = OntographError;

    fn try_from(value: OntologyData<HI, T>) -> Result<Self, Self::Error> {
        // TODO: I am not sure this is the most efficient way to build the ontology.
        let terms = value.terms().to_vec().into_boxed_slice();
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

        let hierarchy = CsrOntologyHierarchy::try_from(value.edges())?;
        let metadata = value.metadata().clone();
        Ok(Self {
            terms,
            term_id_to_idx,
            hierarchy,
            metadata,
        })
    }
}

impl<HI, T> HierarchyAware for CsrOntology<HI, T>
where
    HI: TermIdx + HierarchyIdx + CsrIdx + Hash,
    T: MinimalTerm,
{
    type HI = HI;
    type Hierarchy = CsrOntologyHierarchy<HI>;

    fn hierarchy(&self) -> &Self::Hierarchy {
        &self.hierarchy
    }
}

impl<HI, T> TermAware for CsrOntology<HI, T>
where
    HI: TermIdx + HierarchyIdx + CsrIdx + Hash,
    T: MinimalTerm,
{
    type TI = HI;
    type Term = T;
    type TermIter<'a> = std::slice::Iter<'a, Self::Term>
    where
        Self: 'a;

    fn iter_terms(&self) -> Self::TermIter<'_> {
        self.terms.iter()
    }

    fn idx_to_term(&self, idx: Self::TI) -> Option<&T> {
        self.terms.get(TermIdx::index(idx))
    }

    fn id_to_idx<ID>(&self, id: &ID) -> Option<Self::TI>
    where
        ID: Identified,
    {
        self.term_id_to_idx.get(id.identifier()).copied()
    }

    fn len(&self) -> usize {
        self.terms.len()
    }
}

impl<HI, T> MetadataAware for CsrOntology<HI, T>
where
    HI: TermIdx + HierarchyIdx + CsrIdx + Hash,
    T: MinimalTerm,
{
    fn version(&self) -> &str {
        self.metadata
            .get("version")
            .map(|a| a.as_str())
            .unwrap_or("Whoa, a missing version!")
    }
}

impl<HI, T> Ontology for CsrOntology<HI, T>
where
    HI: TermIdx + HierarchyIdx + CsrIdx + Hash,
    T: MinimalTerm,
{
    type Idx = HI;
    type T = T;
}

#[cfg(test)]
mod test {

    use std::{collections::HashSet, str::FromStr};

    use super::*;
    use crate::{
        base::term::simple::SimpleMinimalTerm,
        ontology::{AllTermIdsIter, State, TermIdIter},
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
