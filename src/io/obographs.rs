//! Load ontology data from Obographs formats.
use std::convert::Infallible;
use std::error::Error;
use std::fmt::Display;
use std::io::BufRead;
use std::{collections::HashMap, sync::Arc};

use anyhow::{bail, Context};
use curieosa::{CurieUtil, TrieCurieUtil};
use obographs_dev::model::{
    DefinitionPropertyValue, Edge, Graph, GraphDocument, Meta, Node, SynonymPropertyValue,
};

use crate::term::simple::{SimpleMinimalTerm, SimpleTerm};
use crate::term::{Definition, MinimalTerm, Synonym, SynonymCategory, SynonymType};
use crate::{Identified, TermId};

use super::{
    GraphEdge, Index, OntologyData, OntologyDataParser, OntologyLoaderBuilder, Relationship,
    Uninitialized, WithParser,
};

impl TryFrom<DefinitionPropertyValue> for Definition {
    type Error = Infallible; // Can be replaced with a more specific error in the future.
    fn try_from(value: DefinitionPropertyValue) -> std::result::Result<Self, Self::Error> {
        Ok(Definition {
            val: value.val,
            xrefs: value.xrefs.unwrap_or_default(),
        })
    }
}

/// Represents the reasons why parsing of a Synonym can fail.
///
/// We reserve the right to add more reasons for failure in the future.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum SynonymParseError {
    /// An unknown synonym type was encountered.
    UnknownType(String),
    /// An unknown synonym category was encountered.
    UnknownCategory(String),
    /// Parsing of the external reference (xref) failed.
    UnparsableXref(String),
}

impl Display for SynonymParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SynonymParseError::UnknownType(v) => {
                write!(f, "Unknown synonym type {v}")
            }
            SynonymParseError::UnknownCategory(v) => {
                write!(f, "Unknown synonym category {v}")
            }
            SynonymParseError::UnparsableXref(v) => {
                write!(f, "Unparsable xref {v}")
            }
        }
    }
}

impl Error for SynonymParseError {}

fn parse_synonym_category(pred: String) -> Result<SynonymCategory, SynonymParseError> {
    match pred.as_str() {
        "hasExactSynonym" => Ok(SynonymCategory::Exact),
        "hasBroadSynonym" => Ok(SynonymCategory::Broad),
        "hasNarrowSynonym" => Ok(SynonymCategory::Narrow),
        "hasRelatedSynonym" => Ok(SynonymCategory::Related),
        _ => Err(SynonymParseError::UnknownCategory(pred)),
    }
}

fn parse_synonym_type(synonym_type: String) -> Result<SynonymType, SynonymParseError> {
    match synonym_type.as_ref() {
        "http://purl.obolibrary.org/obo/hp#layperson" => Ok(SynonymType::LaypersonTerm),
        "http://purl.obolibrary.org/obo/hp#abbreviation" => Ok(SynonymType::Abbreviation),
        "http://purl.obolibrary.org/obo/hp#uk_spelling" => Ok(SynonymType::UkSpelling),
        "http://purl.obolibrary.org/obo/hp#obsolete_synonym" => Ok(SynonymType::ObsoleteSynonym),
        "http://purl.obolibrary.org/obo/hp#plural_form" => Ok(SynonymType::PluralForm),
        "http://purl.obolibrary.org/obo/hp#allelic_requirement" => {
            Ok(SynonymType::AllelicRequirement)
        }
        "http://purl.obolibrary.org/obo/go#systematic_synonym" => {
            Ok(SynonymType::SystematicSynonym)
        }
        "http://purl.obolibrary.org/obo/go#syngo_official_label" => {
            Ok(SynonymType::SyngoOfficialLabel)
        }
        _ => Err(SynonymParseError::UnknownType(synonym_type)),
    }
}

fn parse_synonym_xref(xref: String) -> Result<TermId, SynonymParseError> {
    if let Some(remainder) = xref.strip_prefix("https://orcid.org/") {
        Ok(TermId::from(("ORCID", remainder)))
    } else {
        xref.parse()
            .map_err(|_x| SynonymParseError::UnparsableXref(xref))
    }
}

impl TryFrom<SynonymPropertyValue> for Synonym {
    type Error = SynonymParseError;

    fn try_from(value: SynonymPropertyValue) -> std::result::Result<Self, Self::Error> {
        let category = Some(parse_synonym_category(value.pred)?);

        let synonym_type = if let Some(st) = value.synonym_type {
            Some(parse_synonym_type(st)?)
        } else {
            None
        };

        let xrefs = if let Some(xrefs) = value.xrefs {
            let mut parsed = Vec::with_capacity(xrefs.len());
            for xref in xrefs {
                parsed.push(parse_synonym_xref(xref)?);
            }
            parsed
        } else {
            Default::default()
        };

        Ok(Self {
            name: value.val,
            category: category,
            r#type: synonym_type,
            xrefs: xrefs,
        })
    }
}

/// The term factory parses the obographs node into a term.
pub trait ObographsTermMapper<T> {
    fn create(&self, node: Node) -> anyhow::Result<T>;
}

/// `DefaultObographsTermMapper` parses the obograph term nodes
/// into [[SimpleMinimalTerm]] or into [[SimpleTerm]] for a downstream use.
///
/// ### Mapping behavior
///
/// The errors encountered while parsing the mandatory fields of [[crate::term::MinimalTerm]] (or [[crate::term::Term]]) are reported as errors.
/// However, the errors in the optional parts (e.g. one of the alternative term IDs or synonym) are *ignored*.
///
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
            .flat_map(|bpv| bpv.val.parse()) // Ignores unparsable alt term IDs.
            .collect()
    }
}

impl<CU> ObographsTermMapper<SimpleMinimalTerm> for DefaultObographsTermMapper<CU>
where
    CU: CurieUtil,
{
    fn create(&self, node: Node) -> anyhow::Result<SimpleMinimalTerm> {
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
    fn create(&self, node: Node) -> anyhow::Result<SimpleTerm> {
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
                            meta.definition.map(|d| Definition::try_from(d).unwrap_or_default()), // Ignores an unparsable definition.
                            meta.synonyms
                                .into_iter()
                                .flat_map(Synonym::try_from) // Ignores unparsable synonyms.
                                .collect(),
                            meta.xrefs
                                .into_iter()
                                .flat_map(|xpv| xpv.val.parse()) // Ignores unparsable xrefs.
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
    pub fn load_from_graph<I, T>(&self, graph: Graph) -> anyhow::Result<OntologyData<I, T>>
    where
        TF: ObographsTermMapper<T>,
        CU: CurieUtil,
        I: Index,
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
            .into_iter()
            .flat_map(|edge| parse_edge(edge, &*self.curie_util, &termid2idx))
            .collect();

        let metadata = graph.meta.map(parse_metadata).unwrap_or_default();

        Ok(OntologyData::from((terms, edges, metadata)))
    }
}

fn parse_metadata(meta: Box<Meta>) -> HashMap<String, String> {
    let mut metadata = HashMap::new();

    if let Some(version) = parse_version(&meta) {
        metadata.insert("version".into(), version);
    }

    metadata
}
fn parse_version(meta: &Meta) -> Option<String> {
    // First, try to find the version in Basic Property Value item:
    // {
    //    "pred" : "http://www.w3.org/2002/07/owl#versionInfo",
    //    "val" : "2025-01-16"
    //  }
    for bpv in &meta.basic_property_values {
        if bpv.pred.ends_with("#versionInfo") {
            return Some(bpv.val.clone());
        }
    }
    // Next, parse the `version` item, if present:
    // "version" : "http://purl.obolibrary.org/obo/hp/releases/2025-01-16/hp.json"
    if let Some(version) = &meta.version {
        let tokens: Vec<_> = version.split("/").collect();
        if let Some(&penultimate) = tokens.get(tokens.len() - 2) {
            if penultimate
                .split("-")
                .all(|val| val.chars().all(|c| c.is_ascii_digit()))
            {
                return Some(penultimate.to_string());
            }
        }
    }

    // Last, give up and report.
    eprintln!("Could not parse ontology version");
    None
}

impl<TF, CU, I, T> OntologyDataParser<I, T> for ObographsParser<TF, CU>
where
    TF: ObographsTermMapper<T>,
    CU: CurieUtil,
    I: Index,
    T: MinimalTerm,
{
    fn load_from_buf_read<R>(&self, read: R) -> anyhow::Result<OntologyData<I, T>>
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

fn parse_edge<I, CU>(
    edge: Edge,
    curie_util: &CU,
    termid2idx: &HashMap<&TermId, I>,
) -> Option<GraphEdge<I>>
where
    I: Clone,
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

fn parse_relationship(pred: &str) -> anyhow::Result<Relationship> {
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

#[cfg(test)]
mod test_obographs {
    use super::*;
    use crate::term::SynonymType;

    #[test]
    fn test_parse_synonym_type() {
        const PAYLOAD: [(&str, Result<SynonymType, SynonymParseError>); 8] = [
            (
                "http://purl.obolibrary.org/obo/hp#layperson",
                Ok(SynonymType::LaypersonTerm),
            ),
            (
                "http://purl.obolibrary.org/obo/hp#abbreviation",
                Ok(SynonymType::Abbreviation),
            ),
            (
                "http://purl.obolibrary.org/obo/hp#uk_spelling",
                Ok(SynonymType::UkSpelling),
            ),
            (
                "http://purl.obolibrary.org/obo/hp#obsolete_synonym",
                Ok(SynonymType::ObsoleteSynonym),
            ),
            (
                "http://purl.obolibrary.org/obo/hp#plural_form",
                Ok(SynonymType::PluralForm),
            ),
            (
                "http://purl.obolibrary.org/obo/hp#allelic_requirement",
                Ok(SynonymType::AllelicRequirement),
            ),
            (
                "http://purl.obolibrary.org/obo/go#systematic_synonym",
                Ok(SynonymType::SystematicSynonym),
            ),
            (
                "http://purl.obolibrary.org/obo/go#syngo_official_label",
                Ok(SynonymType::SyngoOfficialLabel),
            ),
        ];

        for (value, expected) in PAYLOAD {
            let actual = parse_synonym_type(value.to_string());
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_parse_synonym_type_no_good() {
        assert!(parse_synonym_type("No good value".to_string()).is_err());
    }
}
