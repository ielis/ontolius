use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

use crate::error::OntographError;
use crate::hierarchy::{
    AncestorNodes, ChildNodes, DescendantNodes, GraphEdge, HierarchyIdx, OntologyHierarchy,
    ParentNodes, Relationship,
};

use graph_builder::index::Idx as CsrIdx;
use graph_builder::GraphBuilder;
use graph_builder::{DirectedCsrGraph, DirectedNeighbors};

// TODO: here graph_builder is part of the public API through `I`.
/// An ontology graph backed by a CSR adjacency matrix.
pub struct CsrOntologyHierarchy<I>
where
    I: CsrIdx + HierarchyIdx + Hash,
{
    root_idx: I,
    adjacency_matrix: DirectedCsrGraph<I>,
}

impl<I> TryFrom<&[GraphEdge<I>]> for CsrOntologyHierarchy<I>
where
    I: CsrIdx + HierarchyIdx + Hash,
{
    type Error = OntographError;
    // TODO: we do not need an array, we need IntoIterator!
    fn try_from(graph_edges: &[GraphEdge<I>]) -> Result<Self, Self::Error> {
        let root_idx = find_root_idx(graph_edges).map(Clone::clone)?;

        let adjacency_matrix = GraphBuilder::new()
            .csr_layout(graph_builder::CsrLayout::Sorted)
            .edges(make_edge_iterator(graph_edges))
            .build();

        Ok(CsrOntologyHierarchy {
            root_idx,
            adjacency_matrix,
        })
    }
}

fn find_root_idx<I>(graph_edges: &[GraphEdge<I>]) -> Result<&I, OntographError>
where
    I: Hash + HierarchyIdx,
{
    let mut root_candidate_set: HashSet<&I> = HashSet::new();
    let mut remove_mark_set: HashSet<&I> = HashSet::new();

    for edge in graph_edges.iter() {
        match edge.pred {
            Relationship::Child => {
                root_candidate_set.insert(&edge.obj);
                remove_mark_set.insert(&edge.sub);
            }
            Relationship::Parent => {
                root_candidate_set.insert(&edge.obj);
                remove_mark_set.insert(&edge.sub);
            }
        }
    }

    let candidates: Vec<_> = root_candidate_set.difference(&remove_mark_set).collect();

    match candidates.len() {
        0 => Err(OntographError::OntologyAssemblyError(
            "No root candidate found!".into(),
        )),
        1 => Ok(*candidates[0]),
        _ => Err(OntographError::OntologyAssemblyError(
            "More than one root candidate found".into(),
        )),
    }
}

fn make_edge_iterator<I>(graph_edges: &[GraphEdge<I>]) -> impl Iterator<Item = (I, I)> + '_
where
    I: HierarchyIdx,
{
    graph_edges.iter().flat_map(|edge| {
        match edge.pred {
            // `sub -> is_a -> obj`` is what we want!
            Relationship::Child => Some((edge.sub, edge.obj)),
            Relationship::Parent => Some((edge.obj, edge.sub)),
        }
    })
}

impl<I> ChildNodes for CsrOntologyHierarchy<I>
where
    I: CsrIdx + HierarchyIdx + Hash,
{
    type I = I;
    type ChildIter<'a> = std::slice::Iter<'a, I> where I: 'a;

    fn children_of(&self, node: I) -> Self::ChildIter<'_> {
        self.adjacency_matrix.in_neighbors(node)
    }
}

impl<I> ParentNodes for CsrOntologyHierarchy<I>
where
    I: CsrIdx + HierarchyIdx + Hash,
{
    type I = I;
    type ParentIter<'a> = std::slice::Iter<'a, I> where I: 'a;

    fn parents_of(&self, node: I) -> Self::ParentIter<'_> {
        self.adjacency_matrix.out_neighbors(node)
    }
}

impl<I> DescendantNodes for CsrOntologyHierarchy<I>
where
    I: CsrIdx + HierarchyIdx + Hash,
{
    type I = I;
    type DescendantIter<'a> = DescendantsIter<'a, I>;

    fn descendants_of(&self, node: I) -> Self::DescendantIter<'_> {
        DescendantsIter {
            adjacency_matrix: &self.adjacency_matrix,
            seen: HashSet::new(),
            queue: VecDeque::from_iter(self.adjacency_matrix.in_neighbors(node)),
        }
    }
}

pub struct DescendantsIter<'a, I>
where
    I: CsrIdx + HierarchyIdx + Hash,
{
    adjacency_matrix: &'a DirectedCsrGraph<I>,
    seen: HashSet<&'a I>,
    queue: VecDeque<&'a I>,
}

impl<'a, I> Iterator for DescendantsIter<'a, I>
where
    I: CsrIdx + HierarchyIdx + Hash,
{
    type Item = &'a I;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(i) = self.queue.pop_front() {
            if self.seen.insert(i) {
                // newly inserted
                self.queue.extend(self.adjacency_matrix.in_neighbors(*i));
                return Some(i);
            }
        }
        None
    }
}

impl<I> AncestorNodes for CsrOntologyHierarchy<I>
where
    I: CsrIdx + HierarchyIdx + Hash,
{
    type I = I;
    type AncestorIter<'a> = AncestorIter<'a, I>
    where
        Self: 'a;

    fn ancestors_of(&self, node: I) -> Self::AncestorIter<'_> {
        AncestorIter {
            adjacency_matrix: &self.adjacency_matrix,
            seen: HashSet::new(),
            queue: VecDeque::from_iter(self.adjacency_matrix.out_neighbors(node)),
        }
    }
}

pub struct AncestorIter<'a, I>
where
    I: CsrIdx + HierarchyIdx + Hash,
{
    adjacency_matrix: &'a DirectedCsrGraph<I>,
    seen: HashSet<&'a I>,
    queue: VecDeque<&'a I>,
}

impl<'a, I> Iterator for AncestorIter<'a, I>
where
    I: CsrIdx + HierarchyIdx + Hash,
{
    type Item = &'a I;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(i) = self.queue.pop_front() {
            if self.seen.insert(i) {
                // newly inserted
                self.queue.extend(self.adjacency_matrix.out_neighbors(*i));
                return Some(i);
            }
        }
        None
    }
}

impl<I> OntologyHierarchy for CsrOntologyHierarchy<I>
where
    I: CsrIdx + HierarchyIdx + Hash,
{
    type HI = I;

    /// Get index of the ontology root.
    fn root(&self) -> &I {
        &self.root_idx
    }

    fn subhierarchy(&self, subroot: I) -> Self {
        // TODO: implement
        let mut edge_map: HashMap<&I, HashSet<&I>> = HashMap::new();
        for descendant in std::iter::once(&subroot).chain(self.descendants_of(subroot)) {
            for child in self.children_of(*descendant) {
                edge_map.entry(child).or_default().insert(descendant);
            }
        }

        let mut edges = Vec::new();
        for (&child, parents) in edge_map {
            for &parent in parents {
                edges.push((child, parent))
            }
        }

        let adjacency_matrix = GraphBuilder::new()
            .csr_layout(graph_builder::CsrLayout::Sorted)
            .edges(edges)
            .build();
        let _hierarchy = CsrOntologyHierarchy {
            root_idx: subroot,
            adjacency_matrix,
        };
        // TODO: may not be right, because it assumes we'll keep the same array of nodes!
        todo!()
    }
}

#[cfg(test)]
mod test_hierarchy {

    use graph_builder::CsrLayout;

    use super::*;

    macro_rules! check_members {
        ($hierarchy: expr, $func: expr, $i: expr, $exp: expr) => {
            let expected = HashSet::from($exp);
            let actual = $func(&$hierarchy, $i)
                .map(|val| *val)
                .collect::<HashSet<_>>();

            assert_eq!(actual, expected);
        };
    }

    #[test]
    fn test_children_of() {
        let hierarchy = build_example_hierarchy();
        let func = CsrOntologyHierarchy::children_of;

        check_members!(hierarchy, func, 0, [1, 5, 9]);
        check_members!(hierarchy, func, 1, [2, 3]);
        check_members!(hierarchy, func, 2, [4]);
        check_members!(hierarchy, func, 3, [4]);
        check_members!(hierarchy, func, 4, [0; 0]);
        check_members!(hierarchy, func, 5, [6, 7, 8]);
        check_members!(hierarchy, func, 6, [0; 0]);
        check_members!(hierarchy, func, 7, [0; 0]);
        check_members!(hierarchy, func, 8, [0; 0]);
        check_members!(hierarchy, func, 9, [0; 0]);
    }

    #[test]
    fn test_descendants_of() {
        let hierarchy = build_example_hierarchy();
        let func = CsrOntologyHierarchy::descendants_of;

        check_members!(hierarchy, func, 0, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
        check_members!(hierarchy, func, 1, [2, 3, 4]);
        check_members!(hierarchy, func, 2, [4]);
        check_members!(hierarchy, func, 3, [4]);
        check_members!(hierarchy, func, 4, [0; 0]);
        check_members!(hierarchy, func, 5, [6, 7, 8]);
        check_members!(hierarchy, func, 6, [0; 0]);
        check_members!(hierarchy, func, 7, [0; 0]);
        check_members!(hierarchy, func, 8, [0; 0]);
        check_members!(hierarchy, func, 9, [0; 0]);
    }

    #[test]
    fn test_parents_of() {
        let hierarchy = build_example_hierarchy();
        let func = CsrOntologyHierarchy::parents_of;

        check_members!(hierarchy, func, 0, [0; 0]);
        check_members!(hierarchy, func, 1, [0]);
        check_members!(hierarchy, func, 2, [1]);
        check_members!(hierarchy, func, 3, [1]);
        check_members!(hierarchy, func, 4, [2, 3]);
        check_members!(hierarchy, func, 5, [0]);
        check_members!(hierarchy, func, 6, [5]);
        check_members!(hierarchy, func, 7, [5]);
        check_members!(hierarchy, func, 8, [5]);
        check_members!(hierarchy, func, 9, [0]);
    }

    #[test]
    fn test_ancestors_of() {
        let hierarchy = build_example_hierarchy();
        let func = CsrOntologyHierarchy::ancestors_of;

        check_members!(hierarchy, func, 0, [0; 0]);
        check_members!(hierarchy, func, 1, [0]);
        check_members!(hierarchy, func, 2, [0, 1]);
        check_members!(hierarchy, func, 3, [0, 1]);
        check_members!(hierarchy, func, 4, [0, 1, 2, 3]);
        check_members!(hierarchy, func, 5, [0]);
        check_members!(hierarchy, func, 6, [0, 5]);
        check_members!(hierarchy, func, 7, [0, 5]);
        check_members!(hierarchy, func, 8, [0, 5]);
        check_members!(hierarchy, func, 9, [0]);
    }

    fn build_example_hierarchy() -> CsrOntologyHierarchy<u16> {
        let root_idx = 0;
        // let nodes = vec![
        //     "HP:1", "HP:01", "HP:010", "HP:011", "HP:0110", "HP:02", "HP:020", "HP:021",
        //     "HP:022", "HP:03",
        // ];
        let edges = vec![
            (1, 0),
            (2, 1),
            (3, 1),
            (4, 2),
            (4, 3),
            (5, 0),
            (6, 5),
            (7, 5),
            (8, 5),
            (9, 0),
        ];

        let adjacency_matrix = GraphBuilder::new()
            .csr_layout(CsrLayout::Sorted)
            .edges(edges)
            .build();
        CsrOntologyHierarchy {
            root_idx,
            adjacency_matrix,
        }
    }
}

#[cfg(test)]
mod create_csr_hierarchy {

    use super::*;

    #[test]
    fn try_from_graph_edges() {
        let edges = [
            GraphEdge::from((1, Relationship::Child, 0)),
            GraphEdge::from((2, Relationship::Child, 1)),
            GraphEdge::from((3, Relationship::Child, 1)),
        ];
        let hierarchy = CsrOntologyHierarchy::try_from(edges.as_slice());
        assert!(hierarchy.is_ok());
    }
}
