use std::collections::{HashMap, HashSet};

use crate::{
    ontology::HierarchyWalks,
    sim::{Observed, SimilarityMeasure},
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
    fn induce_graph<'a, F, T>(&'a self, features: F) -> HashSet<&'a TermId>
    where
        F: IntoIterator<Item = &'a T>,
        T: Identified + Clone + 'a,
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

impl<H, T> SimilarityMeasure<T> for SetSim<H>
where
    H: HierarchyWalks,
    T: Identified + Observed + Clone,
{
    type Sim = f64;

    fn compute(&self, a: &[T], b: &[T]) -> Self::Sim {
        // TODO: implement the PE variant
        let aig = self.induce_graph(a);
        let big = self.induce_graph(b);
        aig.union(&big).map(|&t| self.tici.get(t)).flatten().sum()
    }
}
