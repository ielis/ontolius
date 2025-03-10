use std::{collections::HashMap, iter::once};

use crate::{
    base::TermId,
    hierarchy::GraphEdge,
    io::OntologyData,
    prelude::{HierarchyIdx, Identified, MinimalTerm, OntologyHierarchy},
};

use super::{HierarchyAware, MetadataAware, Ontology, TermAware, TermIdx};

#[deprecated(note = "Do not use")]
pub struct SimpleOntology<I, H, T> {
    terms: Vec<T>,
    term_id_to_idx: HashMap<TermId, I>,
    hierarchy: H,
    metadata: HashMap<String, String>,
}

impl<I, H, T> TryFrom<OntologyData<I, T>> for SimpleOntology<I, H, T>
where
    I: HierarchyIdx,
    H: TryFrom<Vec<GraphEdge<I>>, Error = anyhow::Error> + OntologyHierarchy<I>,
    T: MinimalTerm,
{
    type Error = anyhow::Error;

    fn try_from(value: OntologyData<I, T>) -> Result<Self, Self::Error> {
        // Keep primary terms only
        let terms: Vec<_> = value
            .terms
            .into_iter()
            .filter(MinimalTerm::is_current)
            .collect();

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

        let hierarchy = H::try_from(value.edges)?;
        let metadata = value.metadata;

        Ok(Self {
            terms,
            term_id_to_idx,
            hierarchy,
            metadata,
        })
    }
}

impl<I, H, T> TermAware<I, T> for SimpleOntology<I, H, T>
where
    I: TermIdx,
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
}

impl<I, H, T> HierarchyAware<I> for SimpleOntology<I, H, T>
where
    H: OntologyHierarchy<I>,
{
    type Hierarchy = H;

    fn hierarchy(&self) -> &Self::Hierarchy {
        &self.hierarchy
    }
}

impl<I, H, T> MetadataAware for SimpleOntology<I, H, T> {
    fn version(&self) -> &str {
        self.metadata
            .get("version")
            .map(|a| a.as_str())
            .unwrap_or("Whoa, a missing version!")
    }
}

impl<I, H, T> Ontology<I, T> for SimpleOntology<I, H, T>
where
    I: TermIdx,
    H: OntologyHierarchy<I>,
{
    fn root_term(&self) -> &T {
        self.idx_to_term(self.hierarchy().root())
            .expect("Ontology should contain a term for term index")
    }

    fn root_term_id<'a>(&'a self) -> &'a TermId
    where
        T: Identified + 'a,
    {
        self.root_term().identifier()
    }
}
