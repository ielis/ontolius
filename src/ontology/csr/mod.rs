//! Module with `Ontology` backed by a term array
//! and a CSR adjacency matrix.
//!
//! # Example
//!
//! ```rust
//! use std::fs::File;
//! use std::io::BufReader;
//! use flate2::bufread::GzDecoder;
//!
//! use ontolius::prelude::*;
//! use ontolius::base::term::simple::SimpleMinimalTerm;
//! use ontolius::ontology::csr::CsrOntology;
//!
//! // Configure the ontology loader to parse Obographs JSON file.
//! let loader = OntologyLoaderBuilder::new()
//!                .obographs_parser()
//!                .build();
//!
//! // Load a small Obographs JSON file into `CsrOntology`.
//! // Use `usize` as ontology graph indices.
//! let path = "resources/hp.small.json.gz";
//!
//! /// Use `flate2` to decompress JSON on the fly
//! let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
//! let ontology: CsrOntology<usize, SimpleMinimalTerm> = loader.load_from_read(reader)
//!                                         .expect("Obographs JSON should be parsable");
//!
//! // or do the same using the `MinimalCsrOntology` alias to save some typing:
//! use ontolius::ontology::csr::MinimalCsrOntology;
//!
//! let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
//! let ontology: MinimalCsrOntology = loader.load_from_read(reader)
//!                                         .expect("Obographs JSON should be parsable");
//!
//! // Check the number of primary terms
//! assert_eq!(ontology.len(), 614);
//! ```
//!
//! Check the [`crate::ontology::Ontology`] documentation for more info
//! regarding the supported functionality.
mod hierarchy;
mod ontology;

pub use hierarchy::CsrOntologyHierarchy;
pub use ontology::CsrOntology;

use crate::base::term::simple::SimpleMinimalTerm;

/// A [`CsrOntology`] with [`usize`] used as node indexer and [`SimpleMinimalTerm`] as the term.
pub type MinimalCsrOntology = CsrOntology<usize, SimpleMinimalTerm>;
