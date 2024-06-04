//! A simple model of ontology hierarchy graph
//! that is based on the `is_a` relationships.
//!
//! The hierarchy can be queried for children,
//! descendants, parents, and ancestors
//! of an ontology node.
//!
//! Check out the [`OntologyHierarchy`] documentation
//! for more info on the provided functionality.
mod edge;

pub use edge::{GraphEdge, Relationship};

/// Trait for types that can provide the child nodes of an ontology node.
pub trait ChildNodes {
    // Type used to index the ontology nodes.
    type I: HierarchyIdx;
    type ChildIter<'a>: Iterator<Item = &'a Self::I>
    where
        Self: 'a,
        Self::I: 'a;

    /// Returns an iterator of all nodes which are children of `node`.
    fn children_of(&self, node: Self::I) -> Self::ChildIter<'_>;

    /// Test if `sub` is child of the `obj` node.
    fn is_child_of(&self, sub: Self::I, obj: Self::I) -> bool {
        self.children_of(obj).any(|&child| child == sub)
    }

    /// Test if `node` is a leaf, i.e. a node with no child nodes.
    fn is_leaf(&self, node: Self::I) -> bool {
        self.children_of(node).count() == 0
    }
}

/// Trait for types that can provide the descendant nodes of an ontology node.
pub trait DescendantNodes {
    // Type used to index the ontology nodes.
    type I: HierarchyIdx;
    type DescendantIter<'a>: Iterator<Item = &'a Self::I>
    where
        Self: 'a,
        Self::I: 'a;

    /// Returns an iterator of all nodes which are descendants of `node`.
    fn descendants_of(&self, node: Self::I) -> Self::DescendantIter<'_>;
}

/// Trait for types that can provide the parent nodes of an ontology node.
pub trait ParentNodes {
    // Type used to index the ontology nodes.
    type I: HierarchyIdx;
    type ParentIter<'a>: Iterator<Item = &'a Self::I>
    where
        Self: 'a,
        Self::I: 'a;

    /// Returns an iterator of all nodes which are parents of `node`.
    fn parents_of(&self, node: Self::I) -> Self::ParentIter<'_>;

    /// Test if `sub` is parent of the `obj` node.
    fn is_parent_of(&self, sub: Self::I, obj: Self::I) -> bool {
        self.parents_of(obj).any(|&parent| parent == sub)
    }
}

/// Trait for types that can provide the ancestor nodes of an ontology node.
pub trait AncestorNodes {
    // Type used to index the ontology nodes.
    type I: HierarchyIdx;
    type AncestorIter<'a>: Iterator<Item = &'a Self::I>
    where
        Self: 'a,
        Self::I: 'a;

    /// Returns an iterator of all nodes which are ancestors of `node`.
    fn ancestors_of(&self, node: Self::I) -> Self::AncestorIter<'_>;

    /// Test if `sub` is an ancestor of `obj`.
    fn is_ancestor_of(&self, sub: Self::I, obj: Self::I) -> bool {
        self.ancestors_of(obj).any(|&anc| anc == sub)
    }

    /// Test if `sub`` is a descendant of `obj`.
    fn is_descendant_of(&self, sub: Self::I, obj: Self::I) -> bool {
        self.ancestors_of(sub).any(|&parent| parent == obj)
    }
}

/// Trait for types that support all basic ontology hierarchy operations,
/// such as getting the parents, ancestors, children and descendants
/// of an ontology node.
pub trait OntologyHierarchy:
    ChildNodes<I = Self::HI>
    + DescendantNodes<I = Self::HI>
    + ParentNodes<I = Self::HI>
    + AncestorNodes<I = Self::HI>
{
    // Type used to index the ontology nodes.
    type HI: HierarchyIdx;

    /// Get index of the root element.
    fn root(&self) -> &Self::HI;

    // TODO: augment a container with ancestors & self
    // TODO: augment a container with descendants & self

    fn subhierarchy(&self, subroot_idx: Self::HI) -> Self;
}

/// The implementors can be used to index the [`super::OntologyHierarchy`].
pub trait HierarchyIdx: Copy + Ord {
    fn new(idx: usize) -> Self;
}

macro_rules! impl_idx {
    ($TYPE:ty) => {
        impl HierarchyIdx for $TYPE {
            fn new(idx: usize) -> Self {
                assert!(idx <= <$TYPE>::MAX as usize);
                idx as $TYPE
            }
        }
    };
}

impl_idx!(u8);
impl_idx!(u16);
impl_idx!(u32);
impl_idx!(u64);
impl_idx!(usize);

impl_idx!(i8);
impl_idx!(i16);
impl_idx!(i32);
impl_idx!(i64);
impl_idx!(isize);
