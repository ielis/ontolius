use std::collections::{HashMap, HashSet};

use crate::{
    ontology::HierarchyWalks,
    sim::{base::PresentFeature, IndividualFeature, SimilarityMeasure},
    Identified, TermId,
};

pub struct SetSim<H> {
    hierarchy: H,
    tici: HashMap<TermId, f64>,
    tic: f64,
}

impl<H> SetSim<H> {
    pub fn max_dist(&self) -> f64 {
        self.tic
    }
}

impl<H> SetSim<H>
where
    H: HierarchyWalks,
{
    fn induce_graph<'a, F, I>(&'a self, features: F) -> HashSet<&'a TermId>
    where
        F: IntoIterator<Item = &'a I>,
        I: Identified + Clone + 'a,
    {
        let mut ig = HashSet::new();
        for feature in features {
            ig.extend(
                self.hierarchy
                    .iter_term_and_ancestor_ids(feature.identifier()),
            );
        }
        ig
    }
}

impl<H> SimilarityMeasure<PresentFeature<'_>> for SetSim<H>
where
    H: HierarchyWalks,
{
    type Sim = f64;

    fn compute(&self, a: &[PresentFeature<'_>], b: &[PresentFeature<'_>]) -> Self::Sim {
        let aig = self.induce_graph(a);
        let big = self.induce_graph(b);
        aig.union(&big).map(|&t| self.tici.get(t)).flatten().sum()
    }
}

impl<H> SimilarityMeasure<IndividualFeature<'_>> for SetSim<H>
where
    H: HierarchyWalks,
{
    type Sim = f64;

    fn compute(&self, a: &[IndividualFeature<'_>], b: &[IndividualFeature<'_>]) -> Self::Sim {
        // TODO: implement the PE variant
        todo!()
    }
}
