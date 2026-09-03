//! A module with APIs for working with ontologies.

mod api;
/// Implementation of ontology backed by a CSR adjacency matrix.
#[cfg(feature = "csr")]
pub mod csr;

#[allow(deprecated)]
pub use api::{HierarchyQueries, HierarchyTraversals, HierarchyWalks};
pub use api::{MetadataAware, OntologyTerms, TaxonomyQuery, TaxonomyTraversal, TaxonomyWalk};
