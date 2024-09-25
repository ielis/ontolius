//! Routines for loading ontology data.
#[cfg(feature = "obographs")]
pub mod obographs;

use anyhow::{Context, Result};

use flate2::read::GzDecoder;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
};

use crate::{hierarchy::GraphEdge, prelude::Ontology};

pub struct OntologyData<HI, T> {
    pub terms: Vec<T>,
    pub edges: Vec<GraphEdge<HI>>,
    pub metadata: HashMap<String, String>,
}

impl<HI, T> From<(Vec<T>, Vec<GraphEdge<HI>>, HashMap<String, String>)> for OntologyData<HI, T> {
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
    fn load_from_buf_read<R>(&self, read: R) -> Result<OntologyData<Self::HI, Self::T>>
    where
        R: BufRead;
}

/// [`OntologyLoader`] parses the input into [`OntologyData`] using supplied [`OntologyDataParser`]
/// and then assembles the data into an [`crate::ontology::Ontology`].
pub struct OntologyLoader<P> {
    parser: P,
}

impl<P> OntologyLoader<P> {
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
    pub fn load_from_path<O, P>(&self, path: P) -> Result<O>
    where
        P: AsRef<Path>,
        O: TryFrom<OntologyData<Parser::HI, Parser::T>, Error = anyhow::Error>
            + Ontology<Parser::HI, Parser::T>,
    {
        let path = path.as_ref();
        let file = File::open(path).with_context(|| format!("Opening file at {:?}", path))?;
        if let Some(extension) = path.extension() {
            if extension == "gz" {
                // Decompress gzipped file on the fly.
                self.load_from_read(GzDecoder::new(file))
            } else {
                // All other extensions, e.g. JSON
                self.load_from_read(file)
            }
        } else {
            // We will also read from a file with no extension,
            // assuming plain text.
            self.load_from_read(file)
        }
    }

    /// Load ontology from a reader.
    pub fn load_from_read<R, O>(&self, read: R) -> Result<O>
    where
        R: Read,
        O: TryFrom<OntologyData<Parser::HI, Parser::T>, Error = anyhow::Error>
            + Ontology<Parser::HI, Parser::T>,
    {
        self.load_from_buf_read(BufReader::new(read))
    }

    /// Load ontology from a buffered reader.
    pub fn load_from_buf_read<R, O>(&self, read: R) -> Result<O>
    where
        R: BufRead,
        O: TryFrom<OntologyData<Parser::HI, Parser::T>, Error = anyhow::Error>
            + Ontology<Parser::HI, Parser::T>,
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
        Self {
            state: Uninitialized,
        }
    }
}

impl OntologyLoaderBuilder<Uninitialized> {
    /// Create a new builder.
    pub fn new() -> Self {
        OntologyLoaderBuilder::default()
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

impl<P> OntologyLoaderBuilder<WithParser<P>>
where
    P: OntologyDataParser,
{
    /// Build the ontology loader.
    pub fn build(self) -> OntologyLoader<P> {
        OntologyLoader::new(self.state.parser)
    }
}
