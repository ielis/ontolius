mod csr {

    use ontolius::ontology::{
        csr::MinimalCsrOntology, HierarchyQueries, HierarchyTraversals, HierarchyWalks,
        OntologyTerms,
    };

    #[test]
    fn test_csr_ontology_can_be_used_with_trait_bounds() {
        let o: Option<MinimalCsrOntology> = None;

        if o.is_some() {
            // This will never run. It does not matter,
            // since it is enough that the code compiles.
            let o: &MinimalCsrOntology = o.as_ref().unwrap();

            pretend_to_use_hierarchy_queries(o);
            pretend_to_use_hierarchy_walks(o);
            pretend_to_use_hierarchy_traversals(o);
            pretend_to_use_ontology_terms(o);
        }

        fn pretend_to_use_hierarchy_queries(_o: impl HierarchyQueries) {}
        fn pretend_to_use_hierarchy_walks(_o: impl HierarchyWalks) {}
        fn pretend_to_use_hierarchy_traversals<I>(_o: impl HierarchyTraversals<I>) {}
        fn pretend_to_use_ontology_terms<T>(_o: impl OntologyTerms<T>) {}
    }
}
