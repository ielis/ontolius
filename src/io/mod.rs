//! Routines for loading ontology data.
#[cfg(feature = "obographs")]
pub mod obographs;

use flate2::read::GzDecoder;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
};

use crate::{
    hierarchy::GraphEdge,
    prelude::{OntoliusError, Ontology},
};

pub struct OntologyData<HI, T>
{
    pub terms: Vec<T>,
    pub edges: Vec<GraphEdge<HI>>,
    pub metadata: HashMap<String, String>,
}

impl<HI, T> From<(Vec<T>, Vec<GraphEdge<HI>>, HashMap<String, String>)> for OntologyData<HI, T>
{
    fn from(value: (Vec<T>, Vec<GraphEdge<HI>>, HashMap<String, String>)) -> Self {
        Self {
            terms: value.0,
            edges: value.1,
            metadata: value.2,
        }
    }
}

/// Ontology data parser can read [`OntologyData`] from some input
pub trait OntologyDataParser {
    type HI;
    type T;

    /// Load ontology data from the buffered reader.
    fn load_from_buf_read<R>(
        &self,
        read: &mut R,
    ) -> Result<OntologyData<Self::HI, Self::T>, OntoliusError>
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
    /// 
    /// Gzipped content is uncompressed on the fly,
    /// as long as the `path` is suffixed with `*.gz`.
    pub fn load_from_path<O, P>(&self, path: P) -> Result<O, OntoliusError>
    where
        O: TryFrom<OntologyData<Parser::HI, Parser::T>, Error = OntoliusError>
            + Ontology<Idx = Parser::HI, T = Parser::T>,
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        if let Ok(mut file) = File::open(path) {
            if let Some(extension) = path.extension() {
                if extension == "gz" {
                    // Decompress gzipped file on the fly.
                    let mut read = GzDecoder::new(file);
                    self.load_from_read(&mut read)
                } else {
                    // All other extensions, e.g. JSON
                    self.load_from_read(&mut file)
                }
            } else {
                // We will also read from a file with no extension,
                // assuming plain text.
                self.load_from_read(&mut file)
            }
        } else {
            Err(OntoliusError::Other(format!("Cannot load ontology from {path:?}")))
        }
    }

    /// Load ontology from a reader.
    pub fn load_from_read<R, O>(&self, read: &mut R) -> Result<O, OntoliusError>
    where
        R: Read,
        O: TryFrom<OntologyData<Parser::HI, Parser::T>, Error = OntoliusError>
            + Ontology<Idx = Parser::HI, T = Parser::T>,
    {
        let mut read = BufReader::new(read);
        self.load_from_buf_read(&mut read)
    }

    /// Load ontology from a buffered reader.
    pub fn load_from_buf_read<R, O>(&self, read: &mut R) -> Result<O, OntoliusError>
    where
        R: BufRead,
        O: TryFrom<OntologyData<Parser::HI, Parser::T>, Error = OntoliusError>
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
