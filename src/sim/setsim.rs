use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

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
    pub fn new(hierarchy: H, tici: HashMap<TermId, f64>) -> Self {
        let tic = tici.iter().fold(0., |acc, entry| acc + entry.1);
        Self {
            hierarchy,
            tici,
            tic,
        }
    }

    pub fn max_dist(&self) -> f64 {
        self.tic
    }
}

impl<H> SetSim<H>
where
    H: HierarchyWalks,
{
    fn induce_graph<'a, F, T>(&'a self, features: F) -> HashSet<TermIdAndCic<'a>>
    where
        F: IntoIterator<Item = &'a T>,
        T: Identified + Observed + Clone + 'a,
    {
        let mut nodes = HashSet::new();

        for feature in features {
            if feature.is_present() {
                nodes.extend(
                    self.hierarchy
                        .iter_term_and_ancestor_ids(feature.identifier())
                        .map(|term_id| {
                            self.tici.get(term_id).map(|&conditional_ic| TermIdAndCic {
                                term_id,
                                conditional_ic,
                            })
                        })
                        .flatten(),
                );
            }
        }

        nodes
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

        let p = aig
            .intersection(&big)
            .fold(0., |acc, val| acc + val.conditional_ic);
        
        p
    }
}

#[derive(Debug, Clone)]
struct TermIdAndCic<'a> {
    term_id: &'a TermId,
    conditional_ic: f64,
}

impl PartialEq for TermIdAndCic<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.term_id == other.term_id
    }
}

impl Hash for TermIdAndCic<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.term_id.hash(state);
    }
}

impl Eq for TermIdAndCic<'_> {}

#[cfg(test)]
mod test_setsim {
    use std::collections::HashMap;

    use crate::{
        common::hpo::{
            test::{
                ABNORMALITY_OF_LIMBS, ABNORMALITY_OF_MUSCULOSKELETAL_SYSTEM,
                ABNORMALITY_OF_THE_NERVOUS_SYSTEM,
            },
            PHENOTYPIC_ABNORMALITY,
        },
        sim::{feature::PresentFeature, SimilarityMeasure},
        test::hpo,
        TermId,
    };

    use super::SetSim;

    #[test]
    fn compute_p_variant() {
        let hpo = hpo();

        let tici: HashMap<TermId, f64> = [
            (PHENOTYPIC_ABNORMALITY.clone(), 0.),
            (ABNORMALITY_OF_MUSCULOSKELETAL_SYSTEM.clone(), 1.),
            (ABNORMALITY_OF_LIMBS.clone(), 2.),
            (ABNORMALITY_OF_THE_NERVOUS_SYSTEM.clone(), 1.5),
        ]
        .into_iter()
        .collect();

        let setsim = SetSim::new(hpo, tici);

        let a = [
            PresentFeature::from(&ABNORMALITY_OF_LIMBS),
            PresentFeature::from(&ABNORMALITY_OF_MUSCULOSKELETAL_SYSTEM),
        ];
        let b = [
            PresentFeature::from(&ABNORMALITY_OF_LIMBS),
            PresentFeature::from(&ABNORMALITY_OF_THE_NERVOUS_SYSTEM),
        ];

        let sim = setsim.compute(&a, &b);

        approx::assert_abs_diff_eq!(sim, 2.)
    }
}
