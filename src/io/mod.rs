//! Routines for loading ontology data.
#[cfg(feature = "obographs")]
pub mod obographs;

use anyhow::{Context, Result};

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
};

use crate::hierarchy::GraphEdge;

pub struct OntologyData<I, T> {
    pub terms: Vec<T>,
    pub edges: Vec<GraphEdge<I>>,
    pub metadata: HashMap<String, String>,
}

impl<I, T> From<(Vec<T>, Vec<GraphEdge<I>>, HashMap<String, String>)> for OntologyData<I, T> {
    fn from(value: (Vec<T>, Vec<GraphEdge<I>>, HashMap<String, String>)) -> Self {
        Self {
            terms: value.0,
            edges: value.1,
            metadata: value.2,
        }
    }
}

/// Ontology data parser can read [`OntologyData`] from some input.
pub trait OntologyDataParser<I, T> {
    /// Load ontology data from the buffered reader.
    fn load_from_buf_read<R>(&self, read: R) -> Result<OntologyData<I, T>>
    where
        R: BufRead;
}

/// [`OntologyLoader`] parses the input into [`OntologyData`] using supplied [`OntologyDataParser`]
/// and then assembles the data into an ontology.
/// 
/// Use [`OntologyLoaderBuilder`] to create the loader and load ontology from a path,
/// read, or buf read.
pub struct OntologyLoader<P> {
    parser: P,
}

impl<P> OntologyLoader<P> {
    fn new(parser: P) -> Self {
        Self { parser }
    }
}

impl<P> OntologyLoader<P> {
    /// Load ontology from a path.
    pub fn load_from_path<I, T, O, Q>(&self, path: Q) -> Result<O>
    where
        P: OntologyDataParser<I, T>,
        Q: AsRef<Path>,
        O: TryFrom<OntologyData<I, T>, Error = anyhow::Error>,
    {
        let path = path.as_ref();
        let file = File::open(path).with_context(|| format!("Opening file at {:?}", path))?;

        self.load_from_read(file)
    }

    /// Load ontology from a reader.
    pub fn load_from_read<I, T, O, R>(&self, read: R) -> Result<O>
    where
        P: OntologyDataParser<I, T>,
        R: Read,
        O: TryFrom<OntologyData<I, T>, Error = anyhow::Error>,
    {
        self.load_from_buf_read(BufReader::new(read))
    }

    /// Load ontology from a buffered reader.
    pub fn load_from_buf_read<I, T, O, R>(&self, read: R) -> Result<O>
    where
        P: OntologyDataParser<I, T>,
        R: BufRead,
        O: TryFrom<OntologyData<I, T>, Error = anyhow::Error>,
    {
        let data = self.parser.load_from_buf_read(read)?;
        O::try_from(data)
    }
}

pub struct Uninitialized;

pub struct WithParser<P> {
    parser: P,
}

/// A builder for configuring [`OntologyLoader`].
/// 
pub struct OntologyLoaderBuilder<State> {
    state: State,
}

/// Creates a new "blank" builder.
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
    pub fn parser<P>(self, parser: P) -> OntologyLoaderBuilder<WithParser<P>> {
        OntologyLoaderBuilder {
            state: WithParser { parser },
        }
    }
}

impl<P> OntologyLoaderBuilder<WithParser<P>> {
    /// Finish the build and get the ontology loader.
    pub fn build(self) -> OntologyLoader<P> {
        OntologyLoader::new(self.state.parser)
    }
}
