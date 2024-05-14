//! Routines for loading ontology data.
#[cfg(feature = "obographs")]
pub mod obographs;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
};

use crate::{
    base::MinimalTerm,
    hierarchy::{GraphEdge, HierarchyIdx},
    prelude::{OntographError, Ontology, TermIdx},
};

pub struct OntologyData<HI, T>
where
    HI: HierarchyIdx,
    T: MinimalTerm,
{
    terms: Box<[T]>,
    edges: Box<[GraphEdge<HI>]>,
    metadata: HashMap<String, String>,
}

impl<HI: HierarchyIdx, T: MinimalTerm> OntologyData<HI, T> {
    pub fn terms(&self) -> &[T] {
        &self.terms
    }

    pub fn edges(&self) -> &[GraphEdge<HI>] {
        &self.edges
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        // TODO: the signature overpromises. We should probably only promise an iterator over (String, String).
        &self.metadata
    }
}

impl<HI, T> From<(Box<[T]>, Box<[GraphEdge<HI>]>, HashMap<String, String>)> for OntologyData<HI, T>
where
    HI: HierarchyIdx,
    T: MinimalTerm,
{
    fn from(value: (Box<[T]>, Box<[GraphEdge<HI>]>, HashMap<String, String>)) -> Self {
        Self {
            terms: value.0,
            edges: value.1,
            metadata: value.2,
        }
    }
}

/// Ontology data parser can read [`OntologyData`] from some input
pub trait OntologyDataParser {
    type HI: TermIdx + HierarchyIdx;
    type T: MinimalTerm;

    /// Load ontology data from the buffered reader.
    fn load_from_buf_read<R>(
        &self,
        read: &mut R,
    ) -> Result<OntologyData<Self::HI, Self::T>, OntographError>
    where
        R: BufRead;
}

/// [`OntologyLoader`] parses the input into [`OntologyData`] using supplied [`OntologyDataParser`]
/// and then assembles the data into an [`crate::ontology::Ontology`].
pub struct OntologyLoader<P>
where
    P: OntologyDataParser,
{
    parser: P,
}

impl<P> OntologyLoader<P>
where
    P: OntologyDataParser,
{
    pub fn new(parser: P) -> Self {
        Self { parser }
    }
}

impl<Parser> OntologyLoader<Parser>
where
    Parser: OntologyDataParser,
{
    /// Load ontology from a path.
    pub fn load_from_path<O, P>(&self, path: P) -> Result<O, OntographError>
    where
        O: TryFrom<OntologyData<Parser::HI, Parser::T>, Error = OntographError>
            + Ontology<Idx = Parser::HI, T = Parser::T>,
        P: AsRef<Path>,
    {
        if let Ok(mut file) = File::open(path) {
            self.load_from_read(&mut file)
        } else {
            Err(OntographError::Other("Unable".into()))
        }
    }

    /// Load ontology from a reader.
    pub fn load_from_read<R, O>(&self, read: &mut R) -> Result<O, OntographError>
    where
        R: Read,
        O: TryFrom<OntologyData<Parser::HI, Parser::T>, Error = OntographError>
            + Ontology<Idx = Parser::HI, T = Parser::T>,
    {
        let mut read = BufReader::new(read);
        self.load_from_buf_read(&mut read)
    }

    /// Load ontology from a buffered reader.
    pub fn load_from_buf_read<R, O>(&self, read: &mut R) -> Result<O, OntographError>
    where
        R: BufRead,
        O: TryFrom<OntologyData<Parser::HI, Parser::T>, Error = OntographError>
            + Ontology<Idx = Parser::HI, T = Parser::T>,
    {
        let data = self.parser.load_from_buf_read(read)?;
        O::try_from(data)
    }
}

pub struct Uninitialized;

pub struct WithParser<P>
where
    P: OntologyDataParser,
{
    parser: P,
}

pub struct OntologyLoaderBuilder<State> {
    state: State,
}

impl Default for OntologyLoaderBuilder<Uninitialized> {
    fn default() -> Self {
        Self::new()
    }
}

impl OntologyLoaderBuilder<Uninitialized> {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            state: Uninitialized,
        }
    }

    /// Set [`OntologyDataParser`] for parsing ontology data
    #[must_use]
    pub fn parser<P>(self, parser: P) -> OntologyLoaderBuilder<WithParser<P>>
    where
        P: OntologyDataParser,
    {
        OntologyLoaderBuilder {
            state: WithParser { parser },
        }
    }
}

impl<P: OntologyDataParser> OntologyLoaderBuilder<WithParser<P>> {
    /// Build the ontology loader.
    pub fn build(self) -> OntologyLoader<P> {
        OntologyLoader::new(self.state.parser)
    }
}
