//! Load ontology data from Obographs formats.
use std::io::BufRead;
use std::{collections::HashMap, sync::Arc};

use anyhow::{bail, Context, Result};
use curieosa::{CurieUtil, TrieCurieUtil};
use obographs_dev::model::{
    DefinitionPropertyValue, Edge, Graph, GraphDocument, Meta, Node, SynonymPropertyValue,
};

use crate::term::simple::{SimpleMinimalTerm, SimpleTerm};
use crate::term::{Definition, MinimalTerm, Synonym, SynonymCategory, SynonymType};
use crate::{ontology::OntologyIdx, Identified, TermId};

use super::{
    GraphEdge, OntologyData, OntologyDataParser, OntologyLoaderBuilder, Relationship,
    Uninitialized, WithParser,
};

impl From<DefinitionPropertyValue> for Definition {
    fn from(value: DefinitionPropertyValue) -> Self {
        Definition {
            val: value.val,
            xrefs: value.xrefs.unwrap_or_default(),
        }
    }
}

fn parse_synonym_category(pred: String) -> Option<SynonymCategory> {
    match pred.as_str() {
        "hasExactSynonym" => Some(SynonymCategory::Exact),
        "hasBroadSynonym" => Some(SynonymCategory::Broad),
        "hasNarrowSynonym" => Some(SynonymCategory::Narrow),
        "hasRelatedSynonym" => Some(SynonymCategory::Related),
        _ => {
            eprintln!("Unknown synonym category {pred:?}",);
            None
        }
    }
}

fn parse_synonym_type(synonym_type: String) -> Option<SynonymType> {
    match synonym_type.as_ref() {
        "http://purl.obolibrary.org/obo/hp#layperson" => Some(SynonymType::LaypersonTerm),
        "http://purl.obolibrary.org/obo/hp#abbreviation" => Some(SynonymType::Abbreviation),
        "http://purl.obolibrary.org/obo/hp#uk_spelling" => Some(SynonymType::UkSpelling),
        "http://purl.obolibrary.org/obo/hp#obsolete_synonym" => Some(SynonymType::ObsoleteSynonym),
        "http://purl.obolibrary.org/obo/hp#plural_form" => Some(SynonymType::PluralForm),
        "http://purl.obolibrary.org/obo/go#systematic_synonym" => {
            Some(SynonymType::SystematicSynonym)
        }
        "http://purl.obolibrary.org/obo/go#syngo_official_label" => {
            Some(SynonymType::SyngoOfficialLabel)
        }
        _ => {
            eprintln!("Unknown synonym category {synonym_type:?}",);
            None
        }
    }
}

fn parse_synonym_xref(xref: String) -> Option<TermId> {
    if let Some(remainder) = xref.strip_prefix("https://orcid.org/") {
        Some(TermId::from(("ORCID", remainder)))
    } else {
        xref.parse().ok()
    }
}

impl From<SynonymPropertyValue> for Synonym {
    fn from(value: SynonymPropertyValue) -> Self {
        Self {
            name: value.val,
            category: parse_synonym_category(value.pred),
            r#type: value.synonym_type.and_then(parse_synonym_type),
            xrefs: value
                .xrefs
                .unwrap_or_default()
                .into_iter()
                .flat_map(parse_synonym_xref)
                .collect(),
        }
    }
}

/// The term factory parses the obographs node into a term.
pub trait ObographsTermMapper<T> {
    fn create(&self, node: Node) -> Result<T>;
}

#[derive(Default)]
pub struct DefaultObographsTermMapper<CU> {
    curie_util: Arc<CU>,
}

impl<CU> DefaultObographsTermMapper<CU> {
    fn new(curie_util: Arc<CU>) -> Self {
        Self { curie_util }
    }

    fn parse_comment(comments: Vec<String>) -> Option<String> {
        match comments.len() {
            0 => None,
            _ => Some(comments.join(" ")),
        }
    }

    fn parse_alt_term_ids(node_meta: &Meta) -> Vec<TermId> {
        node_meta
            .basic_property_values
            .iter()
            .filter(|&bpv| bpv.pred.ends_with("#hasAlternativeId"))
            .flat_map(|bpv| match bpv.val.parse() {
                Ok(term_id) => Some(term_id),
                Err(e) => {
                    eprintln!("{}", e); // TODO: really?
                    None
                }
            })
            .collect()
    }
}

impl<CU> ObographsTermMapper<SimpleMinimalTerm> for DefaultObographsTermMapper<CU>
where
    CU: CurieUtil,
{
    fn create(&self, node: Node) -> Result<SimpleMinimalTerm> {
        let cp = self.curie_util.get_curie_data(&node.id);

        match (cp, node.lbl) {
            (Some(cp), Some(name)) => {
                let term_id = TermId::from((cp.get_prefix(), cp.get_id()));

                let (alt_term_ids, is_obsolete) = match node.meta {
                    Some(meta) => (
                        Self::parse_alt_term_ids(&meta),
                        meta.deprecated.unwrap_or(false),
                    ),
                    None => (Default::default(), false),
                };

                Ok(SimpleMinimalTerm::new(
                    term_id,
                    name,
                    alt_term_ids,
                    is_obsolete,
                ))
            }
            (Some(cp), None) => bail!("Missing term label for {}:{}", cp.get_prefix(), cp.get_id()),
            (None, Some(lbl)) => bail!("Unparsable term id of {}: {}", lbl, &node.id),
            _ => bail!("Unparsable node"),
        }
    }
}

impl<CU> ObographsTermMapper<SimpleTerm> for DefaultObographsTermMapper<CU>
where
    CU: CurieUtil,
{
    fn create(&self, node: Node) -> Result<SimpleTerm> {
        let cp = self.curie_util.get_curie_data(&node.id);

        match (cp, node.lbl) {
            (Some(cp), Some(name)) => {
                let term_id = TermId::from((cp.get_prefix(), cp.get_id()));

                let (alt_term_ids, is_obsolete, comment, definition, synonyms, xrefs) =
                    match node.meta {
                        Some(meta) => (
                            Self::parse_alt_term_ids(&meta),
                            meta.deprecated.unwrap_or(false),
                            Self::parse_comment(meta.comments),
                            meta.definition.map(Definition::from),
                            meta.synonyms.into_iter().map(Synonym::from).collect(),
                            // Ignore unparsable Xrefs.
                            meta.xrefs
                                .into_iter()
                                .flat_map(|xpv| xpv.val.parse())
                                .collect(),
                        ),
                        None => Default::default(),
                    };

                Ok(SimpleTerm::new(
                    term_id,
                    name,
                    alt_term_ids,
                    is_obsolete,
                    definition,
                    comment,
                    synonyms,
                    xrefs,
                ))
            }
            (Some(cp), None) => bail!("Missing term label for {}:{}", cp.get_prefix(), cp.get_id()),
            (None, Some(lbl)) => bail!("Unparsable term id of {}: {}", lbl, &node.id),
            _ => bail!("Unparsable node"),
        }
    }
}

#[derive(Default)]
pub struct ObographsParser<TM, CU> {
    term_mapper: TM,
    curie_util: Arc<CU>,
}

impl<TM, CU> ObographsParser<TM, CU> {
    pub fn new(term_mapper: TM, curie_util: Arc<CU>) -> Self {
        Self {
            term_mapper,
            curie_util,
        }
    }
}

impl<TF, CU> ObographsParser<TF, CU> {
    pub fn load_from_graph<I, T>(&self, graph: Graph) -> Result<OntologyData<I, T>>
    where
        TF: ObographsTermMapper<T>,
        CU: CurieUtil,
        I: OntologyIdx,
        T: MinimalTerm,
    {
        let terms: Vec<_> = graph
            .nodes
            .into_iter()
            .flat_map(|node| self.term_mapper.create(node).ok())
            .filter(MinimalTerm::is_current) // ◀️◀️◀️ filters out the obsolete terms
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
            .flat_map(|edge| parse_edge(edge, &*self.curie_util, &termid2idx))
            .collect();

        let metadata = HashMap::new(); // TODO: parse out metadata

        Ok(OntologyData::from((terms, edges, metadata)))
    }
}

impl<TF, CU, I, T> OntologyDataParser<I, T> for ObographsParser<TF, CU>
where
    TF: ObographsTermMapper<T>,
    CU: CurieUtil,
    I: OntologyIdx,
    T: MinimalTerm,
{
    fn load_from_buf_read<R>(&self, read: R) -> Result<OntologyData<I, T>>
    where
        R: BufRead,
    {
        let mut graph_document =
            GraphDocument::from_reader(read).context("Reading graph document")?;
        self.load_from_graph(
            graph_document
                .graphs
                .pop()
                .expect("Obographs document should include at least one graph"),
        )
    }
}

fn parse_edge<HI, CU>(
    edge: &Edge,
    curie_util: &CU,
    termid2idx: &HashMap<&TermId, HI>,
) -> Option<GraphEdge<HI>>
where
    HI: Clone,
    CU: CurieUtil,
{
    let sub_parts = curie_util.get_curie_data(&edge.sub);
    let obj_parts = curie_util.get_curie_data(&edge.obj);
    match (sub_parts, obj_parts) {
        (Some(sub), Some(obj)) => {
            let sub = TermId::from((sub.get_prefix(), sub.get_id()));
            let obj = TermId::from((obj.get_prefix(), obj.get_id()));
            if let Ok(pred) = parse_relationship(&edge.pred) {
                match (termid2idx.get(&sub), termid2idx.get(&obj)) {
                    (Some(sub_idx), Some(obj_idx)) => Some(GraphEdge::from((
                        Clone::clone(sub_idx),
                        pred,
                        Clone::clone(obj_idx),
                    ))),
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
    ///
    /// ## Example
    ///
    /// Configure a loader for loading ontology
    /// from an Obographs JSON file.
    ///
    /// ```
    /// use ontolius::io::OntologyLoaderBuilder;
    ///
    /// let builder = OntologyLoaderBuilder::new()
    ///                 .obographs_parser()
    ///                 .build();
    /// ```
    #[must_use]
    pub fn obographs_parser(
        self,
    ) -> OntologyLoaderBuilder<
        WithParser<ObographsParser<DefaultObographsTermMapper<TrieCurieUtil>, TrieCurieUtil>>,
    > {
        OntologyLoaderBuilder {
            state: WithParser {
                parser: ObographsParser::default(),
            },
        }
    }

    #[must_use]
    pub fn obographs_parser_with_curie_util<CU>(
        self,
        cu: CU,
    ) -> OntologyLoaderBuilder<WithParser<ObographsParser<DefaultObographsTermMapper<CU>, CU>>>
    {
        let cu = Arc::new(cu);
        let tm = DefaultObographsTermMapper::new(Arc::clone(&cu));
        OntologyLoaderBuilder {
            state: WithParser {
                parser: ObographsParser::new(tm, Arc::clone(&cu)),
            },
        }
    }
}
