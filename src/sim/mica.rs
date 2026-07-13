use std::collections::{HashMap, HashSet};

use crate::{ontology::HierarchyWalks, TermId};

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
pub struct DynamicIcAccessor<G> {
    graph: G,
    term_id2ic: HashMap<TermId, f64>,
}

impl<T> IcMicaAccessor for DynamicIcAccessor<T>
where
    T: HierarchyWalks,
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
