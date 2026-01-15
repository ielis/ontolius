//! Routines for loading ontology data.
#[cfg(feature = "obographs")]
pub mod obographs;

use anyhow::Context;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
};

/// Requirements for an ontology graph node index.
pub trait Index: Clone {
    fn new(val: usize) -> Self;
}

macro_rules! impl_index {
    ($TYPE:ty) => {
        impl Index for $TYPE {
            fn new(val: usize) -> Self {
                assert!(val <= <$TYPE>::MAX as usize);
                val as $TYPE
            }
        }
    };
}

impl_index!(u8);
impl_index!(u16);
impl_index!(u32);
impl_index!(u64);
impl_index!(usize);

/// A relationship between the ontology concepts.
///
/// At this time, we only support `is_a` relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Relationship {
    /// Subject is the parent of the object.
    Parent,
    /// Subject is the child of the object.
    Child,
    /// Subject is part of the object.
    ///
    /// Corresponds to [part of](http://purl.obolibrary.org/obo/BFO_0000050).
    PartOf,
}

/// A representation of an ontology graph edge.
///
/// The edge consists of three parts:
/// * `I` with the index of the source term
/// * [`Relationship`] with one of supported relationships
/// * `I` with the index of the destination term
#[derive(Debug, Clone, Hash)]
pub struct GraphEdge<I> {
    pub sub: I,
    pub pred: Relationship,
    pub obj: I,
}

impl<I> From<(I, Relationship, I)> for GraphEdge<I> {
    fn from(value: (I, Relationship, I)) -> Self {
        Self {
            sub: value.0,
            pred: value.1,
            obj: value.2,
        }
    }
}

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
    fn load_from_buf_read<R>(&self, read: R) -> anyhow::Result<OntologyData<I, T>>
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
    pub fn load_from_path<I, T, O, Q>(&self, path: Q) -> anyhow::Result<O>
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
    pub fn load_from_read<I, T, O, R>(&self, read: R) -> anyhow::Result<O>
    where
        P: OntologyDataParser<I, T>,
        R: Read,
        O: TryFrom<OntologyData<I, T>, Error = anyhow::Error>,
    {
        self.load_from_buf_read(BufReader::new(read))
    }

    /// Load ontology from a buffered reader.
    pub fn load_from_buf_read<I, T, O, R>(&self, read: R) -> anyhow::Result<O>
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
