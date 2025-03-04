use std::fmt::Debug;
use std::hash::Hash;

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
