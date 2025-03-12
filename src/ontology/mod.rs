//! A module with APIs for working with ontologies.

mod api;
/// Implementation of ontology backed by a CSR adjacency matrix.
#[cfg(feature = "csr")]
pub mod csr;

pub use api::{
    HierarchyQueries, HierarchyTraversals, HierarchyWalks, MetadataAware, OntologyTerms,
};
