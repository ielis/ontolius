use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::{
    ontology::{HierarchyTraversals, HierarchyWalks},
    sim::{
        ic::{IcCalculator, IcCollector},
        Observed, SimilarityMeasure,
    },
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

#[derive(Debug, Clone)]
pub struct ConditionalIcCalculator<H, K = u32> {
    hierarchy: H,
    ic_calculator: IcCalculator<H, K>,
}

impl<H> ConditionalIcCalculator<H>
where
    H: Clone,
{
    pub fn new(hierarchy: H) -> Self {
        Self {
            hierarchy: Clone::clone(&hierarchy),
            ic_calculator: IcCalculator::new(hierarchy),
        }
    }
}

impl<H, K> ConditionalIcCalculator<H, K>
where
    H: HierarchyTraversals<K> + HierarchyWalks,
    K: Eq + Hash + Clone,
{
    pub fn submit_item<I, F>(&mut self, item: I)
    where
        I: IntoIterator<Item = F>,
        F: Identified + Observed,
    {
        self.ic_calculator.submit_item(item)
    }

    pub fn submit_items<C, I, F>(&mut self, corpus: C)
    where
        C: IntoIterator<Item = I>,
        I: IntoIterator<Item = F>,
        F: Identified + Observed,
    {
        self.ic_calculator.submit_items(corpus)
    }

    pub fn collect_cic<R, C>(&mut self, root: R, mut collector: C)
    where
        R: Identified,
        C: IcCollector,
    {
        let mut ic_collector = HashMap::new();
        self.ic_calculator
            .collect_ic(root.identifier(), &mut ic_collector);
        self.ic_calculator.clear();

        for (term_id, ic) in &ic_collector {
            let cond_ic = if term_id == root.identifier() {
                0.
            } else {
                let mut mean = 0.;
                let mut n = 0.;

                for parent_id in self.hierarchy.iter_parent_ids(term_id) {
                    if let Some(parent_ic) = ic_collector.get(parent_id) {
                        mean = n * mean + parent_ic;
                        n += 1.;
                        mean /= n;
                    }
                }

                ic - mean
            };

            collector.collect(term_id, cond_ic);
        }
    }
}

#[cfg(test)]
mod test_conditional_ic_calculator {
    use std::collections::HashMap;

    use crate::{
        common::hpo::{
            test::{ARACHNODACTYLY, CLONIC_SEIZURE, HYPERTENSION, POLYDACTYLY, SEIZURE},
            PHENOTYPIC_ABNORMALITY,
        },
        ontology::OntologyTerms,
        sim::{
            feature::{IndividualFeature, IndividualFeatureBuilder},
            setsim::ConditionalIcCalculator,
            ObservationStatus,
        },
        term::MinimalTerm,
        test::hpo,
        TermId,
    };

    #[test]
    fn test_submit_items_and_collect_cic() {
        let hpo = hpo();

        let mut calc = ConditionalIcCalculator::new(hpo);

        let items = vec![
            vec![
                make_feature(&ARACHNODACTYLY, ObservationStatus::Present),
                make_feature(&CLONIC_SEIZURE, ObservationStatus::Present),
            ],
            vec![
                make_feature(&SEIZURE, ObservationStatus::Present),
                make_feature(&HYPERTENSION, ObservationStatus::Present),
            ],
            vec![make_feature(&POLYDACTYLY, ObservationStatus::Present)],
            vec![make_feature(&SEIZURE, ObservationStatus::Present)],
        ];

        calc.submit_items(&items);

        let mut collector: HashMap<_, _> = HashMap::new();
        calc.collect_cic(&PHENOTYPIC_ABNORMALITY, &mut collector);

        for (term_id, cond_ic) in &collector {
            let name = hpo.term_by_id(term_id).map(|t| t.name()).unwrap();
            println!("{term_id} {name:<50} {cond_ic:.4}");
        }
    }

    fn make_feature<'a>(term_id: &'a TermId, status: ObservationStatus) -> IndividualFeature<'a> {
        IndividualFeatureBuilder::from(term_id)
            .with_status(status)
            .build()
    }
}
