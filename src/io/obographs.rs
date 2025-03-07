use std::io::BufRead;
use std::str::FromStr;
use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use curieosa::{CurieUtil, TrieCurieUtil};
use obographs_dev::model::{Edge, GraphDocument, Meta, Node};

use crate::{
    base::{
        term::{simple::SimpleMinimalTerm, MinimalTerm},
        Identified, TermId,
    },
    hierarchy::{GraphEdge, Relationship},
    ontology::OntologyIdx,
};

use super::{OntologyData, OntologyDataParser, OntologyLoaderBuilder, Uninitialized, WithParser};

fn parse_alt_term_ids(node_meta: &Meta) -> Vec<TermId> {
    node_meta
        .basic_property_values
        .iter()
        .filter(|&bpv| bpv.pred.ends_with("#hasAlternativeId"))
        .flat_map(|bpv| match TermId::from_str(&bpv.val) {
            Ok(term_id) => Some(term_id),
            Err(e) => {
                eprintln!("{}", e); // TODO: really?
                None
            }
        })
        .collect()
}

pub struct ObographsParser<CU> {
    curie_util: CU,
}

impl Default for ObographsParser<TrieCurieUtil> {
    fn default() -> Self {
        Self {
            curie_util: TrieCurieUtil::default(),
        }
    }
}

impl<CU> ObographsParser<CU>
where
    CU: CurieUtil,
{
    pub fn new(curie_util: CU) -> Self {
        Self { curie_util }
    }
}

impl<CU> ObographsParser<CU>
where
    CU: CurieUtil,
{
    fn create(&self, data: &Node) -> Result<SimpleMinimalTerm> {
        let cp = self.curie_util.get_curie_data(&data.id);
        let name = &data.lbl;

        match (cp, name) {
            (Some(cp), Some(name)) => {
                let term_id = TermId::from((cp.get_prefix(), cp.get_id()));
                let (alt_term_ids, is_obsolete) = match &data.meta {
                    Some(meta) => (parse_alt_term_ids(meta), meta.deprecated.unwrap_or(false)),
                    None => (vec![], false),
                };
                Ok(SimpleMinimalTerm::new(
                    term_id,
                    name,
                    alt_term_ids,
                    is_obsolete,
                ))
            }
            (Some(cp), None) => bail!("Missing term label for {}:{}", cp.get_prefix(), cp.get_id()),
            (None, Some(lbl)) => bail!("Unparsable term id of {}: {}", lbl, &data.id),
            _ => bail!("Unparsable node"),
        }
    }
}

impl<CU, I> OntologyDataParser<I, SimpleMinimalTerm> for ObographsParser<CU>
where
    CU: CurieUtil,
    I: OntologyIdx,
{
    fn load_from_buf_read<R: BufRead>(
        &self,
        read: R,
    ) -> Result<OntologyData<I, SimpleMinimalTerm>> {
        let gd = GraphDocument::from_reader(read).context("Reading graph document")?;

        let graph = gd.graphs.first().context("Getting the first graph")?;

        // Filter out the obsolete terms
        let terms: Vec<_> = graph
            .nodes
            .iter()
            .flat_map(|node| self.create(node).ok())
            .filter(MinimalTerm::is_current)
            .collect();

        let termid2idx: HashMap<_, _> = terms
            .iter()
            .map(Identified::identifier)
            .enumerate()
            .map(|(i, t)| (t, I::new(i)))
            .collect();

        let edges: Vec<GraphEdge<_>> = graph
            .edges
            .iter()
            .flat_map(|edge| parse_edge(edge, &self.curie_util, &termid2idx))
            .collect();

        let metadata = HashMap::new(); // TODO: parse out metadata

        Ok(OntologyData::from((terms, edges, metadata)))
    }
}

fn parse_edge<HI>(
    edge: &Edge,
    curie_util: &dyn CurieUtil,
    termid2idx: &HashMap<&TermId, HI>,
) -> Option<GraphEdge<HI>>
where
    HI: Clone,
{
    let sub_parts = curie_util.get_curie_data(&edge.sub);
    let obj_parts = curie_util.get_curie_data(&edge.obj);
    match (sub_parts, obj_parts) {
        (Some(sub), Some(obj)) => {
            let sub = TermId::from((sub.get_prefix(), sub.get_id()));
            let obj = TermId::from((obj.get_prefix(), obj.get_id()));
            if let Ok(pred) = parse_relationship(&edge.pred) {
                match (termid2idx.get(&sub), termid2idx.get(&obj)) {
                    (Some(sub_idx), Some(obj_idx)) => {
                        Some(GraphEdge::from((sub_idx.clone(), pred, obj_idx.clone())))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn parse_relationship(pred: &str) -> Result<Relationship> {
    match pred {
        // This may be too simplistic
        "is_a" => Ok(Relationship::Child),
        "http://purl.obolibrary.org/obo/BFO_0000050" => Ok(Relationship::PartOf),
        _ => bail!("Unknown predicate {:?}", pred),
    }
}

/// Add a convenience function for using [`ObographsParser`] to [`OntologyLoaderBuilder`].
impl OntologyLoaderBuilder<Uninitialized> {
    /// Load ontology graphs using [`ObographsParser`].        
    #[must_use]
    pub fn obographs_parser(
        self,
    ) -> OntologyLoaderBuilder<WithParser<ObographsParser<TrieCurieUtil>>>
    {
        OntologyLoaderBuilder {
            state: WithParser {
                parser: ObographsParser::default(),
            },
        }
    }
}
