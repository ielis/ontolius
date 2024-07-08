use std::io::BufRead;
use std::str::FromStr;
use std::{collections::HashMap, marker::PhantomData};

use anyhow::{bail, Context, Result};
use curie_util::{CurieUtil, TrieCurieUtil};
use obographs::model::{Edge, GraphDocument, Meta, Node};

use crate::{
    base::{term::simple::SimpleMinimalTerm, Identified, TermId},
    hierarchy::{GraphEdge, HierarchyIdx, Relationship},
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

pub struct ObographsParser<CU, HI> {
    curie_util: CU,
    _marker: PhantomData<HI>,
}

impl<HI> Default for ObographsParser<TrieCurieUtil, HI> {
    fn default() -> Self {
        Self {
            curie_util: TrieCurieUtil::default(),
            _marker: Default::default(),
        }
    }
}

impl<CU, I> ObographsParser<CU, I>
where
    CU: CurieUtil,
{
    pub fn new(curie_util: CU) -> Self {
        Self {
            curie_util,
            _marker: PhantomData,
        }
    }

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

impl<CU, I> OntologyDataParser for ObographsParser<CU, I>
where
    CU: CurieUtil,
    I: OntologyIdx,
{
    type HI = I;
    type T = SimpleMinimalTerm;

    fn load_from_buf_read<R: BufRead>(&self, read: R) -> Result<OntologyData<Self::HI, Self::T>> {
        let gd = GraphDocument::from_reader(read).context("Reading graph document")?;

        let graph = gd.graphs.first().context("Getting the first graph")?;
        
        let terms: Vec<_> = graph
            .nodes
            .iter()
            .flat_map(|node| self.create(node).ok())
            .collect();

        let term_ids: Vec<_> = terms.iter().map(Identified::identifier).collect();
        let termid2idx: HashMap<_, _> = term_ids
            .iter()
            .enumerate()
            .map(|(i, &t)| (t.to_string(), I::new(i)))
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

fn parse_edge<HI: HierarchyIdx>(
    edge: &Edge,
    curie_util: &dyn CurieUtil,
    termid2idx: &HashMap<String, HI>,
) -> Option<GraphEdge<HI>> {
    let sub_parts = curie_util.get_curie_data(&edge.sub);
    let rel = parse_relationship(&edge.pred);
    let obj_parts = curie_util.get_curie_data(&edge.obj);
    match (sub_parts, rel, obj_parts) {
        (Some(sub), Ok(pred), Some(obj)) => {
            // TODO: the matching is hacky and likely inefficient. Improve!
            let sub = format!("{}:{}", sub.get_prefix(), sub.get_id());
            let obj = format!("{}:{}", obj.get_prefix(), obj.get_id());
            match (termid2idx.get(&sub), termid2idx.get(&obj)) {
                (Some(sub_idx), Some(obj_idx)) => Some(GraphEdge::from((*sub_idx, pred, *obj_idx))),
                _ => None,
            }
        }
        (_, Err(e), _) => {
            println!("Missing relationship: {e}");
            None
        }
        _ => None,
    }
}

fn parse_relationship(pred: &str) -> Result<Relationship> {
    match pred {
        // This may be too simplistic
        "is_a" => Ok(Relationship::Child),
        _ => bail!("Unknown predicate {}", pred),
    }
}

/// Add a convenience function for using [`ObographsParser`] to [`OntologyLoaderBuilder`].
impl OntologyLoaderBuilder<Uninitialized> {
    /// Load ontology graphs using [`ObographsParser`].        
    #[must_use]
    pub fn obographs_parser<HI>(
        self,
    ) -> OntologyLoaderBuilder<WithParser<ObographsParser<TrieCurieUtil, HI>>>
    where
        HI: OntologyIdx,
    {
        OntologyLoaderBuilder {
            state: WithParser {
                parser: ObographsParser::default(),
            },
        }
    }
}
