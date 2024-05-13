use std::fmt::Debug;
use std::hash::Hash;

use super::HierarchyIdx;

/// A relationship between the ontology concepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Relationship {
    /// Subject is the parent of the object.
    Parent,
    /// Subject is the child of the object.
    Child,
}

/// A representation of an ontology graph edge.
///
/// The edge consists of three parts:
/// * `I` with the index of the source term
/// * [`Relationship`] with one of supported relationships
/// * `I` with the index of the destination term
#[derive(Debug, Clone, Hash)]
pub struct GraphEdge<I: HierarchyIdx> {
    pub sub: I,
    pub pred: Relationship,
    pub obj: I,
}

impl<I: HierarchyIdx> From<(I, Relationship, I)> for GraphEdge<I> {
    fn from(value: (I, Relationship, I)) -> Self {
        Self { sub: value.0, pred: value.1, obj: value.2 }
    }
}
