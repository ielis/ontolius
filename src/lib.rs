//! `ontolius` is a library to empower algorithms 
//! that use Human Phenotype Ontology (HPO).
//! 
//! The project is in *alpha* stage. 
//! We will extend the documentation with more prose later.
//! 
//! # Examples
//! 
//! We provide examples of *loading* ontology and its suggested usage
//! in applications.
//! 
//! ## Load HPO
//! 
//! `ontolius` supports loading HPO from Obographs JSON file. 
//! However, the API is built to support other formats in future.
//!
//! We can load the JSON file as follows:
//! 
//! ```rust
//! use std::path::Path;
//! 
//! use curie_util::TrieCurieUtil;
//! use ontolius::io::obographs::ObographsParser;
//! use ontolius::io::OntologyLoaderBuilder;
//! use ontolius::ontology::csr::CsrOntology;
//! 
//! // Load a toy Obographs file from the repo
//! let path = Path::new("resources/hp.small.json");
//! 
//! // Configure the loader to parse the input as an Obographs file
//! let loader = OntologyLoaderBuilder::new()
//!                .parser(ObographsParser::new(TrieCurieUtil::default()))
//!                .build();
//! 
//! let hpo: CsrOntology<usize, _> = loader.load_from_path(path)
//!                                    .expect("HPO should be loaded");
//! ```
//! 
//! We loaded an ontology from a toy JSON file. During the load, each term 
//! is assigned a numeric index and the indices are used as vertices 
//! of the ontology graph. 
//! 
//! As the name suggests, `CsrOntology` is backed by a CSR adjacency matrix.
//! However, the ontology type should be treated as an implementation detail 
//! and the important part here is that `CsrOntology` implements 
//! the [`crate::ontology::Ontology`] trait.
//! 
//! That's it for loading, let's move on to the example usage.
//! 
//! ## Use HPO
//! 
//! In the previous section, we loaded an ontology from Obographs JSON file.
//! Now we have an instance of [`crate::ontology::Ontology`] that can 
//! be used for various tasks.
//! 
//! Note, we will import the *prelude* [`crate::prelude`] to simplify imports.
//! 
//! ### Work with ontology terms
//! 
//! [`crate::ontology::Ontology`] acts as a container of terms to support 
//! retrieval of specific terms by its index or `TermId`, and to iterate 
//! over all terms and `TermId`s. 
//! 
//! We can get a term by its `TermId`: 
//! 
//! ```rust
//! # use std::path::Path;
//! # use curie_util::TrieCurieUtil;
//! # use ontolius::io::obographs::ObographsParser;
//! # use ontolius::io::OntologyLoaderBuilder;
//! # use ontolius::ontology::csr::CsrOntology;
//! # let loader = OntologyLoaderBuilder::new().parser(ObographsParser::new(TrieCurieUtil::default())).build();
//! # let hpo: CsrOntology<usize, _> = loader.load_from_path(Path::new("resources/hp.small.json"))
//! #                                    .expect("HPO should be loaded");
//! #
//! use ontolius::prelude::*;
//! 
//! // `HP:0001250` corresponds to `Arachnodactyly``
//! let term_id: TermId = ("HP", "0001166").into();
//! 
//! // Get the term by its term ID and check the name. 
//! let term = hpo.id_to_term(&term_id).expect("Arachnodactyly should be present");
//! 
//! assert_eq!(term.name(), "Arachnodactyly");
//! ```
//! 
//! or iterate over the terms or term IDs:
//! 
//! ```rust
//! # use std::path::Path;
//! # use curie_util::TrieCurieUtil;
//! # use ontolius::io::obographs::ObographsParser;
//! # use ontolius::io::OntologyLoaderBuilder;
//! # use ontolius::ontology::csr::CsrOntology;
//! # let loader = OntologyLoaderBuilder::new().parser(ObographsParser::new(TrieCurieUtil::default())).build();
//! # let hpo: CsrOntology<usize, _> = loader.load_from_path(Path::new("resources/hp.small.json"))
//! #                                    .expect("HPO should be loaded");
//! #
//! use ontolius::prelude::*;
//! 
//! // The toy HPO contains 614 terms and primary term ids,
//! let terms: Vec<_> = hpo.iter_terms().collect();
//! assert_eq!(terms.len(), 614);
//! assert_eq!(hpo.iter_term_ids().count(), 614);
//! 
//! // and the total of 1121 term ids (primary + obsolete)
//! assert_eq!(hpo.iter_all_term_ids().count(), 1121);
//! ```
//! 
//! See [`crate::ontology::HierarchyAware`] for more details.
//! 
//! ### Browse the hierarchy
//! 
//! `ontograph` models the ontology hierarchy using the [`crate::hierarchy::OntologyHierarchy`] type, 
//! an instance of which is available from `Ontology`. The hierarchy is represented as
//! a directed acyclic graph that is built from `is_a` relationships. The graph vertices 
//! correspond to term indices (not `TermId`s) that are determined during ontology loading.
//! 
//! All methods of the ontology hierarchy operate in the term index space. The indices have 
//! all properties of `TermId`s, and can, therefore, be used *in lieu* of the `TermId`s. 
//! Moreover, the indices provide substantial performance advantage  
//! since hashing and comparisons are much more efficient than in `TermId`.
//! 
//! Let's see how to use the hierarchy. For instance, get the parent terms of a term.
//! 
//! ```rust
//! # use std::path::Path;
//! # use curie_util::TrieCurieUtil;
//! # use ontolius::io::obographs::ObographsParser;
//! # use ontolius::io::OntologyLoaderBuilder;
//! # use ontolius::ontology::csr::CsrOntology;
//! # let loader = OntologyLoaderBuilder::new().parser(ObographsParser::new(TrieCurieUtil::default())).build();
//! # let hpo: CsrOntology<usize, _> = loader.load_from_path(Path::new("resources/hp.small.json"))
//! #                                    .expect("HPO should be loaded");
//! #
//! use ontolius::prelude::*;
//! 
//! let hierarchy = hpo.hierarchy();
//! 
//! let arachnodactyly: TermId = ("HP", "0001166").into();
//! 
//! let idx = hpo.id_to_idx(&arachnodactyly)
//!             .expect("Arachnodacyly should be in HPO");
//! let parents: Vec<_> = hierarchy.parents_of(idx)
//!                         .flat_map(|idx| hpo.idx_to_term(*idx))
//!                         .collect();
//! let names: Vec<_> = parents.iter().map(|term| term.name()).collect();
//! assert_eq!(vec!["Slender finger", "Long fingers"], names);
//! ```
//! 
//! Similar methods exist for getting ancestors, children, and descendent terms.
//! See [`crate::hierarchy::OntologyHierarchy`] for more details.
//! 
//! That's it for now.
//! 
//! TODO: Add more prose.
pub mod base;
pub mod error;
pub mod hierarchy;
pub mod io;
pub mod ontology;
pub mod prelude;
