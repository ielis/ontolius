//! The errors used by the library.
use std::fmt::Debug;
use thiserror::Error;

/// The error
#[derive(Error, Debug)]
pub enum OntographError {
    
    /// Returned when the input data cannot be parsed into
    /// [`crate::io::OntologyData`].
    #[error("{0}")]
    OntologyDataParseError(String),
    
    /// Returned when the [`crate::io::OntologyData`] cannot
    /// be assembled into an [`crate::ontology::Ontology`].
    #[error("{0}")]
    OntologyAssemblyError(String),

    /// Fallback error with a message.
    #[error("{0}")]
    Other(String),

    /// Fallback error with no message.
    #[error("Unknown ontograph error")]
    Unknown,
}
