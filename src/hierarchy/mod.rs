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
    type I: Eq;

    /// Returns an iterator of all nodes which are children of `node`.
    #[deprecated(since = "0.1.3", note = "Use `iter_children_of` instead")]
    fn children_of(&self, node: &Self::I) -> impl Iterator<Item = &Self::I> {
        self.iter_children_of(node)
    }

    /// Returns an iterator of all nodes which are children of `node`.
    fn iter_children_of(&self, node: &Self::I) -> impl Iterator<Item = &Self::I>;

    /// Test if `sub` is child of the `obj` node.
    fn is_child_of(&self, sub: &Self::I, obj: &Self::I) -> bool {
        self.iter_children_of(obj).any(|child| *child == *sub)
    }

    /// Test if `node` is a leaf, i.e. a node with no child nodes.
    fn is_leaf(&self, node: &Self::I) -> bool {
        self.iter_children_of(node).count() == 0
    }

    /// Get an iterator for iterating over a node followed by all its children.
    fn iter_node_and_children_of<'a>(
        &'a self,
        node: &'a Self::I,
    ) -> impl Iterator<Item = &'a Self::I> {
        std::iter::once(node).chain(self.iter_children_of(node))
    }

    /// Augment the collection with children of the `source` node.
    fn augment_with_children<'a, T>(&'a self, source: &Self::I, collection: &mut T)
    where
        T: Extend<&'a Self::I>,
        Self: 'a,
    {
        collection.extend(self.iter_children_of(source))
    }

    /// Augment the collection with the source `node` and its children.
    fn augment_with_node_and_children<'a, T>(&'a self, node: &'a Self::I, collection: &mut T)
    where
        T: Extend<&'a Self::I>,
        Self: 'a,
    {
        collection.extend(self.iter_node_and_children_of(node));
    }
}

/// Trait for types that can provide the descendant nodes of an ontology node.
pub trait DescendantNodes {
    // Type used to index the ontology nodes.
    type I;

    /// Returns an iterator of all nodes which are descendants of `node`.
    #[deprecated(since = "0.1.3", note = "Use `iter_descendants_of` instead")]
    fn descendants_of(&self, node: &Self::I) -> impl Iterator<Item = &Self::I> {
        self.iter_descendants_of(node)
    }

    /// Returns an iterator of all nodes which are descendants of `node`.
    fn iter_descendants_of(&self, node: &Self::I) -> impl Iterator<Item = &Self::I>;

    /// Get an iterator for iterating over a node followed by all its descendants.
    fn iter_node_and_descendants_of<'a>(
        &'a self,
        node: &'a Self::I,
    ) -> impl Iterator<Item = &'a Self::I> {
        std::iter::once(node).chain(self.iter_descendants_of(node))
    }

    /// Augment the collection with *descendants* of the source `node`.
    fn augment_with_descendants<'a, T>(&'a self, node: &Self::I, collection: &mut T)
    where
        T: Extend<&'a Self::I>,
        Self: 'a,
    {
        collection.extend(self.iter_descendants_of(node))
    }

    /// Augment the collection with the source `node` and its *descendants*.
    fn augment_with_source_and_descendants<'a, T>(&'a self, node: &'a Self::I, collection: &mut T)
    where
        T: Extend<&'a Self::I>,
        Self: 'a,
    {
        collection.extend(self.iter_descendants_of(node));
    }
}

/// Trait for types that can provide the parent nodes of an ontology node.
pub trait ParentNodes {
    // Type used to index the ontology nodes.
    type I: Eq;

    /// Returns an iterator of all nodes which are parents of `node`.
    #[deprecated(since = "0.1.3", note = "Use `iter_parents_of` instead")]
    fn parents_of(&self, node: &Self::I) -> impl Iterator<Item = &Self::I> {
        self.iter_parents_of(node)
    }

    /// Returns an iterator of all nodes which are parents of `node`.
    fn iter_parents_of(&self, node: &Self::I) -> impl Iterator<Item = &Self::I>;

    /// Test if `sub` is parent of the `obj` node.
    fn is_parent_of(&self, sub: &Self::I, obj: &Self::I) -> bool {
        self.iter_parents_of(obj).any(|parent| *parent == *sub)
    }

    /// Get an iterator for iterating over a node followed by all its parents.
    fn iter_node_and_parents_of<'a>(
        &'a self,
        node: &'a Self::I,
    ) -> impl Iterator<Item = &'a Self::I> {
        std::iter::once(node).chain(self.iter_parents_of(node))
    }

    /// Augment the collection with *parents* of the source `node`.
    fn augment_with_parents<'a, T>(&'a self, node: &Self::I, collection: &mut T)
    where
        T: Extend<&'a Self::I>,
        Self: 'a,
    {
        collection.extend(self.iter_parents_of(node))
    }

    /// Augment the collection with the source `node` and its *parents*.
    fn augment_with_source_and_parents<'a, T>(&'a self, node: &'a Self::I, collection: &mut T)
    where
        T: Extend<&'a Self::I>,
        Self: 'a,
    {
        collection.extend(self.iter_node_and_parents_of(node));
    }
}

/// Trait for types that can provide the ancestor nodes of an ontology node.
pub trait AncestorNodes {
    // Type used to index the ontology nodes.
    type I: Eq;

    /// Returns an iterator of all nodes which are ancestors of `node`.
    #[deprecated(since = "0.1.3", note = "Use `iter_ancestors_of` instead")]
    fn ancestors_of(&self, node: &Self::I) -> impl Iterator<Item = &Self::I> {
        self.iter_ancestors_of(node)
    }

    /// Returns an iterator of all nodes which are ancestors of `node`.
    fn iter_ancestors_of(&self, node: &Self::I) -> impl Iterator<Item = &Self::I>;

    /// Test if `sub` is an ancestor of `obj`.
    fn is_ancestor_of(&self, sub: &Self::I, obj: &Self::I) -> bool {
        self.iter_ancestors_of(obj).any(|anc| *anc == *sub)
    }

    /// Test if `sub`` is a descendant of `obj`.
    fn is_descendant_of(&self, sub: &Self::I, obj: &Self::I) -> bool {
        self.iter_ancestors_of(sub).any(|parent| *parent == *obj)
    }

    /// Get an iterator for iterating over a node followed by all its ancestors.
    fn iter_node_and_ancestors_of<'a>(
        &'a self,
        node: &'a Self::I,
    ) -> impl Iterator<Item = &'a Self::I> {
        std::iter::once(node).chain(self.iter_ancestors_of(node))
    }

    /// Augment the collection with *ancestors* of the source `node`.
    fn augment_with_ancestors<'a, T>(&'a self, node: &Self::I, collection: &mut T)
    where
        T: Extend<&'a Self::I>,
        Self: 'a,
    {
        collection.extend(self.iter_ancestors_of(node))
    }

    /// Augment the collection with the source `node` and its *ancestors*.
    fn augment_with_node_and_ancestors<'a, T>(&'a self, node: &'a Self::I, collection: &mut T)
    where
        T: Extend<&'a Self::I>,
        Self: 'a,
    {
        collection.extend(self.iter_node_and_ancestors_of(node));
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
    type HI: Eq;

    /// Get index of the root element.
    fn root(&self) -> &Self::HI;

    fn subhierarchy(&self, subroot_idx: &Self::HI) -> Self;
}

/// The implementors can be used to index the [`super::OntologyHierarchy`].
pub trait HierarchyIdx: Copy + Eq {
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
