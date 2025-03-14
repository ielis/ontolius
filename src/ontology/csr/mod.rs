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
//! use ontolius::io::OntologyLoaderBuilder;
//! use ontolius::ontology::csr::CsrOntology;
//! use ontolius::ontology::OntologyTerms;
//! use ontolius::term::simple::SimpleMinimalTerm;
//!
//! // Configure the ontology loader to parse Obographs JSON file.
//! let loader = OntologyLoaderBuilder::new()
//!                .obographs_parser()
//!                .build();
//!
//! // Load a small Obographs JSON file into `CsrOntology`.
//! // Use `u32` as ontology graph index type.
//! let path = "resources/hp.small.json.gz";
//!
//! /// Use `flate2` to decompress JSON on the fly ...
//! let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
//! let ontology: CsrOntology<u32, SimpleMinimalTerm> = loader.load_from_read(reader)
//!                                         .expect("Obographs JSON should be parsable");
//!
//! // ... or do the same using the `MinimalCsrOntology` alias to save some typing:
//! use ontolius::ontology::csr::MinimalCsrOntology;
//!
//! let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
//! let ontology: MinimalCsrOntology = loader.load_from_read(reader)
//!                                         .expect("Obographs JSON should be parsable");
//!
//! // Check the number of primary terms
//! assert_eq!(ontology.len(), 614);
//! ```

mod beta;

pub use beta::CsrOntology;

use crate::term::simple::{SimpleMinimalTerm, SimpleTerm};

/*
Using `u32` as a default because `u16` led to performance regression in ancestor/descendant
hierarchy traversals (manual benchmark in `bences/hierarchy_traversals.rs`).
*/
/// A [`CsrOntology`] with [`u32`] used as node indexer and [`SimpleMinimalTerm`] as the term.
pub type MinimalCsrOntology = CsrOntology<u32, SimpleMinimalTerm>;

/// A [`CsrOntology`] with [`u32`] used as node indexer and [`SimpleTerm`] as the term.
pub type FullCsrOntology = CsrOntology<u32, SimpleTerm>;
