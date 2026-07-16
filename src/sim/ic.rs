//! Functionality for computing Information Content (IC) of ontology terms.
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::{
    ontology::{HierarchyTraversals, HierarchyWalks},
    sim::Observed,
    Identified, TermId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TermIdPair {
    pair: [TermId; 2],
}

impl<'a> From<[&'a TermId; 2]> for TermIdPair {
    fn from(mut value: [&'a TermId; 2]) -> Self {
        value.sort_unstable();

        TermIdPair {
            pair: [Clone::clone(value[0]), Clone::clone(value[1])],
        }
    }
}

/// Access the information content (IC) of the most-informative common ancestor (MICA).
pub trait IcMicaAccessor {
    /// Get the $IC_{MICA}(t_a, t_b)$.
    ///
    /// Returns `0.` if $t_a$ and $t_b$ are unrelated.
    fn get_ic_mica(&self, a: &TermId, b: &TermId) -> f64;
}

impl IcMicaAccessor for std::collections::HashMap<TermIdPair, f64> {
    fn get_ic_mica(&self, a: &TermId, b: &TermId) -> f64 {
        let tp = TermIdPair::from([a, b]);
        self.get(&tp).copied().unwrap_or(0.)
    }
}

impl IcMicaAccessor for std::collections::BTreeMap<TermIdPair, f64> {
    fn get_ic_mica(&self, a: &TermId, b: &TermId) -> f64 {
        let tp = TermIdPair::from([a, b]);
        self.get(&tp).copied().unwrap_or(0.)
    }
}

/// Compute the $IC_{MICA$ by traversing the ontology graph and then
/// looking up the $IC_t$ in a `HashMap<TermId, f64>`.
pub struct DynamicIcAccessor<H> {
    graph: H,
    term_id2ic: HashMap<TermId, f64>,
}

impl<H> IcMicaAccessor for DynamicIcAccessor<H>
where
    H: HierarchyWalks,
{
    fn get_ic_mica(&self, a: &TermId, b: &TermId) -> f64 {
        let mut ic = 0f64;

        let anc_a: HashSet<_> = self.graph.iter_term_and_ancestor_ids(a).collect();
        for tid_b in self.graph.iter_term_and_ancestor_ids(b) {
            if anc_a.contains(tid_b) {
                if let Some(ib) = self.term_id2ic.get(tid_b) {
                    ic = ic.max(*ib);
                }
            }
        }

        ic
    }
}

pub fn compute_ic_mica<H, IC, TP>(h: H, root: &TermId, ic: IC, mut dest: TP)
where
    H: HierarchyWalks,
    IC: Fn(&TermId) -> Option<f64>,
    TP: FnMut(TermIdPair, f64),
{
    let terms: Vec<_> = h.iter_term_and_descendant_ids(root).collect();
    let mut anc: HashSet<_> = HashSet::new();
    for (i, &left) in terms.iter().enumerate() {
        anc.extend(h.iter_term_and_ancestor_ids(left));
        for &right in &terms[i..] {
            if let Some(ic_mica) = h
                .iter_term_and_ancestor_ids(right)
                .filter(|&t| anc.contains(t))
                .map(|t| ic(t).filter(|&f| f > 0.))
                .flatten()
                .reduce(f64::max)
            {
                dest(TermIdPair::from([left, right]), ic_mica)
            }
        }
        anc.clear();
    }
}

/// Collects the information content (IC) values computed in [`IcCalculator`].
pub trait IcCollector {
    fn collect(&mut self, term_id: &TermId, ic: f64);
}

impl<'a> IcCollector for &'a mut std::collections::HashMap<TermId, f64> {
    fn collect(&mut self, term_id: &TermId, ic: f64) {
        self.insert(Clone::clone(term_id), ic);
    }
}

/// Compute the information content (IC) of ontology terms from a corpus of annotated items.
///
/// The calculator computes the IC by accepting an annotated item
/// via [`IcCalculator::submit_item`] method
/// or a corpus of items via the [`IcCalculator::submit_items`] method.
/// The calculator keeps track of the observed ontology terms.
///
/// After submitting all corpora, the IC can be collected from [`IcCalculator::collect_ic`].
/// Note, the collection does not reset calculator's counters and the [`IcCalculator::clear`]
/// must be called to prepare for the next corpora.
#[derive(Debug, Clone)]
pub struct IcCalculator<H, K = u32> {
    hierarchy: H,
    counter: HashMap<K, u64>,
}

impl<H> IcCalculator<H> {
    /// Create a new calculator.
    pub fn new(hierarchy: H) -> Self {
        Self {
            hierarchy,
            counter: HashMap::new(),
        }
    }
}

impl<H, K> IcCalculator<H, K> {
    /// Clear the calculator's state to prepare for another corpora.
    pub fn clear(&mut self) {
        self.counter.clear();
    }
}

impl<H, K> IcCalculator<H, K>
where
    H: HierarchyTraversals<K>,
    K: Eq + Hash + Clone,
{
    /// Submit an annotated item.
    pub fn submit_item<I, F>(&mut self, item: I)
    where
        I: IntoIterator<Item = F>,
        F: Identified + Observed,
    {
        let mut ig = HashSet::new();
        self.account_an_item(&mut ig, item);
    }

    /// Submit a corpus of annotated items.
    pub fn submit_items<C, I, F>(&mut self, corpus: C)
    where
        C: IntoIterator<Item = I>,
        I: IntoIterator<Item = F>,
        F: Identified + Observed,
    {
        let mut ig = HashSet::new();
        for item in corpus {
            self.account_an_item(&mut ig, item);
        }
    }

    fn account_an_item<I, F>(&mut self, ig: &mut HashSet<K>, item: I)
    where
        I: IntoIterator<Item = F>,
        F: Identified + Observed,
    {
        for feature in item {
            if feature.status().is_present() {
                if let Some(term_index) = self.hierarchy.term_index(feature.identifier()) {
                    ig.insert(Clone::clone(&term_index));

                    for anc_idx in self.hierarchy.iter_ancestor_idxs(term_index) {
                        ig.insert(Clone::clone(&anc_idx));
                    }
                }
            }
        }

        ig.drain()
            .for_each(|i| *self.counter.entry(i).or_default() += 1);
    }

    /// Collect the IC of the `root` and its descendants into the `collector`.
    pub fn collect_ic<R, C>(&self, root: R, mut collector: C)
    where
        R: Identified,
        C: IcCollector,
    {
        if let Some(root_idx) = self.hierarchy.term_index(root.identifier()) {
            if let Some(pop_cnt) = self.counter.get(&root_idx) {
                if let Some(root_term_id) = self.hierarchy.idx_to_term_id(Clone::clone(&root_idx)) {
                    collector.collect(root_term_id, 0.);

                    for desc_idx in self.hierarchy.iter_descendant_idxs(root_idx) {
                        if let Some(cnt) = self.counter.get(&desc_idx) {
                            if let Some(term_id) = self.hierarchy.idx_to_term_id(desc_idx) {
                                let ic = f64::log2((*pop_cnt as f64) / (*cnt as f64));
                                collector.collect(term_id, ic)
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test_ic_calculator {
    use std::collections::HashMap;

    use approx::assert_abs_diff_eq;

    use super::IcCalculator;
    use crate::{
        common::hpo::{
            test::{ARACHNODACTYLY, CLONIC_SEIZURE, HYPERTENSION, POLYDACTYLY, SEIZURE},
            PHENOTYPIC_ABNORMALITY,
        },
        sim::{
            feature::{IndividualFeature, IndividualFeatureBuilder},
            ObservationStatus,
        },
        test::hpo,
        TermId,
    };

    fn make_feature<'a>(term_id: &'a TermId, status: ObservationStatus) -> IndividualFeature<'a> {
        IndividualFeatureBuilder::from(term_id)
            .with_status(status)
            .build()
    }

    #[test]
    fn no_ic_is_collected_when_no_items_are_submitted() {
        let hpo = hpo();
        let c = IcCalculator::new(hpo);

        let mut collector: HashMap<_, _> = HashMap::new();
        c.collect_ic(&PHENOTYPIC_ABNORMALITY, &mut collector);

        assert!(collector.is_empty())
    }

    #[test]
    fn submit_items_sequentially() {
        let hpo = hpo();
        let mut calc = IcCalculator::new(hpo);

        calc.submit_item(&[
            make_feature(&ARACHNODACTYLY, ObservationStatus::Present),
            make_feature(&CLONIC_SEIZURE, ObservationStatus::Present),
        ]);
        calc.submit_item(&[
            make_feature(&SEIZURE, ObservationStatus::Present),
            make_feature(&HYPERTENSION, ObservationStatus::Present),
        ]);
        calc.submit_item(&[make_feature(&POLYDACTYLY, ObservationStatus::Present)]);
        calc.submit_item(&[make_feature(&SEIZURE, ObservationStatus::Present)]);

        let root = &PHENOTYPIC_ABNORMALITY;
        let mut collector: HashMap<_, _> = HashMap::new();
        calc.collect_ic(root, &mut collector);

        check_collector(&collector);
    }

    #[test]
    fn submit_items_in_bulk() {
        let hpo = hpo();
        let mut calc = IcCalculator::new(hpo);

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

        let root = &PHENOTYPIC_ABNORMALITY;
        let mut collector: HashMap<_, _> = HashMap::new();
        calc.collect_ic(root, &mut collector);

        check_collector(&collector);
    }

    fn check_collector(collector: &HashMap<TermId, f64>) {
        assert_eq!(collector.get(&PHENOTYPIC_ABNORMALITY), Some(&0.));
        assert_eq!(collector.get(&ARACHNODACTYLY), Some(&2.));
        assert_abs_diff_eq!(collector.get(&SEIZURE).unwrap(), &0.415_037, epsilon = 5e-5);
        assert_eq!(collector.get(&CLONIC_SEIZURE), Some(&2.));
        assert_eq!(collector.get(&POLYDACTYLY), Some(&2.));
    }
}

#[cfg(test)]
mod test_compute_ic_mica {
    use std::collections::HashMap;

    use crate::{
        common::hpo::{
            test::{ARACHNODACTYLY, CLONIC_SEIZURE, HYPERTENSION, POLYDACTYLY, SEIZURE},
            PHENOTYPIC_ABNORMALITY,
        },
        sim::{
            feature::{IndividualFeature, IndividualFeatureBuilder},
            ic::{compute_ic_mica, IcCalculator},
            ObservationStatus,
        },
        test::hpo,
        TermId,
    };

    fn make_feature<'a>(term_id: &'a TermId, status: ObservationStatus) -> IndividualFeature<'a> {
        IndividualFeatureBuilder::from(term_id)
            .with_status(status)
            .build()
    }

    #[test]
    #[ignore = "ran manually"]
    fn compute_ic_mica_naive() {
        let hpo = hpo();

        let mut ic_calculator = IcCalculator::new(hpo);

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
        ic_calculator.submit_items(&items);

        let mut ic: HashMap<TermId, f64> = HashMap::new();
        ic_calculator.collect_ic(&PHENOTYPIC_ABNORMALITY, &mut ic);

        let mut dest = HashMap::new();

        compute_ic_mica(
            hpo,
            &PHENOTYPIC_ABNORMALITY,
            |t| ic.get(t).copied(),
            |tp, ic| {
                dest.insert(tp, ic);
            },
        );

        println!("Computed for {} term pairs", dest.len());
        // Takes around 134 seconds on the release build.
    }
}
