//! Module with `Ontology` backed by a term array 
//! and a CSR adjacency matrix.
//! 
//! # Example 
//! 
//! ```rust
//! use ontolius::io::OntologyLoaderBuilder;
//! use ontolius::ontology::csr::CsrOntology;
//! use ontolius::prelude::*;
//! 
//! // Configure the ontology loader to parse Obographs JSON file.
//! let loader = OntologyLoaderBuilder::new()
//!                .obographs_parser()
//!                .build();
//! 
//! // Load a small Obographs JSON file into `CsrOntology`.
//! // Use `usize` as ontology graph indices.
//! let path = "resources/hp.small.json.gz";
//! let ontology: CsrOntology<usize, _> = loader.load_from_path(path)
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
