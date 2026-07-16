//! Functionality for computing Information Content (IC) of ontology terms.
use std::hash::Hash;

use crate::{
    ontology::{HierarchyTraversals, HierarchyWalks},
    sim::Observed,
    Identified, TermId,
};

/// A representation of an ordered [`TermId`] pair.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TermIdPair {
    pair: [TermId; 2],
}

impl<'a> From<[&'a TermId; 2]> for TermIdPair {
    fn from(value: [&'a TermId; 2]) -> Self {
        TermIdPair::from([Clone::clone(value[0]), Clone::clone(value[1])])
    }
}

impl<'a> From<&'a TermIdPair> for [&'a TermId; 2] {
    fn from(value: &'a TermIdPair) -> Self {
        [&value.pair[0], &value.pair[1]]
    }
}

impl From<[TermId; 2]> for TermIdPair {
    fn from(mut pair: [TermId; 2]) -> Self {
        pair.sort_unstable();
        TermIdPair { pair }
    }
}

impl From<TermIdPair> for [TermId; 2] {
    fn from(value: TermIdPair) -> Self {
        value.pair
    }
}

/// Access the information content (IC) of the most-informative common ancestor (MICA).
pub trait IcMicaAccessor {
    /// Get the information content of the most-informative common ancestor
    /// of `a` and `b`.
    ///
    /// Returns `0.` if `t_a` and `t_b` are unrelated.
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

/// Compute the information content of the most-informative common ancestor (IC MICA)
/// by finding the MICA through ontology graph traversal followed by
/// retrieval of the IC MICA from a Hash map.
pub struct DynamicIcMicaAccessor<H> {
    graph: H,
    term_id2ic: std::collections::HashMap<TermId, f64>,
}

impl<H> IcMicaAccessor for DynamicIcMicaAccessor<H>
where
    H: HierarchyWalks,
{
    fn get_ic_mica(&self, a: &TermId, b: &TermId) -> f64 {
        let mut ic = 0f64;

        let anc_a: std::collections::HashSet<_> =
            self.graph.iter_term_and_ancestor_ids(a).collect();
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

/// Collects the IC MICA values from methods that can calculate them.
pub trait IcMicaCollector {
    fn collect(&mut self, pair: TermIdPair, ic_mica: f64);

    fn collect_all<I>(&mut self, values: I)
    where
        I: IntoIterator<Item = (TermIdPair, f64)>,
    {
        values
            .into_iter()
            .for_each(|(pair, ic_mica)| self.collect(pair, ic_mica));
    }
}

impl IcMicaCollector for std::collections::HashMap<TermIdPair, f64> {
    fn collect(&mut self, pair: TermIdPair, ic_mica: f64) {
        self.insert(pair, ic_mica);
    }

    fn collect_all<I>(&mut self, values: I)
    where
        I: IntoIterator<Item = (TermIdPair, f64)>,
    {
        self.extend(values);
    }
}

impl IcMicaCollector for std::collections::BTreeMap<TermIdPair, f64> {
    fn collect(&mut self, pair: TermIdPair, ic_mica: f64) {
        self.insert(pair, ic_mica);
    }

    fn collect_all<I>(&mut self, values: I)
    where
        I: IntoIterator<Item = (TermIdPair, f64)>,
    {
        self.extend(values);
    }
}

// The map type used inside of `IcMicaContainer`.
// Production
#[cfg(not(test))]
type MapType<K, V> = std::collections::HashMap<K, V>;
#[cfg(not(test))]
type MapIter<K, V> = std::collections::hash_map::IntoIter<K, V>;

// Testing - for determinism.
#[cfg(test)]
type MapType<K, V> = std::collections::BTreeMap<K, V>;
#[cfg(test)]
type MapIter<K, V> = std::collections::btree_map::IntoIter<K, V>;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(test, derive(PartialEq))]
pub struct IcMicaContainer {
    version: String, // Ontology version.
    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "IcMicaContainer::serialize_pairs",
            deserialize_with = "IcMicaContainer::deserialize_pairs"
        )
    )]
    values: MapType<TermIdPair, f64>,
}

impl IcMicaContainer {
    pub fn new(version: impl ToString) -> Self {
        Self {
            version: version.to_string(),
            values: MapType::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl IntoIterator for IcMicaContainer {
    type Item = (TermIdPair, f64);
    type IntoIter = MapIter<TermIdPair, f64>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl IcMicaAccessor for IcMicaContainer {
    fn get_ic_mica(&self, a: &TermId, b: &TermId) -> f64 {
        self.values.get_ic_mica(a, b)
    }
}

impl IcMicaCollector for IcMicaContainer {
    fn collect(&mut self, pair: TermIdPair, ic_mica: f64) {
        self.values.collect(pair, ic_mica);
    }

    fn collect_all<I>(&mut self, values: I)
    where
        I: IntoIterator<Item = (TermIdPair, f64)>,
    {
        self.values.collect_all(values);
    }
}

#[cfg(feature = "serde")]
mod ic_mica_container_serde {
    use std::fmt::Write;

    use serde::ser::SerializeSeq;

    use crate::{
        sim::ic::{IcMicaContainer, TermIdPair},
        TermId, TermIdParseError,
    };

    use super::MapType;

    impl IcMicaContainer {
        pub fn serialize_pairs<'a, S, T>(pairs: T, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
            T: IntoIterator<Item = (&'a TermIdPair, &'a f64)>,
        {
            let mut seq = serializer.serialize_seq(None)?;
            let mut buf = String::new();
            for (pair, ic_mica) in pairs {
                let [a, b] = pair.into();
                write!(&mut buf, "{}", a)
                    .map_err(|_e| serde::ser::Error::custom("cannot serialize term"))?;
                seq.serialize_element(buf.as_str())?;
                buf.clear();

                write!(&mut buf, "{}", b)
                    .map_err(|_e| serde::ser::Error::custom("cannot serialize term"))?;
                seq.serialize_element(buf.as_str())?;
                buf.clear();

                seq.serialize_element(ic_mica)?;
            }
            seq.end()
        }

        pub fn deserialize_pairs<'de, D>(
            deserializer: D,
        ) -> Result<MapType<TermIdPair, f64>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct TermPairVisitor;

            impl<'de> serde::de::Visitor<'de> for TermPairVisitor {
                type Value = MapType<TermIdPair, f64>;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(
                        formatter,
                        "a sequence of n triples where each triple contains the term pair and its IC MICA value (e.g. [\"HP:0001250\", \"HP:0001188\", 1.234, ...])"
                    )
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let mut values = MapType::new();
                    loop {
                        let a = if let Some(curie) =
                            seq.next_element::<std::borrow::Cow<'de, str>>()?
                        {
                            curie.parse::<TermId>().map_err(|e| match e {
                                TermIdParseError::MissingDelimiter => {
                                    serde::de::Error::invalid_value(
                                        serde::de::Unexpected::Str(&curie),
                                        &"missing delimiter",
                                    )
                                }
                            })?
                        } else {
                            // No more elements.
                            return Ok(values);
                        };

                        let b = if let Some(curie) =
                            seq.next_element::<std::borrow::Cow<'de, str>>()?
                        {
                            curie.parse::<TermId>().map_err(|e| match e {
                                TermIdParseError::MissingDelimiter => {
                                    serde::de::Error::invalid_value(
                                        serde::de::Unexpected::Str(&curie),
                                        &"missing delimiter",
                                    )
                                }
                            })?
                        } else {
                            return Err(serde::de::Error::custom(
                                "missing 2nd curie of the term pair",
                            ));
                        };

                        let ic_mica = if let Some(val) = seq.next_element::<f64>()? {
                            val
                        } else {
                            return Err(serde::de::Error::custom(
                                "missing 2nd curie of the term pair",
                            ));
                        };
                        values.insert(TermIdPair::from([a, b]), ic_mica);
                    }
                }
            }

            deserializer.deserialize_seq(TermPairVisitor)
        }
    }
    #[cfg(test)]
    mod test_serde {
        use serde_test::{assert_tokens, Token};

        use crate::{
            sim::ic::{IcMicaCollector, IcMicaContainer, TermIdPair},
            TermId,
        };

        #[test]
        fn test_roundtrip() {
            let pairs = [
                (
                    TermIdPair::from([
                        &"HP:1".parse::<TermId>().unwrap(),
                        &"HP:2".parse::<TermId>().unwrap(),
                    ]),
                    1.23,
                ),
                (
                    TermIdPair::from([
                        &"HP:2".parse::<TermId>().unwrap(),
                        &"HP:4".parse::<TermId>().unwrap(),
                    ]),
                    3.12,
                ),
            ];

            let mut container = IcMicaContainer::new("v2026-06-24");
            container.collect_all(pairs);

            assert_tokens(
                &container,
                &[
                    Token::Struct {
                        name: "IcMicaContainer",
                        len: 2,
                    },
                    Token::Str("version"),
                    Token::Str("v2026-06-24"),
                    Token::Str("values"),
                    Token::Seq { len: None },
                    Token::Str("HP:1"),
                    Token::Str("HP:2"),
                    Token::F64(1.23),
                    Token::Str("HP:2"),
                    Token::Str("HP:4"),
                    Token::F64(3.12),
                    Token::SeqEnd,
                    Token::StructEnd,
                ],
            );
        }
    }
}

pub fn compute_ic_mica<H, IC, C>(o: &H, root: &TermId, ic: IC, collector: &mut C)
where
    H: HierarchyWalks,
    IC: Fn(&TermId) -> Option<f64>,
    C: IcMicaCollector,
{
    let terms: Vec<_> = o.iter_term_and_descendant_ids(root).collect();
    let mut anc = std::collections::HashSet::new();
    for (i, &left) in terms.iter().enumerate() {
        anc.extend(o.iter_term_and_ancestor_ids(left));
        for &right in &terms[i..] {
            if let Some(ic_mica) = o
                .iter_term_and_ancestor_ids(right)
                .filter(|&t| anc.contains(t))
                .flat_map(|t| ic(t).filter(|&f| f > 0.))
                .reduce(f64::max)
            {
                collector.collect(TermIdPair::from([left, right]), ic_mica)
            }
        }
        anc.clear();
    }
}

/// Collects the information content (IC) values computed in [`IcCalculator`].
pub trait IcCollector {
    fn collect(&mut self, term_id: TermId, ic: f64);
}

impl IcCollector for std::collections::HashMap<TermId, f64> {
    fn collect(&mut self, term_id: TermId, ic: f64) {
        self.insert(term_id, ic);
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
    counter: std::collections::HashMap<K, u64>,
}

impl<H> IcCalculator<H> {
    /// Create a new calculator.
    pub fn new(hierarchy: H) -> Self {
        Self {
            hierarchy,
            counter: std::collections::HashMap::new(),
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
        let mut ig = std::collections::HashSet::new();
        self.account_an_item(&mut ig, item);
    }

    /// Submit a corpus of annotated items.
    pub fn submit_items<C, I, F>(&mut self, corpus: C)
    where
        C: IntoIterator<Item = I>,
        I: IntoIterator<Item = F>,
        F: Identified + Observed,
    {
        let mut ig = std::collections::HashSet::new();
        for item in corpus {
            self.account_an_item(&mut ig, item);
        }
    }

    fn account_an_item<I, F>(&mut self, ig: &mut std::collections::HashSet<K>, item: I)
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
    pub fn collect_ic<R, C>(&self, root: &R, collector: &mut C)
    where
        R: Identified,
        C: IcCollector,
    {
        if let Some(root_idx) = self.hierarchy.term_index(root.identifier()) {
            if let Some(pop_cnt) = self.counter.get(&root_idx) {
                if let Some(root_term_id) = self.hierarchy.idx_to_term_id(Clone::clone(&root_idx)) {
                    collector.collect(root_term_id.clone(), 0.);

                    for desc_idx in self.hierarchy.iter_descendant_idxs(root_idx) {
                        if let Some(cnt) = self.counter.get(&desc_idx) {
                            if let Some(term_id) = self.hierarchy.idx_to_term_id(desc_idx) {
                                let ic = f64::log2((*pop_cnt as f64) / (*cnt as f64));
                                collector.collect(term_id.clone(), ic)
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
        approx::assert_abs_diff_eq!(collector.get(&SEIZURE).unwrap(), &0.415_037, epsilon = 5e-5);
        assert_eq!(collector.get(&CLONIC_SEIZURE), Some(&2.));
        assert_eq!(collector.get(&POLYDACTYLY), Some(&2.));
    }
}

// #[cfg(test)]
// mod test_compute_ic_mica {
//     use std::{collections::HashMap, fs::File, io::BufWriter};

//     use flate2::{write::GzEncoder, Compression};

//     use crate::{
//         common::hpo::{
//             test::{ARACHNODACTYLY, CLONIC_SEIZURE, HYPERTENSION, POLYDACTYLY, SEIZURE},
//             PHENOTYPIC_ABNORMALITY,
//         },
//         ontology::MetadataAware,
//         sim::{
//             feature::{IndividualFeature, IndividualFeatureBuilder},
//             ic::{compute_ic_mica, IcCalculator, IcMicaContainer},
//             ObservationStatus,
//         },
//         test::hpo,
//         TermId,
//     };

//     fn make_feature<'a>(term_id: &'a TermId, status: ObservationStatus) -> IndividualFeature<'a> {
//         IndividualFeatureBuilder::from(term_id)
//             .with_status(status)
//             .build()
//     }

//     #[test]
//     #[ignore = "ran manually"]
//     fn compute_ic_mica_naive() {
//         let hpo = hpo();

//         let mut ic_calculator = IcCalculator::new(hpo);

//         let items = vec![
//             vec![
//                 make_feature(&ARACHNODACTYLY, ObservationStatus::Present),
//                 make_feature(&CLONIC_SEIZURE, ObservationStatus::Present),
//             ],
//             vec![
//                 make_feature(&SEIZURE, ObservationStatus::Present),
//                 make_feature(&HYPERTENSION, ObservationStatus::Present),
//             ],
//             vec![make_feature(&POLYDACTYLY, ObservationStatus::Present)],
//             vec![make_feature(&SEIZURE, ObservationStatus::Present)],
//         ];
//         ic_calculator.submit_items(&items);

//         let mut ic: HashMap<TermId, f64> = HashMap::new();
//         ic_calculator.collect_ic(&PHENOTYPIC_ABNORMALITY, &mut ic);

//         let mut collector = IcMicaContainer::new(hpo.version());

//         compute_ic_mica(
//             hpo,
//             &PHENOTYPIC_ABNORMALITY,
//             |t| ic.get(t).copied(),
//             &mut collector,
//         );

//         // Takes around 134 seconds on the release build.
//         println!("Computed for {} term pairs", collector.len());

//         if let Ok(f) = File::options()
//             .create(true)
//             .write(true)
//             .open("stuff.json.gz")
//         {
//             let mut w = GzEncoder::new(BufWriter::new(f), Compression::best());
//             serde_json::to_writer(&mut w, &collector).expect("No issues")
//         }
//     }
// }
